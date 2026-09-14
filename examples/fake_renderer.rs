use std::hash::{DefaultHasher, Hash as _, Hasher as _};
use std::time::Duration;

use qobuz_connect::proto::qconnect::{
    AudioQuality, BufferState, DeviceType, NetworkType, PlayingState,
};
use qobuz_connect::{
    Credentials, Device, Event, PlayerState, RendererCommand, RendererReport, Session,
};

struct Fake {
    state: PlayerState,
    volume: u32,
    muted: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let Some(credentials) = credentials() else {
        eprintln!("set QOBUZ_CONNECT_ENDPOINT and QOBUZ_CONNECT_JWT");
        return;
    };
    let name = std::env::var("QOBUZ_CONNECT_NAME")
        .unwrap_or_else(|_| "qobuz-connect fake renderer".to_owned());
    let activate = std::env::var_os("QOBUZ_CONNECT_ACTIVATE").is_some();
    let mut session = match Session::join(credentials, device(&name)).await {
        Ok(session) => session,
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    };
    let mut fake = Fake {
        state: PlayerState {
            playing: PlayingState::Stopped,
            buffer: BufferState::Ok,
            position: Duration::ZERO,
            duration: Duration::from_secs(240),
            current_queue_item_id: None,
            next_queue_item_id: None,
        },
        volume: 100,
        muted: false,
    };
    let mut ticker = tokio::time::interval(Duration::from_secs(1));
    loop {
        tokio::select! {
            event = session.recv() => {
                let Some(event) = event else { return };
                println!("{event:?}");
                if handle(&mut session, &mut fake, event, activate).await.is_err() {
                    return;
                }
            }
            _ = ticker.tick() => {
                if fake.state.playing == PlayingState::Playing {
                    fake.state.position = fake.state.position.saturating_add(Duration::from_secs(1));
                    if session.report(RendererReport::State(fake.state.clone())).await.is_err() {
                        return;
                    }
                }
            }
        }
    }
}

async fn handle(
    session: &mut Session,
    fake: &mut Fake,
    event: Event,
    activate: bool,
) -> Result<(), qobuz_connect::Error> {
    match event {
        Event::Registered { .. } if activate => session.activate().await,
        Event::Command(RendererCommand::SetState {
            playing,
            position,
            current,
            next,
        }) => {
            apply(&mut fake.state, playing, position, current);
            if let Some(next) = next {
                fake.state.next_queue_item_id = Some(next.queue_item_id);
            }
            session
                .report(RendererReport::State(fake.state.clone()))
                .await
        }
        Event::Command(RendererCommand::SetVolume(volume)) => {
            fake.volume = volume.min(100);
            session.report(RendererReport::Volume(fake.volume)).await
        }
        Event::Command(RendererCommand::ChangeVolume(delta)) => {
            fake.volume = fake.volume.saturating_add_signed(delta).min(100);
            session.report(RendererReport::Volume(fake.volume)).await
        }
        Event::Command(RendererCommand::Mute(muted)) => {
            fake.muted = muted;
            session.report(RendererReport::Muted(muted)).await
        }
        Event::Command(RendererCommand::SetActive(true)) => {
            session.report(RendererReport::Volume(fake.volume)).await?;
            session.report(RendererReport::Muted(fake.muted)).await?;
            session
                .report(RendererReport::MaxAudioQuality {
                    quality: AudioQuality::HiresLevel3,
                    network: NetworkType::Wifi,
                })
                .await
        }
        Event::Command(RendererCommand::SetActive(false)) => {
            fake.state.playing = PlayingState::Stopped;
            session
                .report(RendererReport::State(fake.state.clone()))
                .await
        }
        _ => Ok(()),
    }
}

fn apply(
    state: &mut PlayerState,
    playing: Option<PlayingState>,
    position: Option<Duration>,
    current: Option<qobuz_connect::proto::qconnect::QueueTrackRef>,
) {
    match current {
        Some(track) if track.queue_item_id < 0 => {
            state.playing = PlayingState::Stopped;
            state.current_queue_item_id = None;
        }
        Some(track) if state.current_queue_item_id != Some(track.queue_item_id) => {
            state.current_queue_item_id = Some(track.queue_item_id);
            state.position = position.unwrap_or(Duration::ZERO);
            state.playing = playing.unwrap_or(PlayingState::Playing);
        }
        _ => {
            if let Some(position) = position {
                state.position = position;
            }
            if let Some(playing) = playing {
                state.playing = playing;
            }
        }
    }
}

fn credentials() -> Option<Credentials> {
    Some(Credentials {
        endpoint: std::env::var("QOBUZ_CONNECT_ENDPOINT").ok()?,
        jwt: std::env::var("QOBUZ_CONNECT_JWT").ok()?,
    })
}

fn device(name: &str) -> Device {
    Device {
        uuid: device_uuid(name),
        name: name.to_owned(),
        brand: "qobuz-connect".to_owned(),
        model: "fake renderer".to_owned(),
        kind: DeviceType::Speaker,
        max_audio_quality: AudioQuality::HiresLevel3,
        volume_remote_control: true,
        software_version: env!("CARGO_PKG_VERSION").to_owned(),
    }
}

fn device_uuid(name: &str) -> [u8; 16] {
    let mut uuid = [0; 16];
    for (salt, half) in uuid.chunks_exact_mut(8).enumerate() {
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        salt.hash(&mut hasher);
        half.copy_from_slice(&hasher.finish().to_be_bytes());
    }
    uuid
}
