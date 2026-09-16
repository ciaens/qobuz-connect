#![allow(clippy::unwrap_used, clippy::panic)]

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use common::{frames, listen, messages, send};
use futures_util::SinkExt as _;
use qobuz_connect::proto::qcloud::Disconnect;
use qobuz_connect::proto::qconnect::{self, MessageType, QConnectMessage};
use qobuz_connect::wire::{self, Frame};
use qobuz_connect::{Credentials, Transport, TransportEvent};
use tokio_tungstenite::tungstenite::Message;

fn error_message(code: &str) -> QConnectMessage {
    QConnectMessage {
        message_type: MessageType::Error.into(),
        error: Some(qconnect::Error {
            code: code.to_owned(),
            message: String::new(),
        }),
        ..Default::default()
    }
}

#[tokio::test]
async fn authenticates_subscribes_and_exchanges_payloads() {
    let (credentials, mut connections) = listen().await;
    let mut transport = Transport::connect(credentials).await.unwrap();
    let mut server = connections.recv().await.unwrap();

    let handshake = frames(&mut server).await;
    assert!(matches!(
        handshake.as_slice(),
        [Frame::Authenticate(auth), Frame::Subscribe(sub)]
            if auth.jwt == "jwt" && sub.proto == wire::PROTO_QCONNECT && sub.channels.is_empty()
    ));

    let inbound = error_message("in");
    send(&mut server, vec![inbound.clone()]).await;
    assert!(
        matches!(transport.recv().await, Some(TransportEvent::Message(message)) if *message == inbound)
    );

    let outbound = error_message("out");
    transport.send(vec![outbound.clone()]).unwrap();
    let sent = frames(&mut server).await;
    let [Frame::Payload(payload)] = sent.as_slice() else {
        panic!("expected one payload frame, got {sent:?}");
    };
    assert_eq!(payload.dests, vec![wire::BACKEND_CHANNEL.to_vec()]);
    assert_eq!(wire::messages(payload).unwrap(), vec![outbound]);
}

#[tokio::test]
async fn reconnects_after_the_connection_drops() {
    let (credentials, mut connections) = listen().await;
    let mut transport = Transport::connect(credentials).await.unwrap();
    let mut first = connections.recv().await.unwrap();
    frames(&mut first).await;
    drop(first);

    assert!(matches!(
        transport.recv().await,
        Some(TransportEvent::Disconnected)
    ));
    let mut second = connections.recv().await.unwrap();
    let handshake = frames(&mut second).await;
    assert!(matches!(
        handshake.as_slice(),
        [Frame::Authenticate(_), Frame::Subscribe(_)]
    ));
    assert!(matches!(
        transport.recv().await,
        Some(TransportEvent::Reconnected)
    ));
    transport.send(vec![error_message("again")]).unwrap();
    assert_eq!(messages(&mut second).await, vec![error_message("again")]);
}

#[tokio::test]
async fn stops_when_the_server_refuses_reconnection() {
    let (credentials, mut connections) = listen().await;
    let mut transport = Transport::connect(credentials).await.unwrap();
    let mut server = connections.recv().await.unwrap();
    frames(&mut server).await;

    let refusal = Frame::Disconnect(Disconnect {
        msg_id: 1,
        msg_date: 0,
        reconnect: false,
    });
    server
        .send(Message::binary(wire::encode(&[refusal])))
        .await
        .unwrap();
    assert!(transport.recv().await.is_none());
}

#[tokio::test]
async fn mints_a_token_for_every_connection() {
    let (credentials, mut connections) = listen().await;
    let minted = Arc::new(AtomicUsize::new(0));
    let counter = minted.clone();
    let mut transport = Transport::connect_with(move || {
        let count = counter.fetch_add(1, Ordering::SeqCst).saturating_add(1);
        let endpoint = credentials.endpoint.clone();
        async move {
            Ok(Credentials {
                endpoint,
                jwt: format!("jwt{count}"),
            })
        }
    })
    .await
    .unwrap();
    let mut first = connections.recv().await.unwrap();
    let handshake = frames(&mut first).await;
    assert!(matches!(
        handshake.as_slice(),
        [Frame::Authenticate(auth), _] if auth.jwt == "jwt1"
    ));
    drop(first);

    assert!(matches!(
        transport.recv().await,
        Some(TransportEvent::Disconnected)
    ));
    let mut second = connections.recv().await.unwrap();
    let handshake = frames(&mut second).await;
    assert!(matches!(
        handshake.as_slice(),
        [Frame::Authenticate(auth), _] if auth.jwt == "jwt2"
    ));
    assert!(matches!(
        transport.recv().await,
        Some(TransportEvent::Reconnected)
    ));
    assert_eq!(minted.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn a_silent_server_counts_as_a_lost_connection() {
    let (credentials, mut connections) = listen().await;
    let mut transport = Transport::connect(credentials).await.unwrap();
    let mut first = connections.recv().await.unwrap();
    frames(&mut first).await;
    send(&mut first, vec![error_message("alive")]).await;
    assert!(matches!(
        transport.recv().await,
        Some(TransportEvent::Message(_))
    ));

    tokio::time::pause();
    tokio::time::advance(Duration::from_secs(60)).await;
    tokio::time::resume();
    assert!(matches!(
        transport.recv().await,
        Some(TransportEvent::Disconnected)
    ));
    let mut second = connections.recv().await.unwrap();
    frames(&mut second).await;
    assert!(matches!(
        transport.recv().await,
        Some(TransportEvent::Reconnected)
    ));
    drop(first);
}
