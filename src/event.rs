//! Everything a session can tell its owner, as typed events.

use crate::device::Device;
use crate::proto::qconnect::{
    AudioQuality, LoopMode, MessageType, NetworkType, PlaybackError, PlayingState, QConnectMessage,
    QueueVersion, RendererStatus, SrvrCtrlAutoplayModeSet, SrvrCtrlAutoplayTracksLoaded,
    SrvrCtrlAutoplayTracksRemoved, SrvrCtrlLoopModeSet, SrvrCtrlQueueCleared,
    SrvrCtrlQueueErrorMessage, SrvrCtrlQueueState, SrvrCtrlQueueTracksAdded,
    SrvrCtrlQueueTracksAddedFromAutoplay, SrvrCtrlQueueTracksInserted, SrvrCtrlQueueTracksLoaded,
    SrvrCtrlQueueTracksRemoved, SrvrCtrlQueueTracksReordered, SrvrCtrlShuffleModeSet,
};
use crate::renderer::{PlayerState, RendererCommand, id};

/// Snapshot of the session sent right after joining and whenever it changes.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionState {
    pub session_uuid: Vec<u8>,
    pub active_renderer_id: Option<i32>,
    pub queue_version: Option<QueueVersion>,
    pub playing: PlayingState,
    pub loop_mode: LoopMode,
}

/// Changes about the renderers of the session, this device included.
#[derive(Debug, Clone, PartialEq)]
pub enum RendererEvent {
    Added {
        id: i32,
        device: Device,
    },
    Updated {
        id: i32,
        device: Device,
    },
    Removed {
        id: i32,
    },
    ActiveChanged {
        id: Option<i32>,
    },
    StateUpdated {
        id: i32,
        status: RendererStatus,
        state: Option<PlayerState>,
    },
    Volume {
        id: i32,
        volume: u32,
    },
    Muted {
        id: i32,
        muted: bool,
    },
    MaxAudioQuality {
        id: i32,
        quality: AudioQuality,
        network: NetworkType,
    },
    FileAudioQuality {
        id: i32,
        sampling_rate: u32,
        bit_depth: u32,
        channels: u32,
        quality: AudioQuality,
    },
    DeviceAudioQuality {
        id: i32,
        sampling_rate: u32,
        bit_depth: u32,
        channels: u32,
    },
}

/// Changes of the shared queue, as sent by the server.
#[derive(Debug, Clone, PartialEq)]
pub enum QueueEvent {
    State(SrvrCtrlQueueState),
    Cleared(SrvrCtrlQueueCleared),
    Loaded(SrvrCtrlQueueTracksLoaded),
    Inserted(SrvrCtrlQueueTracksInserted),
    Added(SrvrCtrlQueueTracksAdded),
    AddedFromAutoplay(SrvrCtrlQueueTracksAddedFromAutoplay),
    Removed(SrvrCtrlQueueTracksRemoved),
    Reordered(SrvrCtrlQueueTracksReordered),
    ShuffleModeSet(SrvrCtrlShuffleModeSet),
    LoopModeSet(SrvrCtrlLoopModeSet),
    AutoplayModeSet(SrvrCtrlAutoplayModeSet),
    AutoplayTracksLoaded(SrvrCtrlAutoplayTracksLoaded),
    AutoplayTracksRemoved(SrvrCtrlAutoplayTracksRemoved),
    Error(SrvrCtrlQueueErrorMessage),
}

impl QueueEvent {
    /// The queue version this change resulted in, when the server states one.
    #[must_use]
    pub fn queue_version(&self) -> Option<&QueueVersion> {
        match self {
            Self::State(m) => m.queue_version.as_ref(),
            Self::Cleared(m) => m.queue_version.as_ref(),
            Self::Loaded(m) => m.queue_version.as_ref(),
            Self::Inserted(m) => m.queue_version.as_ref(),
            Self::Added(m) => m.queue_version.as_ref(),
            Self::AddedFromAutoplay(m) => m.queue_version.as_ref(),
            Self::Removed(m) => m.queue_version.as_ref(),
            Self::Reordered(m) => m.queue_version.as_ref(),
            Self::ShuffleModeSet(m) => m.queue_version.as_ref(),
            Self::AutoplayModeSet(m) => m.queue_version.as_ref(),
            Self::AutoplayTracksLoaded(m) => m.queue_version.as_ref(),
            Self::AutoplayTracksRemoved(m) => m.queue_version.as_ref(),
            Self::Error(m) => m.queue_version.as_ref(),
            Self::LoopModeSet(_) => None,
        }
    }
}

/// What a session reports to its owner.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// The connection was lost; the session reconnects and rejoins on its own.
    Disconnected,
    /// The connection is back and the join was sent again.
    Reconnected,
    /// The server registered this device as a renderer.
    Registered {
        renderer_id: i32,
    },
    Command(RendererCommand),
    Session(SessionState),
    Renderer(RendererEvent),
    Queue(QueueEvent),
    Error {
        code: String,
        message: String,
    },
    PlaybackError(PlaybackError),
    /// A message type this crate does not map, given by its number.
    Unknown(i32),
}

pub(crate) fn from_message(message: QConnectMessage) -> Event {
    if let Some(command) = RendererCommand::from_message(&message) {
        return Event::Command(command);
    }
    let kind = message.message_type();
    let event = match kind {
        MessageType::Error => message.error.map(|e| Event::Error {
            code: e.code,
            message: e.message,
        }),
        MessageType::PlaybackError => message.playback_error.map(Event::PlaybackError),
        MessageType::SrvrCtrlSessionState => message.srvr_ctrl_session_state.map(|s| {
            let (playing, loop_mode) = (s.playing_state(), s.loop_mode());
            Event::Session(SessionState {
                session_uuid: s.session_uuid,
                active_renderer_id: id(s.active_renderer_id),
                queue_version: s.queue_version,
                playing,
                loop_mode,
            })
        }),
        _ => renderer_event(kind, &message)
            .map(Event::Renderer)
            .or_else(|| queue_event(kind, message).map(Event::Queue)),
    };
    event.unwrap_or(Event::Unknown(kind.into()))
}

fn renderer_event(kind: MessageType, message: &QConnectMessage) -> Option<RendererEvent> {
    match kind {
        MessageType::SrvrCtrlAddRenderer => {
            message
                .srvr_ctrl_add_renderer
                .as_ref()
                .map(|r| RendererEvent::Added {
                    id: r.renderer_id,
                    device: Device::from_proto(r.device_info.clone().unwrap_or_default()),
                })
        }
        MessageType::SrvrCtrlUpdateRenderer => {
            message
                .srvr_ctrl_update_renderer
                .as_ref()
                .map(|r| RendererEvent::Updated {
                    id: r.renderer_id,
                    device: Device::from_proto(r.device_info.clone().unwrap_or_default()),
                })
        }
        MessageType::SrvrCtrlRemoveRenderer => message
            .srvr_ctrl_remove_renderer
            .as_ref()
            .map(|r| RendererEvent::Removed { id: r.renderer_id }),
        MessageType::SrvrCtrlActiveRendererChanged => message
            .srvr_ctrl_active_renderer_changed
            .as_ref()
            .map(|a| RendererEvent::ActiveChanged {
                id: id(a.active_renderer_id),
            }),
        MessageType::SrvrCtrlRendererStateUpdated => message
            .srvr_ctrl_renderer_state_updated
            .as_ref()
            .map(|u| RendererEvent::StateUpdated {
                id: u.renderer_id,
                status: u.status(),
                state: u.player_state.as_ref().map(PlayerState::from_proto),
            }),
        MessageType::SrvrCtrlVolumeChanged => {
            message
                .srvr_ctrl_volume_changed
                .as_ref()
                .map(|v| RendererEvent::Volume {
                    id: v.renderer_id,
                    volume: v.volume,
                })
        }
        MessageType::SrvrCtrlVolumeMuted => {
            message
                .srvr_ctrl_volume_muted
                .as_ref()
                .map(|m| RendererEvent::Muted {
                    id: m.renderer_id,
                    muted: m.value,
                })
        }
        MessageType::SrvrCtrlMaxAudioQualityChanged => message
            .srvr_ctrl_max_audio_quality_changed
            .as_ref()
            .map(|q| RendererEvent::MaxAudioQuality {
                id: q.renderer_id,
                quality: q.max_audio_quality(),
                network: q.network_type(),
            }),
        MessageType::SrvrCtrlFileAudioQualityChanged => message
            .srvr_ctrl_file_audio_quality_changed
            .as_ref()
            .map(|q| RendererEvent::FileAudioQuality {
                id: q.renderer_id,
                sampling_rate: q.sampling_rate,
                bit_depth: q.bit_depth,
                channels: q.nb_channels,
                quality: q.audio_quality(),
            }),
        MessageType::SrvrCtrlDeviceAudioQualityChanged => message
            .srvr_ctrl_device_audio_quality_changed
            .as_ref()
            .map(|q| RendererEvent::DeviceAudioQuality {
                id: q.renderer_id,
                sampling_rate: q.sampling_rate,
                bit_depth: q.bit_depth,
                channels: q.nb_channels,
            }),
        _ => None,
    }
}

fn queue_event(kind: MessageType, message: QConnectMessage) -> Option<QueueEvent> {
    match kind {
        MessageType::SrvrCtrlQueueState => message.srvr_ctrl_queue_state.map(QueueEvent::State),
        MessageType::SrvrCtrlQueueCleared => {
            message.srvr_ctrl_queue_cleared.map(QueueEvent::Cleared)
        }
        MessageType::SrvrCtrlQueueTracksLoaded => message
            .srvr_ctrl_queue_tracks_loaded
            .map(QueueEvent::Loaded),
        MessageType::SrvrCtrlQueueTracksInserted => message
            .srvr_ctrl_queue_tracks_inserted
            .map(QueueEvent::Inserted),
        MessageType::SrvrCtrlQueueTracksAdded => {
            message.srvr_ctrl_queue_tracks_added.map(QueueEvent::Added)
        }
        MessageType::SrvrCtrlQueueTracksAddedFromAutoplay => message
            .srvr_ctrl_queue_tracks_added_from_autoplay
            .map(QueueEvent::AddedFromAutoplay),
        MessageType::SrvrCtrlQueueTracksRemoved => message
            .srvr_ctrl_queue_tracks_removed
            .map(QueueEvent::Removed),
        MessageType::SrvrCtrlQueueTracksReordered => message
            .srvr_ctrl_queue_tracks_reordered
            .map(QueueEvent::Reordered),
        MessageType::SrvrCtrlShuffleModeSet => message
            .srvr_ctrl_shuffle_mode_set
            .map(QueueEvent::ShuffleModeSet),
        MessageType::SrvrCtrlLoopModeSet => {
            message.srvr_ctrl_loop_mode_set.map(QueueEvent::LoopModeSet)
        }
        MessageType::SrvrCtrlAutoplayModeSet => message
            .srvr_ctrl_autoplay_mode_set
            .map(QueueEvent::AutoplayModeSet),
        MessageType::SrvrCtrlAutoplayTracksLoaded => message
            .srvr_ctrl_autoplay_tracks_loaded
            .map(QueueEvent::AutoplayTracksLoaded),
        MessageType::SrvrCtrlAutoplayTracksRemoved => message
            .srvr_ctrl_autoplay_tracks_removed
            .map(QueueEvent::AutoplayTracksRemoved),
        MessageType::SrvrCtrlQueueErrorMessage => {
            message.srvr_ctrl_queue_error_message.map(QueueEvent::Error)
        }
        _ => None,
    }
}
