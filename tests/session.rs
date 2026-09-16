#![allow(clippy::unwrap_used, clippy::panic)]

mod common;

use std::time::Duration;

use common::{Server, frames, listen, messages, nothing_sent, send};
use qobuz_connect::proto::qconnect::{
    AudioQuality, BufferState, DeviceInfo, DeviceType, LoopMode, MessageType, PlayingState,
    QConnectMessage, QueueTrackRef, QueueVersion, SrvrCtrlAddRenderer, SrvrCtrlSessionState,
    SrvrRndrSetState,
};
use qobuz_connect::{
    Device, Error, Event, PlayerState, RendererCommand, RendererReport, Session, SessionState,
};
use tokio::sync::mpsc;

fn device() -> Device {
    Device {
        uuid: [7; 16],
        name: "test renderer".to_owned(),
        brand: "qobuz-connect".to_owned(),
        model: "test".to_owned(),
        kind: DeviceType::Speaker,
        max_audio_quality: AudioQuality::HiresLevel3,
        volume_remote_control: true,
        software_version: "0".to_owned(),
    }
}

fn add_renderer(renderer_id: i32, device: &Device) -> QConnectMessage {
    QConnectMessage {
        message_type: MessageType::SrvrCtrlAddRenderer.into(),
        srvr_ctrl_add_renderer: Some(SrvrCtrlAddRenderer {
            renderer_id,
            device_info: Some(DeviceInfo {
                device_uuid: device.uuid.to_vec(),
                friendly_name: device.name.clone(),
                ..Default::default()
            }),
        }),
        ..Default::default()
    }
}

fn session_state(session_uuid: [u8; 16], active_renderer_id: i32) -> QConnectMessage {
    QConnectMessage {
        message_type: MessageType::SrvrCtrlSessionState.into(),
        srvr_ctrl_session_state: Some(SrvrCtrlSessionState {
            session_uuid: session_uuid.to_vec(),
            active_renderer_id,
            queue_version: Some(QueueVersion { major: 1, minor: 2 }),
            playing_state: PlayingState::Paused.into(),
            loop_mode: LoopMode::Off.into(),
        }),
        ..Default::default()
    }
}

async fn joined() -> (Session, Server, mpsc::Receiver<Server>) {
    let (credentials, mut connections) = listen().await;
    let session = Session::join(credentials, device()).await.unwrap();
    let mut server = connections.recv().await.unwrap();
    frames(&mut server).await;
    let join = messages(&mut server).await;
    let sent = join
        .first()
        .and_then(|m| m.ctrl_srvr_join_session.as_ref())
        .and_then(|j| j.device_info.as_ref());
    assert_eq!(
        sent.map(|d| d.friendly_name.as_str()),
        Some("test renderer")
    );
    assert_eq!(sent.map(|d| d.device_uuid.clone()), Some(vec![7; 16]));
    (session, server, connections)
}

#[tokio::test]
async fn registration_and_session_state_drive_the_bookkeeping() {
    let (mut session, mut server, _connections) = joined().await;

    send(&mut server, vec![add_renderer(7, &device())]).await;
    assert_eq!(
        session.recv().await,
        Some(Event::Registered { renderer_id: 7 })
    );
    assert_eq!(session.renderer_id(), Some(7));

    send(&mut server, vec![session_state([1; 16], -1)]).await;
    let expected = SessionState {
        session_uuid: vec![1; 16],
        active_renderer_id: None,
        queue_version: Some(QueueVersion { major: 1, minor: 2 }),
        playing: PlayingState::Paused,
        loop_mode: LoopMode::Off,
    };
    assert_eq!(session.recv().await, Some(Event::Session(expected)));
    assert!(!session.is_active());

    let asked = messages(&mut server).await;
    let ask = asked
        .first()
        .and_then(|m| m.ctrl_srvr_ask_for_queue_state.as_ref())
        .unwrap();
    assert_eq!(
        ask.queue_version_ref,
        Some(QueueVersion { major: 1, minor: 2 })
    );
    assert_eq!(ask.action_uuid.len(), 16);
}

#[tokio::test]
async fn commands_arrive_typed_and_reports_carry_the_queue_version() {
    let (mut session, mut server, _connections) = joined().await;
    let track = QueueTrackRef {
        queue_item_id: 6,
        track_id: 388_712_168,
        context_uuid: None,
    };
    let set_state = QConnectMessage {
        message_type: MessageType::SrvrRndrSetState.into(),
        srvr_rndr_set_state: Some(SrvrRndrSetState {
            playing_state: Some(PlayingState::Playing.into()),
            current_position: Some(5000),
            queue_version: Some(QueueVersion { major: 3, minor: 1 }),
            current_track: Some(track.clone()),
            next_track: None,
        }),
        ..Default::default()
    };
    send(&mut server, vec![set_state]).await;

    let expected = RendererCommand::SetState {
        playing: Some(PlayingState::Playing),
        position: Some(Duration::from_secs(5)),
        current: Some(track),
        next: None,
    };
    assert_eq!(session.recv().await, Some(Event::Command(expected)));

    let state = PlayerState {
        playing: PlayingState::Playing,
        buffer: BufferState::Ok,
        position: Duration::from_secs(5),
        duration: Duration::from_secs(246),
        current_queue_item_id: Some(6),
        next_queue_item_id: None,
    };
    session.report(RendererReport::State(state)).unwrap();
    let reported = messages(&mut server).await;
    let reported = reported
        .first()
        .and_then(|m| m.rndr_srvr_state_updated.as_ref())
        .and_then(|u| u.state.as_ref())
        .unwrap();
    assert_eq!(
        reported.queue_version,
        Some(QueueVersion { major: 3, minor: 1 })
    );
    assert_eq!(reported.current_queue_item_id, 6);
    assert_eq!(reported.next_queue_item_id, -1);
    assert_eq!(
        reported.current_position.as_ref().map(|p| p.value),
        Some(5000)
    );
}

#[tokio::test]
async fn rejoins_with_the_session_uuid_after_a_reconnect() {
    let (mut session, mut server, mut connections) = joined().await;
    send(&mut server, vec![session_state([9; 16], -1)]).await;
    assert!(matches!(session.recv().await, Some(Event::Session(_))));
    messages(&mut server).await;
    drop(server);

    assert_eq!(session.recv().await, Some(Event::Disconnected));
    let mut server = connections.recv().await.unwrap();
    frames(&mut server).await;
    assert_eq!(session.recv().await, Some(Event::Reconnected));
    let join = messages(&mut server).await;
    let uuid = join
        .first()
        .and_then(|m| m.ctrl_srvr_join_session.as_ref())
        .map(|j| j.session_uuid.clone());
    assert_eq!(uuid, Some(Some(vec![9; 16])));
}

#[tokio::test]
async fn activation_needs_a_renderer_id() {
    let (mut session, mut server, _connections) = joined().await;
    assert!(matches!(session.activate(), Err(Error::NotRegistered)));

    send(&mut server, vec![add_renderer(7, &device())]).await;
    session.recv().await;
    session.activate().unwrap();
    let sent = messages(&mut server).await;
    let renderer_id = sent
        .first()
        .and_then(|m| m.ctrl_srvr_set_active_renderer.as_ref())
        .map(|a| a.renderer_id);
    assert_eq!(renderer_id, Some(7));
}

#[tokio::test]
async fn a_reconnect_drops_what_was_queued_and_registers_again() {
    let (mut session, mut server, mut connections) = joined().await;
    send(&mut server, vec![add_renderer(7, &device())]).await;
    session.recv().await;
    drop(server);
    assert_eq!(session.recv().await, Some(Event::Disconnected));
    session.report(RendererReport::Muted(true)).unwrap();

    let mut server = connections.recv().await.unwrap();
    frames(&mut server).await;
    assert_eq!(session.recv().await, Some(Event::Reconnected));
    assert_eq!(session.renderer_id(), None);
    let join = messages(&mut server).await;
    assert!(
        join.first()
            .is_some_and(|m| m.ctrl_srvr_join_session.is_some())
    );
    nothing_sent(&mut server).await;

    send(&mut server, vec![add_renderer(8, &device())]).await;
    assert_eq!(
        session.recv().await,
        Some(Event::Registered { renderer_id: 8 })
    );
    assert_eq!(session.renderer_id(), Some(8));
}
