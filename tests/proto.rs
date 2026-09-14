use prost::Message;
use qobuz_connect::proto::qconnect::{
    ActionType, MessageType, PlayingState, QConnectBatch, QConnectMessage, QueueTrackRef,
    QueueVersion, RndrSrvrRendererAction, SrvrCtrlQueueTracksAddedFromAutoplay, SrvrRndrMuteVolume,
    SrvrRndrSetState,
};

fn length_delimited_tag(field: u32) -> Vec<u8> {
    let mut tag = Vec::new();
    prost::encoding::encode_varint(u64::from(field << 3 | 2), &mut tag);
    tag
}

fn contains_tag(bytes: &[u8], field: u32) -> bool {
    let tag = length_delimited_tag(field);
    bytes.windows(tag.len()).any(|window| window == tag)
}

#[test]
fn envelope_tags_follow_the_web_player() {
    let mute = QConnectMessage {
        message_type: MessageType::SrvrRndrMuteVolume.into(),
        srvr_rndr_mute_volume: Some(SrvrRndrMuteVolume { value: true }),
        ..Default::default()
    };
    assert!(contains_tag(&mute.encode_to_vec(), 47));

    let autoplay = QConnectMessage {
        message_type: MessageType::SrvrCtrlQueueTracksAddedFromAutoplay.into(),
        srvr_ctrl_queue_tracks_added_from_autoplay: Some(SrvrCtrlQueueTracksAddedFromAutoplay {
            queue_item_ids: vec![9],
            ..Default::default()
        }),
        ..Default::default()
    };
    assert!(contains_tag(&autoplay.encode_to_vec(), 105));

    let action = QConnectMessage {
        message_type: MessageType::RndrSrvrRendererAction.into(),
        rndr_srvr_renderer_action: Some(RndrSrvrRendererAction {
            seek_position: Some(1500),
            action: ActionType::Seek.into(),
        }),
        ..Default::default()
    };
    assert!(contains_tag(&action.encode_to_vec(), 24));
}

#[test]
fn batch_round_trips() {
    let state = SrvrRndrSetState {
        playing_state: Some(PlayingState::Playing.into()),
        current_position: Some(42),
        queue_version: Some(QueueVersion { major: 3, minor: 2 }),
        current_track: Some(QueueTrackRef {
            queue_item_id: 5,
            track_id: 388_712_168,
            context_uuid: None,
        }),
        next_track: None,
    };
    let batch = QConnectBatch {
        messages_time: 1_700_000_000_000,
        messages_id: 3,
        messages: vec![QConnectMessage {
            message_type: MessageType::SrvrRndrSetState.into(),
            srvr_rndr_set_state: Some(state.clone()),
            ..Default::default()
        }],
    };

    let decoded = QConnectBatch::decode(batch.encode_to_vec().as_slice()).ok();

    assert_eq!(decoded.as_ref(), Some(&batch));
    let decoded_state = decoded
        .as_ref()
        .and_then(|b| b.messages.first())
        .and_then(|m| m.srvr_rndr_set_state.as_ref());
    assert_eq!(decoded_state, Some(&state));
    assert_eq!(
        decoded_state.map(SrvrRndrSetState::playing_state),
        Some(PlayingState::Playing)
    );
}
