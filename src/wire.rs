//! Frames of the Qobuz cloud socket. A WebSocket message carries one or more frames, each a varint kind, a varint length and a protobuf body.

use std::time::{SystemTime, UNIX_EPOCH};

use prost::Message as _;
use prost::encoding::{decode_varint, encode_varint};

use crate::proto::qcloud::{
    Authenticate, Disconnect, MessageType, Payload, Subscribe, Unsubscribe,
};
use crate::proto::qconnect::{QConnectBatch, QConnectMessage};

/// Protocol number of Qobuz Connect inside subscribe and payload frames.
pub const PROTO_QCONNECT: u32 = 1;
/// Channel id of the Qobuz backend, the destination of every message a device sends.
pub const BACKEND_CHANNEL: [u8; 1] = [2];

/// One frame of the cloud socket.
#[derive(Debug, Clone, PartialEq)]
pub enum Frame {
    Authenticate(Authenticate),
    Subscribe(Subscribe),
    Unsubscribe(Unsubscribe),
    Payload(Payload),
    Disconnect(Disconnect),
    Other { kind: u64, body: Vec<u8> },
}

impl Frame {
    fn parts(&self) -> (u64, Vec<u8>) {
        match self {
            Self::Authenticate(message) => {
                (kind(MessageType::Authenticate), message.encode_to_vec())
            }
            Self::Subscribe(message) => (kind(MessageType::Subscribe), message.encode_to_vec()),
            Self::Unsubscribe(message) => (kind(MessageType::Unsubscribe), message.encode_to_vec()),
            Self::Payload(message) => (kind(MessageType::Payload), message.encode_to_vec()),
            Self::Disconnect(message) => (kind(MessageType::Disconnect), message.encode_to_vec()),
            Self::Other { kind, body } => (*kind, body.clone()),
        }
    }

    fn parse(kind: u64, body: &[u8]) -> Result<Self, prost::DecodeError> {
        let frame = match MessageType::try_from(i32::try_from(kind).unwrap_or_default()) {
            Ok(MessageType::Authenticate) => Self::Authenticate(Authenticate::decode(body)?),
            Ok(MessageType::Subscribe) => Self::Subscribe(Subscribe::decode(body)?),
            Ok(MessageType::Unsubscribe) => Self::Unsubscribe(Unsubscribe::decode(body)?),
            Ok(MessageType::Payload) => Self::Payload(Payload::decode(body)?),
            Ok(MessageType::Disconnect) => Self::Disconnect(Disconnect::decode(body)?),
            Ok(MessageType::Unspecified | MessageType::Error) | Err(_) => Self::Other {
                kind,
                body: body.to_vec(),
            },
        };
        Ok(frame)
    }
}

fn kind(kind: MessageType) -> u64 {
    u64::try_from(i32::from(kind)).unwrap_or_default()
}

/// Serializes frames into one WebSocket message.
#[must_use]
pub fn encode(frames: &[Frame]) -> Vec<u8> {
    let mut out = Vec::new();
    for frame in frames {
        let (kind, body) = frame.parts();
        encode_varint(kind, &mut out);
        encode_varint(u64::try_from(body.len()).unwrap_or(u64::MAX), &mut out);
        out.extend_from_slice(&body);
    }
    out
}

/// Splits one WebSocket message into its frames.
pub fn decode(mut bytes: &[u8]) -> Result<Vec<Frame>, prost::DecodeError> {
    let mut frames = Vec::new();
    while !bytes.is_empty() {
        let kind = decode_varint(&mut bytes)?;
        let length = usize::try_from(decode_varint(&mut bytes)?)
            .map_err(|_| prost::DecodeError::new("frame length overflows"))?;
        let (body, rest) = bytes
            .split_at_checked(length)
            .ok_or_else(|| prost::DecodeError::new("truncated frame"))?;
        frames.push(Frame::parse(kind, body)?);
        bytes = rest;
    }
    Ok(frames)
}

#[must_use]
pub fn authenticate(msg_id: u32, jwt: &str) -> Frame {
    Frame::Authenticate(Authenticate {
        msg_id,
        msg_date: now_ms(),
        jwt: jwt.to_owned(),
    })
}

#[must_use]
pub fn subscribe(msg_id: u32) -> Frame {
    Frame::Subscribe(Subscribe {
        msg_id,
        msg_date: now_ms(),
        proto: PROTO_QCONNECT,
        channels: Vec::new(),
    })
}

/// Wraps Qobuz Connect messages into a payload frame addressed to the backend.
#[must_use]
pub fn payload(msg_id: u32, batch_id: i32, messages: Vec<QConnectMessage>) -> Frame {
    let batch = QConnectBatch {
        messages_time: now_ms(),
        messages_id: batch_id,
        messages,
    };
    Frame::Payload(Payload {
        msg_id,
        msg_date: now_ms(),
        proto: PROTO_QCONNECT,
        src: Vec::new(),
        dests: vec![BACKEND_CHANNEL.to_vec()],
        payload: batch.encode_to_vec(),
    })
}

/// Unpacks the Qobuz Connect messages carried by a payload frame.
pub fn messages(payload: &Payload) -> Result<Vec<QConnectMessage>, prost::DecodeError> {
    QConnectBatch::decode(payload.payload.as_slice()).map(|batch| batch.messages)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::qconnect::{self, MessageType as QConnectType};

    fn error_message(code: &str) -> QConnectMessage {
        QConnectMessage {
            message_type: QConnectType::Error.into(),
            error: Some(qconnect::Error {
                code: code.to_owned(),
                message: String::new(),
            }),
            ..Default::default()
        }
    }

    #[test]
    fn frames_round_trip_in_one_message() {
        let frames = [
            authenticate(1, "jwt"),
            subscribe(2),
            payload(3, 1, vec![error_message("e")]),
            Frame::Other {
                kind: 9,
                body: vec![1, 2, 3],
            },
        ];
        let decoded = decode(&encode(&frames));
        assert_eq!(decoded.ok().as_deref(), Some(frames.as_slice()));
    }

    #[test]
    fn payload_carries_messages_for_the_backend() {
        let messages = vec![error_message("a"), error_message("b")];
        let frame = match payload(5, 2, messages.clone()) {
            Frame::Payload(frame) => Some(frame),
            _ => None,
        };
        assert_eq!(
            frame.as_ref().map(|f| f.dests.clone()),
            Some(vec![BACKEND_CHANNEL.to_vec()])
        );
        assert_eq!(frame.as_ref().map(|f| f.proto), Some(PROTO_QCONNECT));
        assert_eq!(
            frame.as_ref().and_then(|f| super::messages(f).ok()),
            Some(messages)
        );
    }

    #[test]
    fn truncated_input_is_an_error() {
        let mut bytes = encode(&[subscribe(1)]);
        bytes.pop();
        assert!(decode(&bytes).is_err());
    }
}
