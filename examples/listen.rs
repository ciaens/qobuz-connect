use std::hash::{DefaultHasher, Hash as _, Hasher as _};

use qobuz_connect::proto::qconnect::{
    AudioQuality, CtrlSrvrJoinSession, DeviceCapabilities, DeviceInfo, DeviceType, MessageType,
    QConnectMessage, VolumeRemoteControl,
};
use qobuz_connect::{Credentials, Transport, TransportEvent};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let Some(credentials) = credentials() else {
        eprintln!("set QOBUZ_CONNECT_ENDPOINT and QOBUZ_CONNECT_JWT");
        return;
    };
    let name = std::env::var("QOBUZ_CONNECT_NAME")
        .unwrap_or_else(|_| "qobuz-connect listen example".to_owned());
    let mut transport = match Transport::connect(credentials).await {
        Ok(transport) => transport,
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    };
    if transport.send(vec![join(&name)]).is_err() {
        return;
    }
    while let Some(event) = transport.recv().await {
        match event {
            TransportEvent::Message(message) => println!("{message:#?}"),
            TransportEvent::Disconnected => println!("disconnected"),
            TransportEvent::Reconnected => {
                println!("reconnected");
                if transport.send(vec![join(&name)]).is_err() {
                    return;
                }
            }
        }
    }
}

fn credentials() -> Option<Credentials> {
    Some(Credentials {
        endpoint: std::env::var("QOBUZ_CONNECT_ENDPOINT").ok()?,
        jwt: std::env::var("QOBUZ_CONNECT_JWT").ok()?,
    })
}

fn join(name: &str) -> QConnectMessage {
    QConnectMessage {
        message_type: MessageType::CtrlSrvrJoinSession.into(),
        ctrl_srvr_join_session: Some(CtrlSrvrJoinSession {
            session_uuid: None,
            device_info: Some(DeviceInfo {
                device_uuid: device_uuid(name),
                friendly_name: name.to_owned(),
                brand: "qobuz-connect".to_owned(),
                model: "listen example".to_owned(),
                serial_number: String::new(),
                r#type: DeviceType::Computer.into(),
                capabilities: Some(DeviceCapabilities {
                    min_audio_quality: AudioQuality::Mp3.into(),
                    max_audio_quality: AudioQuality::HiresLevel3.into(),
                    volume_remote_control: VolumeRemoteControl::Allowed.into(),
                }),
                software_version: env!("CARGO_PKG_VERSION").to_owned(),
            }),
        }),
        ..Default::default()
    }
}

fn device_uuid(name: &str) -> Vec<u8> {
    let mut uuid = Vec::with_capacity(16);
    for salt in 0_u8..2 {
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        salt.hash(&mut hasher);
        uuid.extend_from_slice(&hasher.finish().to_be_bytes());
    }
    uuid
}
