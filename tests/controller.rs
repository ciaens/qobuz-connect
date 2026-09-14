#![allow(clippy::unwrap_used, clippy::panic)]

mod common;

use std::time::Duration;

use common::{Server, frames, listen, messages, send};
use qobuz_connect::proto::qconnect::{
    AudioQuality, DeviceType, Error as ProtoError, LoopMode, MessageType, PlayingState,
    QConnectMessage, QueueItemRef, QueueTrack, QueueVersion, SrvrCtrlQueueErrorMessage,
    SrvrCtrlQueueTracksAdded, SrvrCtrlSessionState,
};
use qobuz_connect::{Autoplay, ControllerCommand, Device, Event, QueueEvent, Session};
use tokio::sync::mpsc;
use tokio::time::timeout;

fn device() -> Device {
    Device {
        uuid: [3; 16],
        name: "test controller".to_owned(),
        brand: "qobuz-connect".to_owned(),
        model: "test".to_owned(),
        kind: DeviceType::Computer,
        max_audio_quality: AudioQuality::HiresLevel3,
        volume_remote_control: false,
        software_version: "0".to_owned(),
    }
}

fn version(major: u32, minor: u32) -> QueueVersion {
    QueueVersion { major, minor }
}

async fn joined() -> (Session, Server, mpsc::Receiver<Server>) {
    let (credentials, mut connections) = listen().await;
    let mut session = Session::join(credentials, device()).await.unwrap();
    let mut server = connections.recv().await.unwrap();
    frames(&mut server).await;
    messages(&mut server).await;
    let state = QConnectMessage {
        message_type: MessageType::SrvrCtrlSessionState.into(),
        srvr_ctrl_session_state: Some(SrvrCtrlSessionState {
            session_uuid: vec![1; 16],
            active_renderer_id: -1,
            queue_version: Some(version(1, 2)),
            playing_state: PlayingState::Paused.into(),
            loop_mode: LoopMode::Off.into(),
        }),
        ..Default::default()
    };
    send(&mut server, vec![state]).await;
    assert!(matches!(session.recv().await, Some(Event::Session(_))));
    messages(&mut server).await;
    (session, server, connections)
}

async fn nothing_sent(server: &mut Server) {
    assert!(
        timeout(Duration::from_millis(200), frames(server))
            .await
            .is_err()
    );
}

async fn next(server: &mut Server) -> QConnectMessage {
    messages(server).await.into_iter().next().unwrap()
}

#[tokio::test]
async fn queue_changes_wait_for_the_previous_answer_and_carry_its_version() {
    let (mut session, mut server, _connections) = joined().await;
    session
        .control(ControllerCommand::AddTracks {
            track_ids: vec![1, 2],
            shuffle_seed: None,
            autoplay: Autoplay::default(),
        })
        .await
        .unwrap();
    session
        .control(ControllerCommand::RemoveTracks {
            queue_item_ids: vec![5],
            autoplay: Autoplay {
                reset: true,
                loading: false,
            },
        })
        .await
        .unwrap();

    let add = next(&mut server).await.ctrl_srvr_queue_add_tracks.unwrap();
    assert_eq!(add.queue_version_ref, Some(version(1, 2)));
    assert_eq!(add.track_ids, vec![1, 2]);
    assert_eq!(add.action_uuid.len(), 16);
    assert_eq!(add.context_uuid.len(), 16);
    nothing_sent(&mut server).await;

    let added = SrvrCtrlQueueTracksAdded {
        queue_version: Some(version(1, 3)),
        action_uuid: add.action_uuid.clone(),
        tracks: vec![
            QueueTrack {
                queue_item_id: 9,
                track_id: 1,
            },
            QueueTrack {
                queue_item_id: 10,
                track_id: 2,
            },
        ],
        ..Default::default()
    };
    let message = QConnectMessage {
        message_type: MessageType::SrvrCtrlQueueTracksAdded.into(),
        srvr_ctrl_queue_tracks_added: Some(added.clone()),
        ..Default::default()
    };
    send(&mut server, vec![message]).await;
    assert_eq!(
        session.recv().await,
        Some(Event::Queue(QueueEvent::Added(added)))
    );
    assert_eq!(session.queue_version(), Some(&version(1, 3)));

    let remove = next(&mut server)
        .await
        .ctrl_srvr_queue_remove_tracks
        .unwrap();
    assert_eq!(remove.queue_version_ref, Some(version(1, 3)));
    assert_eq!(remove.queue_item_ids, vec![5]);
    assert!(remove.autoplay_reset);
    assert_ne!(remove.action_uuid, add.action_uuid);
}

#[tokio::test]
async fn a_queue_error_drops_what_follows_and_asks_for_the_queue_again() {
    let (mut session, mut server, _connections) = joined().await;
    session
        .control(ControllerCommand::ClearQueue)
        .await
        .unwrap();
    session
        .control(ControllerCommand::SetLoopMode(LoopMode::RepeatAll))
        .await
        .unwrap();
    let clear = next(&mut server).await.ctrl_srvr_clear_queue.unwrap();
    nothing_sent(&mut server).await;

    let error = SrvrCtrlQueueErrorMessage {
        queue_version: Some(version(2, 0)),
        action_uuid: clear.action_uuid,
        error: Some(ProtoError {
            code: "stale".to_owned(),
            message: "queue version mismatch".to_owned(),
        }),
    };
    let message = QConnectMessage {
        message_type: MessageType::SrvrCtrlQueueErrorMessage.into(),
        srvr_ctrl_queue_error_message: Some(error.clone()),
        ..Default::default()
    };
    send(&mut server, vec![message]).await;
    assert_eq!(
        session.recv().await,
        Some(Event::Queue(QueueEvent::Error(error)))
    );

    let ask = next(&mut server)
        .await
        .ctrl_srvr_ask_for_queue_state
        .unwrap();
    assert_eq!(ask.queue_version_ref, Some(version(2, 0)));
    nothing_sent(&mut server).await;
}

#[tokio::test]
async fn renderer_commands_go_straight_out_and_player_state_names_the_queue_version() {
    let (mut session, mut server, _connections) = joined().await;
    session
        .control(ControllerCommand::SetPlayerState {
            playing: Some(PlayingState::Playing),
            position: Some(Duration::from_millis(1500)),
            queue_item_id: Some(4),
        })
        .await
        .unwrap();
    session
        .control(ControllerCommand::SetVolume {
            renderer_id: 3,
            volume: 40,
        })
        .await
        .unwrap();
    session
        .control(ControllerCommand::ChangeVolume {
            renderer_id: 3,
            delta: -5,
        })
        .await
        .unwrap();
    session
        .control(ControllerCommand::Mute {
            renderer_id: 3,
            muted: true,
        })
        .await
        .unwrap();
    session
        .control(ControllerCommand::SetMaxAudioQuality {
            renderer_id: 3,
            quality: AudioQuality::HiresLevel1,
        })
        .await
        .unwrap();

    let state = next(&mut server).await.ctrl_srvr_set_player_state.unwrap();
    assert_eq!(state.playing_state, Some(PlayingState::Playing.into()));
    assert_eq!(state.current_position, Some(1500));
    assert_eq!(
        state.current_queue_item,
        Some(QueueItemRef {
            queue_version: Some(version(1, 2)),
            id: 4,
        })
    );
    let volume = next(&mut server).await.ctrl_srvr_set_volume.unwrap();
    assert_eq!(
        (volume.renderer_id, volume.volume, volume.volume_delta),
        (3, Some(40), None)
    );
    let delta = next(&mut server).await.ctrl_srvr_set_volume.unwrap();
    assert_eq!((delta.volume, delta.volume_delta), (None, Some(-5)));
    let mute = next(&mut server).await.ctrl_srvr_mute_volume.unwrap();
    assert!(mute.value);
    let quality = next(&mut server)
        .await
        .ctrl_srvr_set_max_audio_quality
        .unwrap();
    assert_eq!(quality.max_audio_quality(), AudioQuality::HiresLevel1);
}
