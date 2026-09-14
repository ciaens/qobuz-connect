//! What a controller asks of the session and of its renderers.

use std::time::Duration;

use crate::proto::qconnect::{
    AudioQuality, CtrlSrvrAutoplayLoadTracks, CtrlSrvrAutoplayRemoveTracks, CtrlSrvrClearQueue,
    CtrlSrvrMuteVolume, CtrlSrvrQueueAddTracks, CtrlSrvrQueueInsertTracks, CtrlSrvrQueueLoadTracks,
    CtrlSrvrQueueRemoveTracks, CtrlSrvrQueueReorderTracks, CtrlSrvrSetActiveRenderer,
    CtrlSrvrSetAutoplayMode, CtrlSrvrSetLoopMode, CtrlSrvrSetMaxAudioQuality,
    CtrlSrvrSetPlayerState, CtrlSrvrSetQueueState, CtrlSrvrSetShuffleMode, CtrlSrvrSetVolume,
    LoopMode, MessageType, PlayingState, QConnectMessage, QueueItemRef, QueueVersion, TrackRef,
};
use crate::renderer::millis;

/// Hints about the autoplay tracks that follow the queue: `reset` drops them, `loading` announces that new ones are on their way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Autoplay {
    pub reset: bool,
    pub loading: bool,
}

/// What a controller asks of the session and of its renderers.
#[derive(Debug, Clone, PartialEq)]
pub enum ControllerCommand {
    /// Absent fields are unchanged: a queue item means jump to it, otherwise `position` is a seek and `playing` a play or pause.
    SetPlayerState {
        playing: Option<PlayingState>,
        position: Option<Duration>,
        queue_item_id: Option<i32>,
    },
    SetActiveRenderer(i32),
    SetVolume {
        renderer_id: i32,
        volume: u32,
    },
    ChangeVolume {
        renderer_id: i32,
        delta: i32,
    },
    Mute {
        renderer_id: i32,
        muted: bool,
    },
    SetMaxAudioQuality {
        renderer_id: i32,
        quality: AudioQuality,
    },
    SetLoopMode(LoopMode),
    ClearQueue,
    /// Replaces the queue and starts at `position`; a shuffle seed turns shuffle on, with the track at `shuffle_pivot_index` first.
    LoadTracks {
        track_ids: Vec<u32>,
        position: u32,
        shuffle_seed: Option<u32>,
        shuffle_pivot_index: Option<i32>,
        autoplay: Autoplay,
    },
    /// Inserts after the given queue item, or at the front.
    InsertTracks {
        track_ids: Vec<u32>,
        after: Option<i32>,
        shuffle_seed: Option<u32>,
        autoplay: Autoplay,
    },
    AddTracks {
        track_ids: Vec<u32>,
        shuffle_seed: Option<u32>,
        autoplay: Autoplay,
    },
    RemoveTracks {
        queue_item_ids: Vec<i32>,
        autoplay: Autoplay,
    },
    /// Moves the given queue items after another one, or to the front.
    ReorderTracks {
        queue_item_ids: Vec<i32>,
        after: Option<i32>,
        autoplay: Autoplay,
    },
    /// Replaces the whole queue, as the official apps do when they take a session over.
    SetQueueState {
        tracks: Vec<TrackRef>,
        shuffle_mode: bool,
        shuffled_indexes: Vec<u32>,
        autoplay_mode: bool,
        autoplay_loading: bool,
        autoplay_tracks: Vec<TrackRef>,
    },
    SetShuffleMode {
        enabled: bool,
        seed: Option<u32>,
        pivot_queue_item_id: Option<i32>,
        autoplay: Autoplay,
    },
    SetAutoplayMode {
        enabled: bool,
        autoplay: Autoplay,
    },
    LoadAutoplayTracks(Vec<u32>),
    RemoveAutoplayTracks(Vec<i32>),
}

impl ControllerCommand {
    /// Whether the server answers this command with a queue event carrying the action uuid.
    pub(crate) fn changes_queue(&self) -> bool {
        !matches!(
            self,
            Self::SetPlayerState { .. }
                | Self::SetActiveRenderer(_)
                | Self::SetVolume { .. }
                | Self::ChangeVolume { .. }
                | Self::Mute { .. }
                | Self::SetMaxAudioQuality { .. }
                | Self::SetLoopMode(_)
        )
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn into_message(
        self,
        queue_version: Option<QueueVersion>,
        action: &[u8],
    ) -> QConnectMessage {
        let mut message = QConnectMessage::default();
        let action_uuid = action.to_vec();
        let kind = match self {
            Self::SetPlayerState {
                playing,
                position,
                queue_item_id,
            } => {
                message.ctrl_srvr_set_player_state = Some(CtrlSrvrSetPlayerState {
                    playing_state: playing.map(Into::into),
                    current_position: position.map(millis),
                    current_queue_item: queue_item_id.map(|id| QueueItemRef { queue_version, id }),
                });
                MessageType::CtrlSrvrSetPlayerState
            }
            Self::SetActiveRenderer(renderer_id) => {
                message.ctrl_srvr_set_active_renderer =
                    Some(CtrlSrvrSetActiveRenderer { renderer_id });
                MessageType::CtrlSrvrSetActiveRenderer
            }
            Self::SetVolume {
                renderer_id,
                volume,
            } => {
                message.ctrl_srvr_set_volume = Some(CtrlSrvrSetVolume {
                    renderer_id,
                    volume: Some(volume),
                    volume_delta: None,
                });
                MessageType::CtrlSrvrSetVolume
            }
            Self::ChangeVolume { renderer_id, delta } => {
                message.ctrl_srvr_set_volume = Some(CtrlSrvrSetVolume {
                    renderer_id,
                    volume: None,
                    volume_delta: Some(delta),
                });
                MessageType::CtrlSrvrSetVolume
            }
            Self::Mute { renderer_id, muted } => {
                message.ctrl_srvr_mute_volume = Some(CtrlSrvrMuteVolume {
                    renderer_id,
                    value: muted,
                });
                MessageType::CtrlSrvrMuteVolume
            }
            Self::SetMaxAudioQuality {
                renderer_id,
                quality,
            } => {
                message.ctrl_srvr_set_max_audio_quality = Some(CtrlSrvrSetMaxAudioQuality {
                    renderer_id,
                    max_audio_quality: quality.into(),
                });
                MessageType::CtrlSrvrSetMaxAudioQuality
            }
            Self::SetLoopMode(loop_mode) => {
                message.ctrl_srvr_set_loop_mode = Some(CtrlSrvrSetLoopMode {
                    loop_mode: loop_mode.into(),
                });
                MessageType::CtrlSrvrSetLoopMode
            }
            Self::ClearQueue => {
                message.ctrl_srvr_clear_queue = Some(CtrlSrvrClearQueue {
                    queue_version_ref: queue_version,
                    action_uuid,
                });
                MessageType::CtrlSrvrClearQueue
            }
            Self::LoadTracks {
                track_ids,
                position,
                shuffle_seed,
                shuffle_pivot_index,
                autoplay,
            } => {
                message.ctrl_srvr_queue_load_tracks = Some(CtrlSrvrQueueLoadTracks {
                    queue_version_ref: queue_version,
                    action_uuid,
                    track_ids,
                    queue_position: position,
                    shuffle_seed,
                    shuffle_pivot_index,
                    shuffle_mode: Some(shuffle_seed.is_some()),
                    context_uuid: uuid(),
                    autoplay_reset: autoplay.reset,
                    autoplay_loading: autoplay.loading,
                });
                MessageType::CtrlSrvrQueueLoadTracks
            }
            Self::InsertTracks {
                track_ids,
                after,
                shuffle_seed,
                autoplay,
            } => {
                message.ctrl_srvr_queue_insert_tracks = Some(CtrlSrvrQueueInsertTracks {
                    queue_version_ref: queue_version,
                    action_uuid,
                    track_ids,
                    insert_after: after,
                    shuffle_seed,
                    context_uuid: uuid(),
                    autoplay_reset: autoplay.reset,
                    autoplay_loading: autoplay.loading,
                });
                MessageType::CtrlSrvrQueueInsertTracks
            }
            Self::AddTracks {
                track_ids,
                shuffle_seed,
                autoplay,
            } => {
                message.ctrl_srvr_queue_add_tracks = Some(CtrlSrvrQueueAddTracks {
                    queue_version_ref: queue_version,
                    action_uuid,
                    track_ids,
                    shuffle_seed,
                    context_uuid: uuid(),
                    autoplay_reset: autoplay.reset,
                    autoplay_loading: autoplay.loading,
                });
                MessageType::CtrlSrvrQueueAddTracks
            }
            Self::RemoveTracks {
                queue_item_ids,
                autoplay,
            } => {
                message.ctrl_srvr_queue_remove_tracks = Some(CtrlSrvrQueueRemoveTracks {
                    queue_version_ref: queue_version,
                    action_uuid,
                    queue_item_ids,
                    autoplay_reset: autoplay.reset,
                    autoplay_loading: autoplay.loading,
                });
                MessageType::CtrlSrvrQueueRemoveTracks
            }
            Self::ReorderTracks {
                queue_item_ids,
                after,
                autoplay,
            } => {
                message.ctrl_srvr_queue_reorder_tracks = Some(CtrlSrvrQueueReorderTracks {
                    queue_version_ref: queue_version,
                    action_uuid,
                    queue_item_ids,
                    insert_after: after,
                    autoplay_reset: autoplay.reset,
                    autoplay_loading: autoplay.loading,
                });
                MessageType::CtrlSrvrQueueReorderTracks
            }
            Self::SetQueueState {
                tracks,
                shuffle_mode,
                shuffled_indexes,
                autoplay_mode,
                autoplay_loading,
                autoplay_tracks,
            } => {
                message.ctrl_srvr_set_queue_state = Some(CtrlSrvrSetQueueState {
                    queue_version_ref: queue_version,
                    action_uuid,
                    tracks,
                    shuffle_mode,
                    shuffled_track_indexes: shuffled_indexes,
                    autoplay_mode,
                    autoplay_loading,
                    autoplay_tracks,
                });
                MessageType::CtrlSrvrSetQueueState
            }
            Self::SetShuffleMode {
                enabled,
                seed,
                pivot_queue_item_id,
                autoplay,
            } => {
                message.ctrl_srvr_set_shuffle_mode = Some(CtrlSrvrSetShuffleMode {
                    queue_version_ref: queue_version,
                    action_uuid,
                    shuffle_mode: enabled,
                    shuffle_seed: seed,
                    shuffle_pivot_queue_item_id: pivot_queue_item_id,
                    autoplay_reset: autoplay.reset,
                    autoplay_loading: autoplay.loading,
                });
                MessageType::CtrlSrvrSetShuffleMode
            }
            Self::SetAutoplayMode { enabled, autoplay } => {
                message.ctrl_srvr_set_autoplay_mode = Some(CtrlSrvrSetAutoplayMode {
                    queue_version_ref: queue_version,
                    action_uuid,
                    autoplay_mode: enabled,
                    autoplay_reset: autoplay.reset,
                    autoplay_loading: autoplay.loading,
                });
                MessageType::CtrlSrvrSetAutoplayMode
            }
            Self::LoadAutoplayTracks(track_ids) => {
                message.ctrl_srvr_autoplay_load_tracks = Some(CtrlSrvrAutoplayLoadTracks {
                    queue_version_ref: queue_version,
                    action_uuid,
                    track_ids,
                    context_uuid: uuid(),
                });
                MessageType::CtrlSrvrAutoplayLoadTracks
            }
            Self::RemoveAutoplayTracks(queue_item_ids) => {
                message.ctrl_srvr_autoplay_remove_tracks = Some(CtrlSrvrAutoplayRemoveTracks {
                    queue_version_ref: queue_version,
                    action_uuid,
                    queue_item_ids,
                });
                MessageType::CtrlSrvrAutoplayRemoveTracks
            }
        };
        message.message_type = kind.into();
        message
    }
}

pub(crate) fn uuid() -> Vec<u8> {
    uuid::Uuid::new_v4().into_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shuffle_seed_turns_shuffle_on_for_a_load() {
        let command = ControllerCommand::LoadTracks {
            track_ids: vec![7],
            position: 0,
            shuffle_seed: Some(42),
            shuffle_pivot_index: Some(3),
            autoplay: Autoplay {
                reset: true,
                loading: true,
            },
        };
        assert!(command.changes_queue());
        let message = command.into_message(Some(QueueVersion { major: 1, minor: 2 }), &[1; 16]);
        assert_eq!(message.message_type(), MessageType::CtrlSrvrQueueLoadTracks);
        let load = message.ctrl_srvr_queue_load_tracks.unwrap_or_default();
        assert_eq!(
            load.queue_version_ref,
            Some(QueueVersion { major: 1, minor: 2 })
        );
        assert_eq!(load.action_uuid, vec![1; 16]);
        assert_eq!(load.context_uuid.len(), 16);
        assert_eq!(load.shuffle_mode, Some(true));
        assert_eq!(load.shuffle_pivot_index, Some(3));
        assert!(load.autoplay_reset && load.autoplay_loading);
    }

    #[test]
    fn renderer_commands_need_no_answer() {
        assert!(!ControllerCommand::SetLoopMode(LoopMode::Off).changes_queue());
        assert!(!ControllerCommand::SetActiveRenderer(1).changes_queue());
        assert!(ControllerCommand::ClearQueue.changes_queue());
    }
}
