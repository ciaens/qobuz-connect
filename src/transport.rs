//! WebSocket connection to the Qobuz cloud carrying Qobuz Connect messages.

use std::time::{Duration, Instant};

use futures_util::{SinkExt as _, StreamExt as _};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;
use tokio_tungstenite::tungstenite::{self, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use crate::Error;
use crate::proto::qconnect::QConnectMessage;
use crate::wire::{self, Frame};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

const KEEPALIVE: Duration = Duration::from_secs(30);
const BACKOFF_MS: [u64; 8] = [100, 200, 500, 1_000, 2_000, 5_000, 10_000, 20_000];
const STABLE: Duration = Duration::from_secs(40);

/// Where to connect and how to authenticate.
#[derive(Debug, Clone)]
pub struct Credentials {
    pub endpoint: String,
    pub jwt: String,
}

/// What the cloud sends, and changes of the connection itself.
#[derive(Debug)]
pub enum Event {
    Message(Box<QConnectMessage>),
    /// The connection was lost; the transport is reconnecting.
    Disconnected,
    /// The connection is back and authenticated; a session has to join again.
    Reconnected,
}

/// A connection that authenticates, subscribes and reconnects on its own. Dropping it closes the connection.
pub struct Transport {
    events: mpsc::Receiver<Event>,
    outbound: mpsc::Sender<Vec<QConnectMessage>>,
}

#[derive(Default)]
struct Counters {
    frame: u32,
    batch: i32,
}

impl Counters {
    fn frame(&mut self) -> u32 {
        self.frame = self.frame.wrapping_add(1);
        self.frame
    }

    fn batch(&mut self) -> i32 {
        self.batch = self.batch.wrapping_add(1);
        self.batch
    }
}

enum Outcome {
    Lost,
    Refused,
    Closed,
}

impl Transport {
    /// Connects, authenticates and subscribes, then keeps the connection alive in a background task.
    pub async fn connect(credentials: Credentials) -> Result<Self, Error> {
        let mut counters = Counters::default();
        let socket = open(&credentials, &mut counters).await?;
        let (event_sender, events) = mpsc::channel(64);
        let (outbound, outbound_receiver) = mpsc::channel(16);
        tokio::spawn(run(
            credentials,
            counters,
            socket,
            outbound_receiver,
            event_sender,
        ));
        Ok(Self { events, outbound })
    }

    /// Next message or connection change, `None` once the connection is gone for good.
    pub async fn recv(&mut self) -> Option<Event> {
        self.events.recv().await
    }

    /// Sends messages to the backend in one payload frame.
    pub async fn send(&self, messages: Vec<QConnectMessage>) -> Result<(), Error> {
        self.outbound
            .send(messages)
            .await
            .map_err(|_| Error::Closed)
    }
}

async fn open(
    credentials: &Credentials,
    counters: &mut Counters,
) -> Result<Socket, tungstenite::Error> {
    let (mut socket, _) = connect_async(&credentials.endpoint).await?;
    let handshake = [
        wire::authenticate(counters.frame(), &credentials.jwt),
        wire::subscribe(counters.frame()),
    ];
    socket
        .send(Message::binary(wire::encode(&handshake)))
        .await?;
    Ok(socket)
}

async fn run(
    credentials: Credentials,
    mut counters: Counters,
    mut socket: Socket,
    mut outbound: mpsc::Receiver<Vec<QConnectMessage>>,
    events: mpsc::Sender<Event>,
) {
    let mut attempt = 0;
    loop {
        let started = Instant::now();
        match serve(&mut socket, &mut outbound, &events, &mut counters).await {
            Outcome::Closed => {
                let _ = socket.close(None).await;
                return;
            }
            Outcome::Refused => {
                tracing::info!("the server asked not to reconnect");
                return;
            }
            Outcome::Lost => {}
        }
        attempt = next_attempt(attempt, started.elapsed());
        if events.send(Event::Disconnected).await.is_err() {
            return;
        }
        let Some(reopened) = reopen(&credentials, &mut counters, &outbound, &mut attempt).await
        else {
            return;
        };
        socket = reopened;
        if events.send(Event::Reconnected).await.is_err() {
            return;
        }
    }
}

async fn reopen(
    credentials: &Credentials,
    counters: &mut Counters,
    outbound: &mpsc::Receiver<Vec<QConnectMessage>>,
    attempt: &mut usize,
) -> Option<Socket> {
    loop {
        tokio::time::sleep(backoff(*attempt)).await;
        if outbound.is_closed() {
            return None;
        }
        *attempt = attempt.saturating_add(1);
        let attempt = *attempt;
        match open(credentials, counters).await {
            Ok(socket) => {
                tracing::info!(attempt, "reconnected");
                return Some(socket);
            }
            Err(err) => tracing::warn!(attempt, %err, "reconnect failed"),
        }
    }
}

fn backoff(attempt: usize) -> Duration {
    let millis = BACKOFF_MS
        .get(attempt)
        .or(BACKOFF_MS.last())
        .copied()
        .unwrap_or_default();
    Duration::from_millis(millis)
}

fn next_attempt(attempt: usize, uptime: Duration) -> usize {
    if uptime >= STABLE { 0 } else { attempt }
}

async fn serve(
    socket: &mut Socket,
    outbound: &mut mpsc::Receiver<Vec<QConnectMessage>>,
    events: &mpsc::Sender<Event>,
    counters: &mut Counters,
) -> Outcome {
    let mut keepalive = tokio::time::interval(KEEPALIVE);
    keepalive.set_missed_tick_behavior(MissedTickBehavior::Delay);
    keepalive.tick().await;
    loop {
        tokio::select! {
            incoming = socket.next() => match incoming {
                Some(Ok(Message::Binary(bytes))) => {
                    if let Some(outcome) = receive(&bytes, events).await {
                        return outcome;
                    }
                }
                Some(Ok(Message::Close(_)) | Err(_)) | None => return Outcome::Lost,
                Some(Ok(_)) => {}
            },
            outgoing = outbound.recv() => match outgoing {
                Some(messages) => {
                    let frame = wire::payload(counters.frame(), counters.batch(), messages);
                    if socket.send(Message::binary(wire::encode(&[frame]))).await.is_err() {
                        return Outcome::Lost;
                    }
                }
                None => return Outcome::Closed,
            },
            _ = keepalive.tick() => {
                if socket.send(Message::Ping(Vec::new().into())).await.is_err() {
                    return Outcome::Lost;
                }
            }
        }
    }
}

async fn receive(bytes: &[u8], events: &mpsc::Sender<Event>) -> Option<Outcome> {
    let frames = match wire::decode(bytes) {
        Ok(frames) => frames,
        Err(err) => {
            tracing::warn!(%err, "dropping an undecodable message");
            return None;
        }
    };
    for frame in frames {
        match frame {
            Frame::Payload(payload) => match wire::messages(&payload) {
                Ok(messages) => {
                    for message in messages {
                        if events
                            .send(Event::Message(Box::new(message)))
                            .await
                            .is_err()
                        {
                            return Some(Outcome::Closed);
                        }
                    }
                }
                Err(err) => tracing::warn!(%err, "dropping an undecodable payload"),
            },
            Frame::Disconnect(disconnect) => {
                return Some(if disconnect.reconnect {
                    Outcome::Lost
                } else {
                    Outcome::Refused
                });
            }
            other => tracing::debug!(?other, "ignoring frame"),
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_follows_the_web_player_then_stays_capped() {
        assert_eq!(backoff(0), Duration::from_millis(100));
        assert_eq!(backoff(3), Duration::from_secs(1));
        assert_eq!(backoff(7), Duration::from_secs(20));
        assert_eq!(backoff(100), Duration::from_secs(20));
    }

    #[test]
    fn attempts_reset_only_after_a_stable_connection() {
        assert_eq!(next_attempt(5, Duration::from_secs(39)), 5);
        assert_eq!(next_attempt(5, Duration::from_secs(40)), 0);
    }
}
