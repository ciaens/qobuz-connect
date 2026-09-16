#![allow(dead_code, clippy::unwrap_used, clippy::panic)]

use std::time::Duration;

use futures_util::{SinkExt as _, StreamExt as _};
use qobuz_connect::Credentials;
use qobuz_connect::proto::qconnect::QConnectMessage;
use qobuz_connect::wire::{self, Frame};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;

pub type Server = WebSocketStream<TcpStream>;

pub async fn listen() -> (Credentials, mpsc::Receiver<Server>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("ws://{}", listener.local_addr().unwrap());
    let (sender, receiver) = mpsc::channel(4);
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let socket = tokio_tungstenite::accept_async(stream).await.unwrap();
            if sender.send(socket).await.is_err() {
                break;
            }
        }
    });
    let credentials = Credentials {
        endpoint,
        jwt: "jwt".to_owned(),
    };
    (credentials, receiver)
}

pub async fn frames(server: &mut Server) -> Vec<Frame> {
    loop {
        match server.next().await {
            Some(Ok(Message::Binary(bytes))) => return wire::decode(&bytes).unwrap(),
            Some(Ok(Message::Ping(_) | Message::Pong(_))) => {}
            other => panic!("expected a binary message, got {other:?}"),
        }
    }
}

pub async fn nothing_sent(server: &mut Server) {
    assert!(
        timeout(Duration::from_millis(200), frames(server))
            .await
            .is_err()
    );
}

pub async fn messages(server: &mut Server) -> Vec<QConnectMessage> {
    let sent = frames(server).await;
    let [Frame::Payload(payload)] = sent.as_slice() else {
        panic!("expected one payload frame, got {sent:?}");
    };
    wire::messages(payload).unwrap()
}

pub async fn send(server: &mut Server, messages: Vec<QConnectMessage>) {
    let frame = wire::payload(1, 1, messages);
    server
        .send(Message::binary(wire::encode(&[frame])))
        .await
        .unwrap();
}
