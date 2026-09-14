//! What the cloud asks of a renderer and what a renderer reports back.

use std::time::Duration;

use crate::device::Device;
use crate::proto::qconnect::{
    ActionType, AudioQuality, BufferState, ErrorType, LoopMode, MessageType, NetworkType,
    PlaybackError, PlayingState, Position, QConnectMessage, QueueRendererState, QueueTrackRef,
    QueueVersion, RendererState, RndrSrvrDeviceAudioQualityChanged, RndrSrvrDeviceInfoUpdated,
    RndrSrvrFileAudioQualityChanged, RndrSrvrMaxAudioQualityChanged, RndrSrvrRendererAction,
    RndrSrvrStateUpdated, RndrSrvrVolumeChanged, RndrSrvrVolumeMuted,
};
use crate::wire::now_ms;

const NONE: i32 = -1;

/// Playback state of a renderer as reported to the session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerState {
    pub playing: PlayingState,
    pub buffer: BufferState,
    pub position: Duration,
    pub duration: Duration,
    pub current_queue_item_id: Option<i32>,
    pub next_queue_item_id: Option<i32>,
}

impl PlayerState {
    pub(crate) fn from_proto(state: &RendererState) -> Self {
        Self {
            playing: state.playing_state(),
            buffer: state.buffer_state(),
            position: millis_to_duration(state.current_position.as_ref().map_or(0, |p| p.value)),
            duration: millis_to_duration(state.duration),
            current_queue_item_id: id(state.current_queue_item_id),
            next_queue_item_id: None,
        }
    }

    fn to_proto(&self, queue_version: Option<QueueVersion>) -> QueueRendererState {
        QueueRendererState {
            playing_state: self.playing.into(),
            buffer_state: self.buffer.into(),
            current_position: Some(Position {
                timestamp: now_ms(),
                value: millis(self.position),
            }),
            duration: millis(self.duration),
            queue_version,
            current_queue_item_id: self.current_queue_item_id.unwrap_or(NONE),
            next_queue_item_id: self.next_queue_item_id.unwrap_or(NONE),
        }
    }
}

/// A command the cloud sends to this renderer.
#[derive(Debug, Clone, PartialEq)]
pub enum RendererCommand {
    /// Absent fields are unchanged: a track means jump to it (from `position` or the start, playing unless told otherwise), otherwise `position` is a seek and `playing` a play or pause; a track with a negative queue item id means stop.
    SetState {
        playing: Option<PlayingState>,
        position: Option<Duration>,
        current: Option<QueueTrackRef>,
        next: Option<QueueTrackRef>,
    },
    SetVolume(u32),
    ChangeVolume(i32),
    Mute(bool),
    SetActive(bool),
    SetMaxAudioQuality(AudioQuality),
    SetLoopMode(LoopMode),
    SetShuffleMode(bool),
}

impl RendererCommand {
    pub(crate) fn from_message(message: &QConnectMessage) -> Option<Self> {
        match message.message_type() {
            MessageType::SrvrRndrSetState => {
                message
                    .srvr_rndr_set_state
                    .as_ref()
                    .map(|s| Self::SetState {
                        playing: playing(s.playing_state),
                        position: s.current_position.map(millis_to_duration),
                        current: s.current_track.clone(),
                        next: s.next_track.clone(),
                    })
            }
            MessageType::SrvrRndrSetVolume => message.srvr_rndr_set_volume.as_ref().and_then(|v| {
                match (v.volume, v.volume_delta) {
                    (Some(volume), _) => Some(Self::SetVolume(volume)),
                    (None, Some(delta)) => Some(Self::ChangeVolume(delta)),
                    (None, None) => None,
                }
            }),
            MessageType::SrvrRndrMuteVolume => message
                .srvr_rndr_mute_volume
                .as_ref()
                .map(|m| Self::Mute(m.value)),
            MessageType::SrvrRndrSetActive => message
                .srvr_rndr_set_active
                .as_ref()
                .map(|a| Self::SetActive(a.active)),
            MessageType::SrvrRndrSetMaxAudioQuality => message
                .srvr_rndr_set_max_audio_quality
                .as_ref()
                .map(|q| Self::SetMaxAudioQuality(q.max_audio_quality())),
            MessageType::SrvrRndrSetLoopMode => message
                .srvr_rndr_set_loop_mode
                .as_ref()
                .map(|l| Self::SetLoopMode(l.loop_mode())),
            MessageType::SrvrRndrSetShuffleMode => message
                .srvr_rndr_set_shuffle_mode
                .as_ref()
                .map(|s| Self::SetShuffleMode(s.shuffle_mode)),
            _ => None,
        }
    }
}

/// What this renderer tells the session about itself.
#[derive(Debug, Clone, PartialEq)]
pub enum RendererReport {
    State(PlayerState),
    Volume(u32),
    Muted(bool),
    MaxAudioQuality {
        quality: AudioQuality,
        network: NetworkType,
    },
    FileAudioQuality {
        sampling_rate: u32,
        bit_depth: u32,
        channels: u32,
        quality: AudioQuality,
    },
    DeviceAudioQuality {
        sampling_rate: u32,
        bit_depth: u32,
        channels: u32,
    },
    Device(Device),
    Action {
        action: ActionType,
        seek: Option<Duration>,
    },
    PlaybackError {
        queue_item_id: i32,
        error: ErrorType,
    },
}

impl RendererReport {
    pub(crate) fn into_message(self, queue_version: Option<QueueVersion>) -> QConnectMessage {
        let mut message = QConnectMessage::default();
        let kind = match self {
            Self::State(state) => {
                message.rndr_srvr_state_updated = Some(RndrSrvrStateUpdated {
                    state: Some(state.to_proto(queue_version)),
                });
                MessageType::RndrSrvrStateUpdated
            }
            Self::Volume(volume) => {
                message.rndr_srvr_volume_changed = Some(RndrSrvrVolumeChanged { volume });
                MessageType::RndrSrvrVolumeChanged
            }
            Self::Muted(value) => {
                message.rndr_srvr_volume_muted = Some(RndrSrvrVolumeMuted { value });
                MessageType::RndrSrvrVolumeMuted
            }
            Self::MaxAudioQuality { quality, network } => {
                message.rndr_srvr_max_audio_quality_changed =
                    Some(RndrSrvrMaxAudioQualityChanged {
                        max_audio_quality: quality.into(),
                        network_type: Some(network.into()),
                    });
                MessageType::RndrSrvrMaxAudioQualityChanged
            }
            Self::FileAudioQuality {
                sampling_rate,
                bit_depth,
                channels,
                quality,
            } => {
                message.rndr_srvr_file_audio_quality_changed =
                    Some(RndrSrvrFileAudioQualityChanged {
                        sampling_rate,
                        bit_depth,
                        nb_channels: channels,
                        audio_quality: quality.into(),
                    });
                MessageType::RndrSrvrFileAudioQualityChanged
            }
            Self::DeviceAudioQuality {
                sampling_rate,
                bit_depth,
                channels,
            } => {
                message.rndr_srvr_device_audio_quality_changed =
                    Some(RndrSrvrDeviceAudioQualityChanged {
                        sampling_rate,
                        bit_depth,
                        nb_channels: channels,
                    });
                MessageType::RndrSrvrDeviceAudioQualityChanged
            }
            Self::Device(device) => {
                message.rndr_srvr_device_info_updated = Some(RndrSrvrDeviceInfoUpdated {
                    device_info: Some(device.to_proto()),
                });
                MessageType::RndrSrvrDeviceInfoUpdated
            }
            Self::Action { action, seek } => {
                message.rndr_srvr_renderer_action = Some(RndrSrvrRendererAction {
                    seek_position: seek.map(millis),
                    action: action.into(),
                });
                MessageType::RndrSrvrRendererAction
            }
            Self::PlaybackError {
                queue_item_id,
                error,
            } => {
                message.playback_error = Some(PlaybackError {
                    queue_version,
                    queue_item_id,
                    error_type: error.into(),
                });
                MessageType::PlaybackError
            }
        };
        message.message_type = kind.into();
        message
    }
}

pub(crate) fn id(raw: i32) -> Option<i32> {
    (raw >= 0).then_some(raw)
}

fn playing(raw: Option<i32>) -> Option<PlayingState> {
    raw.and_then(|raw| PlayingState::try_from(raw).ok())
        .filter(|state| *state != PlayingState::Unknown)
}

fn millis(duration: Duration) -> u32 {
    u32::try_from(duration.as_millis()).unwrap_or(u32::MAX)
}

fn millis_to_duration(millis: u32) -> Duration {
    Duration::from_millis(u64::from(millis))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::qconnect::SrvrRndrSetState;

    #[test]
    fn missing_items_are_reported_as_minus_one() {
        let state = PlayerState {
            playing: PlayingState::Paused,
            buffer: BufferState::Ok,
            position: Duration::from_millis(1500),
            duration: Duration::from_secs(200),
            current_queue_item_id: None,
            next_queue_item_id: Some(4),
        };
        let proto = state.to_proto(Some(QueueVersion { major: 1, minor: 2 }));
        assert_eq!(proto.current_queue_item_id, -1);
        assert_eq!(proto.next_queue_item_id, 4);
        assert_eq!(proto.current_position.map(|p| p.value), Some(1500));
        assert_eq!(proto.duration, 200_000);
        assert_eq!(
            proto.queue_version,
            Some(QueueVersion { major: 1, minor: 2 })
        );
    }

    #[test]
    fn controller_view_of_a_renderer_state_round_trips() {
        let proto = RendererState {
            playing_state: PlayingState::Playing.into(),
            buffer_state: BufferState::Buffering.into(),
            current_position: Some(Position {
                timestamp: 1,
                value: 42,
            }),
            duration: 1000,
            current_queue_item_id: -1,
        };
        let state = PlayerState::from_proto(&proto);
        assert_eq!(state.playing, PlayingState::Playing);
        assert_eq!(state.buffer, BufferState::Buffering);
        assert_eq!(state.position, Duration::from_millis(42));
        assert_eq!(state.current_queue_item_id, None);
    }

    #[test]
    fn set_state_keeps_absent_fields_unchanged() {
        let message = QConnectMessage {
            message_type: MessageType::SrvrRndrSetState.into(),
            srvr_rndr_set_state: Some(SrvrRndrSetState {
                playing_state: Some(PlayingState::Unknown.into()),
                current_position: None,
                queue_version: None,
                current_track: None,
                next_track: None,
            }),
            ..Default::default()
        };
        let expected = RendererCommand::SetState {
            playing: None,
            position: None,
            current: None,
            next: None,
        };
        assert_eq!(RendererCommand::from_message(&message), Some(expected));
    }
}
