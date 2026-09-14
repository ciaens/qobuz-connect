# Schema differences: play.qobuz.com bundle versus qonductor 0.1.0-alpha.5

Bundle line numbers refer to `web-bundle.js` as dumped on 2026-09-14. Field labels: `optional` means explicit presence in the bundle (ts-proto emits `void 0 !==`), blank means a proto3 field without presence (encoded only when non-default).

## Message type enum

- 2: bundle `MESSAGE_TYPE_PLAYBACK_ERROR`, qonductor `-`
- 3: bundle `MESSAGE_TYPE_AUTHENTICATE`, qonductor `-`
- 47: bundle `MESSAGE_TYPE_SRVR_RNDR_MUTE_VOLUME`, qonductor `MESSAGE_TYPE_SRVR_RNDR_SET_AUTOPLAY_MODE`
- 79: bundle `MESSAGE_TYPE_CTRL_SRVR_AUTOPLAY_LOAD_TRACKS`, qonductor `MESSAGE_TYPE_CTRL_SRVR_AUTOPLAY_ADD_TRACKS`
- 105: bundle `MESSAGE_TYPE_SRVR_CTRL_QUEUE_TRACKS_ADDED_FROM_AUTOPLAY`, qonductor `MESSAGE_TYPE_SRVR_CTRL_QUEUE_VERSION_CHANGED`

## Other enums

- `AudioQuality`: absent in qonductor (bundle members: AUDIO_QUALITY_UNKNOWN=0, AUDIO_QUALITY_MP3=1, AUDIO_QUALITY_CD=2, AUDIO_QUALITY_HIRES_LEVEL1=3, AUDIO_QUALITY_HIRES_LEVEL2=4, AUDIO_QUALITY_HIRES_LEVEL3=5)
- `DeviceType` 2: bundle `DEVICE_TYPE_STREAMER`, qonductor `DEVICE_TYPE_SPEAKERBOX`
- `DeviceType` 4: bundle `DEVICE_TYPE_SOUNDBAR`, qonductor `DEVICE_TYPE_SPEAKERBOX2`
- `DeviceType` 5: bundle `DEVICE_TYPE_COMPUTER`, qonductor `DEVICE_TYPE_LAPTOP`
- `DeviceType` 6: bundle `DEVICE_TYPE_MOBILE`, qonductor `DEVICE_TYPE_PHONE`
- `DeviceType` 7: bundle `DEVICE_TYPE_CAST`, qonductor `DEVICE_TYPE_GOOGLE_CAST`
- `VolumeRemoteControl`: absent in qonductor (bundle members: VOLUME_REMOTE_CONTROL_UNKNOWN=0, VOLUME_REMOTE_CONTROL_NOT_ALLOWED=1, VOLUME_REMOTE_CONTROL_ALLOWED=2)
- `NetworkType`: absent in qonductor (bundle members: NETWORK_TYPE_UNKNOWN=0, NETWORK_TYPE_WIFI=1, NETWORK_TYPE_CELLULAR=2)
- `ErrorType`: absent in qonductor (bundle members: ERROR_TYPE_UNKNOWN=0, ERROR_TYPE_TRACK_NOT_FOUND=1, ERROR_TYPE_TRACK_NOT_STREAMABLE=2, ERROR_TYPE_TRACK_MUSIC_DATA_INVALID=3, ERROR_TYPE_SERVICE_ERROR=4, ERROR_TYPE_NETWORK_ERROR=5, ERROR_TYPE_OTHER_ERRORS=100)
- `RendererStatus`: absent in qonductor (bundle members: RENDERER_STATUS_UNKNOWN=0, RENDERER_STATUS_ACTIVE_CONNECTED=1, RENDERER_STATUS_ACTIVE_DISCONNECTED=2, RENDERER_STATUS_INACTIVE=3)
- `JoinSessionReason`: absent in qonductor (bundle members: JOIN_SESSION_REASON_UNKNOWN=0, JOIN_SESSION_REASON_CONTROLLER_REQUEST=1, JOIN_SESSION_REASON_RECONNECTION=2)
- `ActionType`: absent in qonductor (bundle members: ACTION_TYPE_UNKNOWN=0, ACTION_TYPE_PREVIOUS=1, ACTION_TYPE_NEXT=2, ACTION_TYPE_REPEAT_OFF=3, ACTION_TYPE_REPEAT_ONE=4, ACTION_TYPE_REPEAT_ALL=5, ACTION_TYPE_SHUFFLE_OFF=6, ACTION_TYPE_SHUFFLE_ON=7, ACTION_TYPE_SEEK=8)
- `MessageType`: absent in qonductor (bundle members: MESSAGE_TYPE_UNKNOWN=0, MESSAGE_TYPE_ERROR=1, MESSAGE_TYPE_PLAYBACK_ERROR=2, MESSAGE_TYPE_AUTHENTICATE=3, MESSAGE_TYPE_RNDR_SRVR_JOIN_SESSION=21, MESSAGE_TYPE_RNDR_SRVR_DEVICE_INFO_UPDATED=22, MESSAGE_TYPE_RNDR_SRVR_STATE_UPDATED=23, MESSAGE_TYPE_RNDR_SRVR_RENDERER_ACTION=24, MESSAGE_TYPE_RNDR_SRVR_VOLUME_CHANGED=25, MESSAGE_TYPE_RNDR_SRVR_FILE_AUDIO_QUALITY_CHANGED=26, MESSAGE_TYPE_RNDR_SRVR_DEVICE_AUDIO_QUALITY_CHANGED=27, MESSAGE_TYPE_RNDR_SRVR_MAX_AUDIO_QUALITY_CHANGED=28, MESSAGE_TYPE_RNDR_SRVR_VOLUME_MUTED=29, MESSAGE_TYPE_SRVR_RNDR_SET_STATE=41, MESSAGE_TYPE_SRVR_RNDR_SET_VOLUME=42, MESSAGE_TYPE_SRVR_RNDR_SET_ACTIVE=43, MESSAGE_TYPE_SRVR_RNDR_SET_MAX_AUDIO_QUALITY=44, MESSAGE_TYPE_SRVR_RNDR_SET_LOOP_MODE=45, MESSAGE_TYPE_SRVR_RNDR_SET_SHUFFLE_MODE=46, MESSAGE_TYPE_SRVR_RNDR_MUTE_VOLUME=47, MESSAGE_TYPE_CTRL_SRVR_JOIN_SESSION=61, MESSAGE_TYPE_CTRL_SRVR_SET_PLAYER_STATE=62, MESSAGE_TYPE_CTRL_SRVR_SET_ACTIVE_RENDERER=63, MESSAGE_TYPE_CTRL_SRVR_SET_VOLUME=64, MESSAGE_TYPE_CTRL_SRVR_CLEAR_QUEUE=65, MESSAGE_TYPE_CTRL_SRVR_QUEUE_LOAD_TRACKS=66, MESSAGE_TYPE_CTRL_SRVR_QUEUE_INSERT_TRACKS=67, MESSAGE_TYPE_CTRL_SRVR_QUEUE_ADD_TRACKS=68, MESSAGE_TYPE_CTRL_SRVR_QUEUE_REMOVE_TRACKS=69, MESSAGE_TYPE_CTRL_SRVR_QUEUE_REORDER_TRACKS=70, MESSAGE_TYPE_CTRL_SRVR_SET_SHUFFLE_MODE=71, MESSAGE_TYPE_CTRL_SRVR_SET_LOOP_MODE=72, MESSAGE_TYPE_CTRL_SRVR_MUTE_VOLUME=73, MESSAGE_TYPE_CTRL_SRVR_SET_MAX_AUDIO_QUALITY=74, MESSAGE_TYPE_CTRL_SRVR_SET_QUEUE_STATE=75, MESSAGE_TYPE_CTRL_SRVR_ASK_FOR_QUEUE_STATE=76, MESSAGE_TYPE_CTRL_SRVR_ASK_FOR_RENDERER_STATE=77, MESSAGE_TYPE_CTRL_SRVR_SET_AUTOPLAY_MODE=78, MESSAGE_TYPE_CTRL_SRVR_AUTOPLAY_LOAD_TRACKS=79, MESSAGE_TYPE_CTRL_SRVR_AUTOPLAY_REMOVE_TRACKS=80, MESSAGE_TYPE_SRVR_CTRL_SESSION_STATE=81, MESSAGE_TYPE_SRVR_CTRL_RENDERER_STATE_UPDATED=82, MESSAGE_TYPE_SRVR_CTRL_ADD_RENDERER=83, MESSAGE_TYPE_SRVR_CTRL_UPDATE_RENDERER=84, MESSAGE_TYPE_SRVR_CTRL_REMOVE_RENDERER=85, MESSAGE_TYPE_SRVR_CTRL_ACTIVE_RENDERER_CHANGED=86, MESSAGE_TYPE_SRVR_CTRL_VOLUME_CHANGED=87, MESSAGE_TYPE_SRVR_CTRL_QUEUE_ERROR_MESSAGE=88, MESSAGE_TYPE_SRVR_CTRL_QUEUE_CLEARED=89, MESSAGE_TYPE_SRVR_CTRL_QUEUE_STATE=90, MESSAGE_TYPE_SRVR_CTRL_QUEUE_TRACKS_LOADED=91, MESSAGE_TYPE_SRVR_CTRL_QUEUE_TRACKS_INSERTED=92, MESSAGE_TYPE_SRVR_CTRL_QUEUE_TRACKS_ADDED=93, MESSAGE_TYPE_SRVR_CTRL_QUEUE_TRACKS_REMOVED=94, MESSAGE_TYPE_SRVR_CTRL_QUEUE_TRACKS_REORDERED=95, MESSAGE_TYPE_SRVR_CTRL_SHUFFLE_MODE_SET=96, MESSAGE_TYPE_SRVR_CTRL_LOOP_MODE_SET=97, MESSAGE_TYPE_SRVR_CTRL_VOLUME_MUTED=98, MESSAGE_TYPE_SRVR_CTRL_MAX_AUDIO_QUALITY_CHANGED=99, MESSAGE_TYPE_SRVR_CTRL_FILE_AUDIO_QUALITY_CHANGED=100, MESSAGE_TYPE_SRVR_CTRL_DEVICE_AUDIO_QUALITY_CHANGED=101, MESSAGE_TYPE_SRVR_CTRL_AUTOPLAY_MODE_SET=102, MESSAGE_TYPE_SRVR_CTRL_AUTOPLAY_TRACKS_LOADED=103, MESSAGE_TYPE_SRVR_CTRL_AUTOPLAY_TRACKS_REMOVED=104, MESSAGE_TYPE_SRVR_CTRL_QUEUE_TRACKS_ADDED_FROM_AUTOPLAY=105)

## Messages only in the bundle

- `SrvrRndrMuteVolume` (bundle line 282618)
- `SrvrCtrlQueueTracksLoaded` (bundle line 280786)
- `SrvrCtrlAutoplayTracksRemoved` (bundle line 281725)
- `SrvrCtrlQueueTracksAddedFromAutoplay` (bundle line 281249)
- `TrackRef` (bundle line 279064)
- `QueueTrack` (bundle line 277944)

## Messages only in qonductor

- `SrvrCtrlQueueLoadTracks` (qconnect_queue.proto)
- `SrvrRndrSetAutoplayMode` (qconnect_payload.proto)
- `SrvrCtrlQueueVersionChanged` (qconnect_payload.proto)

## Field differences

- `QConnectMessage` (bundle line 283281)
  - 24: only in bundle: `optional RndrSrvrRendererAction rndr_srvr_renderer_action`
  - 30: only in qonductor: `optional RndrSrvrRendererAction rndr_srvr_renderer_action`
  - 47 `srvr_rndr_mute_volume`: name `srvr_rndr_mute_volume` vs `srvr_rndr_set_autoplay_mode`; type `SrvrRndrMuteVolume` vs `SrvrRndrSetAutoplayMode`
  - 91 `srvr_ctrl_queue_tracks_loaded`: type `SrvrCtrlQueueTracksLoaded` vs `SrvrCtrlQueueLoadTracks`
  - 104 `srvr_ctrl_autoplay_tracks_removed`: type `SrvrCtrlAutoplayTracksRemoved` vs `SrvrCtrlQueueTracksRemoved`
  - 105 `srvr_ctrl_queue_tracks_added_from_autoplay`: name `srvr_ctrl_queue_tracks_added_from_autoplay` vs `srvr_ctrl_queue_version_changed`; type `SrvrCtrlQueueTracksAddedFromAutoplay` vs `SrvrCtrlQueueVersionChanged`
- `RndrSrvrJoinSession` (bundle line 281956)
  - 4 `initial_state`: type `QueueRendererState` vs `RendererState`
- `RndrSrvrRendererAction` (bundle line 282134)
  - 1 `seek_position`: name `seek_position` vs `action`; type `uint32` vs `int32`
  - 2: only in bundle: ` ActionType action`
- `RndrSrvrFileAudioQualityChanged` (bundle line 282344)
  - 1 `sampling_rate`: name `sampling_rate` vs `value`; type `uint32` vs `int32`
  - 2: only in bundle: ` uint32 bit_depth`
  - 3: only in bundle: ` uint32 nb_channels`
  - 4: only in bundle: ` AudioQuality audio_quality`
- `RndrSrvrDeviceAudioQualityChanged` (bundle line 282418)
  - 1 `sampling_rate`: name `sampling_rate` vs `value`; type `uint32` vs `int32`
  - 2: only in bundle: ` uint32 bit_depth`
  - 3: only in bundle: ` uint32 nb_channels`
- `RndrSrvrMaxAudioQualityChanged` (bundle line 282292)
  - 1 `max_audio_quality`: name `max_audio_quality` vs `value`
  - 2: only in bundle: `optional NetworkType network_type`
- `SrvrRndrSetState` (bundle line 282481)
  - 4 `current_track`: name `current_track` vs `current_queue_item`
  - 5 `next_track`: name `next_track` vs `next_queue_item`
- `SrvrRndrSetLoopMode` (bundle line 282700)
  - 1 `loop_mode`: name `loop_mode` vs `mode`
- `SrvrRndrSetShuffleMode` (bundle line 282741)
  - 1 `shuffle_mode`: name `shuffle_mode` vs `shuffle_on`
- `CtrlSrvrClearQueue` (bundle line 279116)
  - 1 `queue_version_ref`: name `queue_version_ref` vs `queue_version`
  - 2: only in bundle: ` bytes action_uuid`
- `CtrlSrvrQueueLoadTracks` (bundle line 279176)
  - 1 `queue_version_ref`: name `queue_version_ref` vs `queue_version`
  - 3 `track_ids`: name `track_ids` vs `session_uuid`; type `fixed32` vs `bytes`; label `repeated` vs `optional`
  - 5 `shuffle_seed`: name `shuffle_seed` vs `qobuz_reference_uuid`
  - 6 `shuffle_pivot_index`: name `shuffle_pivot_index` vs `shuffle_pivot_queue_item_id`
  - 100: only in qonductor: `optional bytes queue_hash`
- `CtrlSrvrQueueInsertTracks` (bundle line 279318)
  - 1 `queue_version_ref`: name `queue_version_ref` vs `queue_version`
  - 3 `track_ids`: name `track_ids` vs `tracks`; type `fixed32` vs `QueueTrackRef`
  - 100: only in qonductor: `optional bytes queue_hash`
- `CtrlSrvrQueueAddTracks` (bundle line 279441)
  - 1 `queue_version_ref`: name `queue_version_ref` vs `queue_version`
  - 3 `track_ids`: name `track_ids` vs `tracks`; type `fixed32` vs `QueueTrackRef`
  - 100: only in qonductor: `optional bytes queue_hash`
- `CtrlSrvrQueueRemoveTracks` (bundle line 279553)
  - 1 `queue_version_ref`: name `queue_version_ref` vs `queue_version`
  - 100: only in qonductor: `optional bytes queue_hash`
- `CtrlSrvrQueueReorderTracks` (bundle line 279648)
  - 1 `queue_version_ref`: name `queue_version_ref` vs `queue_version`
  - 100: only in qonductor: `optional bytes queue_hash`
- `CtrlSrvrSetShuffleMode` (bundle line 279753)
  - 1 `queue_version_ref`: name `queue_version_ref` vs `queue_version`
  - 3 `shuffle_mode`: name `shuffle_mode` vs `shuffle_on`
  - 4 `shuffle_seed`: name `shuffle_seed` vs `current_queue_item_id`
  - 5 `shuffle_pivot_queue_item_id`: name `shuffle_pivot_queue_item_id` vs `shuffle_pivot`; type `int32` vs `uint32`
  - 6 `autoplay_reset`: name `autoplay_reset` vs `autoplay_mode`
  - 100: only in qonductor: `optional bytes queue_hash`
- `CtrlSrvrSetLoopMode` (bundle line 278442)
  - 1 `loop_mode`: name `loop_mode` vs `mode`
- `CtrlSrvrSetMaxAudioQuality` (bundle line 278565)
  - 1 `renderer_id`: name `renderer_id` vs `max_audio_quality`
  - 2: only in bundle: ` AudioQuality max_audio_quality`
- `CtrlSrvrSetQueueState` (bundle line 278944)
  - 1: only in bundle: `optional QueueVersion queue_version_ref`
  - 2: only in bundle: ` bytes action_uuid`
  - 3: only in bundle: `repeated TrackRef tracks`
  - 4: only in bundle: ` bool shuffle_mode`
  - 5: only in bundle: `repeated uint32 shuffled_track_indexes`
  - 6: only in bundle: ` bool autoplay_mode`
  - 7: only in bundle: ` bool autoplay_loading`
  - 8: only in bundle: `repeated TrackRef autoplay_tracks`
- `CtrlSrvrAskForQueueState` (bundle line 278886)
  - 1 `queue_version_ref`: name `queue_version_ref` vs `queue_version`
  - 2 `action_uuid`: name `action_uuid` vs `queue_uuid`
- `CtrlSrvrAskForRendererState` (bundle line 278839)
  - 1 `renderer_id`: name `renderer_id` vs `session_id`; type `int32` vs `uint64`
- `CtrlSrvrSetAutoplayMode` (bundle line 278492)
  - 1 `queue_version_ref`: name `queue_version_ref` vs `autoplay_on`; type `QueueVersion` vs `bool`
  - 2: only in bundle: ` bytes action_uuid`
  - 3: only in bundle: ` bool autoplay_mode`
  - 4: only in bundle: ` bool autoplay_reset`
  - 5: only in bundle: ` bool autoplay_loading`
- `CtrlSrvrAutoplayLoadTracks` (bundle line 279852)
  - 1 `queue_version_ref`: name `queue_version_ref` vs `queue_version`
  - 5: only in qonductor: `optional bool autoplay_loading`
  - 6: only in qonductor: `optional bool autoplay_reset`
  - 7: only in qonductor: `optional uint32 shuffle_seed`
  - 8: only in qonductor: `optional uint32 insert_after`
  - 9: only in qonductor: `optional bool prepend`
  - 10: only in qonductor: `optional bool append`
  - 100: only in qonductor: `optional bytes queue_hash`
- `CtrlSrvrAutoplayRemoveTracks` (bundle line 279935)
  - 1: only in bundle: `optional QueueVersion queue_version_ref`
  - 2: only in bundle: ` bytes action_uuid`
  - 3: only in bundle: `repeated int32 queue_item_ids`
- `SrvrCtrlSessionState` (bundle line 280010)
  - 2 `active_renderer_id`: name `active_renderer_id` vs `session_id`; type `int32` vs `uint64`
  - 4 `playing_state`: name `playing_state` vs `track_index`; type `PlayingState` vs `uint32`
  - 5 `loop_mode`: name `loop_mode` vs `unknown`; type `LoopMode` vs `bool`
- `SrvrCtrlRendererStateUpdated` (bundle line 280083)
  - 1 `renderer_id`: type `int32` vs `uint64`
  - 2 `status`: name `status` vs `message_id`; type `RendererStatus` vs `uint64`
  - 3 `player_state`: name `player_state` vs `state`
- `SrvrCtrlAddRenderer` (bundle line 280245)
  - 1 `renderer_id`: type `int32` vs `uint64`
  - 2 `device_info`: name `device_info` vs `renderer`
- `SrvrCtrlUpdateRenderer` (bundle line 280297)
  - 1 `renderer_id`: name `renderer_id` vs `renderer`; type `int32` vs `DeviceInfo`
  - 2: only in bundle: `optional DeviceInfo device_info`
- `SrvrCtrlRemoveRenderer` (bundle line 280349)
  - 1 `renderer_id`: type `int32` vs `uint64`
- `SrvrCtrlActiveRendererChanged` (bundle line 280390)
  - 1 `active_renderer_id`: name `active_renderer_id` vs `renderer_id`; type `int32` vs `uint64`
- `SrvrCtrlVolumeChanged` (bundle line 280431)
  - 1 `renderer_id`: type `int32` vs `uint64`
- `SrvrCtrlQueueTracksInserted` (bundle line 281337)
  - 3 `tracks`: type `QueueTrack` vs `QueueTrackRef`
- `SrvrCtrlQueueTracksAdded` (bundle line 281146)
  - 3 `tracks`: type `QueueTrack` vs `QueueTrackRef`
- `SrvrCtrlQueueTracksRemoved` (bundle line 281040)
  - 3 `queue_item_ids`: type `int32` vs `uint32`
- `SrvrCtrlQueueTracksReordered` (bundle line 280927)
  - 3 `queue_item_ids`: type `int32` vs `uint32`
  - 4 `insert_after`: type `int32` vs `uint32`
- `SrvrCtrlShuffleModeSet` (bundle line 281461)
  - 3 `shuffle_mode`: name `shuffle_mode` vs `shuffle_on`
  - 4 `shuffle_seed`: name `shuffle_seed` vs `current_queue_item_id`
  - 5 `shuffle_pivot_queue_item_id`: name `shuffle_pivot_queue_item_id` vs `shuffle_pivot`; type `int32` vs `uint32`
  - 6 `autoplay_reset`: name `autoplay_reset` vs `autoplay_mode`
- `SrvrCtrlLoopModeSet` (bundle line 280535)
  - 1 `loop_mode`: name `loop_mode` vs `mode`
- `SrvrCtrlVolumeMuted` (bundle line 280483)
  - 1 `renderer_id`: type `int32` vs `uint64`
- `SrvrCtrlMaxAudioQualityChanged` (bundle line 278617)
  - 1 `renderer_id`: name `renderer_id` vs `max_audio_quality`
  - 2: only in bundle: ` AudioQuality max_audio_quality`
  - 3: only in bundle: `optional NetworkType network_type`
- `SrvrCtrlFileAudioQualityChanged` (bundle line 278680)
  - 1 `renderer_id`: name `renderer_id` vs `file_audio_quality`
  - 2: only in bundle: ` uint32 sampling_rate`
  - 3: only in bundle: ` uint32 bit_depth`
  - 4: only in bundle: ` uint32 nb_channels`
  - 5: only in bundle: ` AudioQuality audio_quality`
- `SrvrCtrlDeviceAudioQualityChanged` (bundle line 278765)
  - 1 `renderer_id`: name `renderer_id` vs `device_audio_quality`
  - 2: only in bundle: ` uint32 sampling_rate`
  - 3: only in bundle: ` uint32 bit_depth`
  - 4: only in bundle: ` uint32 nb_channels`
- `SrvrCtrlAutoplayModeSet` (bundle line 281570)
  - 1 `queue_version`: name `queue_version` vs `autoplay_on`; type `QueueVersion` vs `bool`
  - 2: only in bundle: ` bytes action_uuid`
  - 3: only in bundle: ` bool autoplay_mode`
  - 4: only in bundle: ` bool autoplay_reset`
  - 5: only in bundle: ` bool autoplay_loading`
- `SrvrCtrlAutoplayTracksLoaded` (bundle line 281651)
  - 3 `tracks`: type `QueueTrack` vs `QueueTrackRef`
- `RendererState` (bundle line 280160)
  - 5 `current_queue_item_id`: name `current_queue_item_id` vs `current_queue_index`; type `int32` vs `uint32`
  - 7: only in qonductor: `optional int32 next_queue_item_id`
- `QueueVersion` (bundle line 277892)
  - 1 `major`: type `uint32` vs `uint64`
  - 2 `minor`: type `uint32` vs `int32`
- `QueueTrackRef` (bundle line 277996)
  - 1 `queue_item_id`: type `int32` vs `uint64`
- `QueueItemRef` (bundle line 278234)
  - 2 `id`: type `int32` vs `uint32`

## Presence semantics

qonductor declares every field `optional` in proto2. The bundle is proto3: only the fields listed as `optional` in the generated proto files carry explicit presence, the rest are encoded only when non-default. Wire compatible for decoding; encoding differs only in that zero values are omitted.
