use std::hash::{DefaultHasher, Hash as _, Hasher as _};
use std::time::Duration;

use qobuz_connect::proto::qconnect::{AudioQuality, DeviceType, PlayingState};
use qobuz_connect::{
    Autoplay, ControllerCommand, Credentials, Device, Event, QueueEvent, RendererEvent, Session,
};

struct Controller {
    renderer: Option<i32>,
    items: Vec<i32>,
    step: usize,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let Some(credentials) = credentials() else {
        eprintln!("set QOBUZ_CONNECT_ENDPOINT and QOBUZ_CONNECT_JWT");
        return;
    };
    let tracks = tracks();
    if tracks.is_empty() {
        eprintln!("set QOBUZ_CONNECT_TRACKS to comma separated Qobuz track ids");
        return;
    }
    let wanted =
        std::env::var("QOBUZ_CONNECT_RENDERER").unwrap_or_else(|_| "fake renderer".to_owned());
    let mut session = match Session::join(credentials, device()).await {
        Ok(session) => session,
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    };
    let mut controller = Controller {
        renderer: None,
        items: Vec::new(),
        step: 0,
    };
    let mut ticker = tokio::time::interval(Duration::from_secs(5));
    loop {
        tokio::select! {
            event = session.recv() => {
                let Some(event) = event else { return };
                println!("{event:?}");
                if handle(&mut session, &mut controller, &wanted, &tracks, event).await.is_err() {
                    return;
                }
            }
            _ = ticker.tick(), if !controller.items.is_empty() => {
                let Some(command) = script(controller.step, &controller.items) else { return };
                controller.step = controller.step.saturating_add(1);
                println!("sending {command:?}");
                if session.control(command).await.is_err() {
                    return;
                }
            }
        }
    }
}

async fn handle(
    session: &mut Session,
    controller: &mut Controller,
    wanted: &str,
    tracks: &[u32],
    event: Event,
) -> Result<(), qobuz_connect::Error> {
    match event {
        Event::Renderer(RendererEvent::Added { id, device }) if device.name.contains(wanted) => {
            println!("controlling {} as renderer {id}", device.name);
            controller.renderer = Some(id);
            session
                .control(ControllerCommand::SetActiveRenderer(id))
                .await
                .map(|_| ())
        }
        Event::Renderer(RendererEvent::ActiveChanged { id })
            if id.is_some() && id == controller.renderer && controller.items.is_empty() =>
        {
            session
                .control(ControllerCommand::LoadTracks {
                    track_ids: tracks.to_vec(),
                    position: 0,
                    shuffle_seed: None,
                    shuffle_pivot_index: None,
                    autoplay: Autoplay::default(),
                })
                .await
                .map(|_| ())
        }
        Event::Queue(QueueEvent::Loaded(loaded)) => {
            controller.items = loaded
                .tracks
                .iter()
                .map(|track| track.queue_item_id)
                .collect();
            Ok(())
        }
        _ => Ok(()),
    }
}

fn script(step: usize, items: &[i32]) -> Option<ControllerCommand> {
    let first = items.first().copied()?;
    let second = items.get(1).copied().unwrap_or(first);
    let state = |playing, position, queue_item_id| ControllerCommand::SetPlayerState {
        playing,
        position,
        queue_item_id,
    };
    Some(match step {
        0 => state(
            Some(PlayingState::Playing),
            Some(Duration::ZERO),
            Some(first),
        ),
        1 | 5 => state(Some(PlayingState::Paused), None, None),
        2 => state(
            Some(PlayingState::Playing),
            Some(Duration::from_secs(60)),
            None,
        ),
        3 => state(None, Some(Duration::from_secs(90)), None),
        4 => state(
            Some(PlayingState::Playing),
            Some(Duration::ZERO),
            Some(second),
        ),
        _ => return None,
    })
}

fn credentials() -> Option<Credentials> {
    Some(Credentials {
        endpoint: std::env::var("QOBUZ_CONNECT_ENDPOINT").ok()?,
        jwt: std::env::var("QOBUZ_CONNECT_JWT").ok()?,
    })
}

fn tracks() -> Vec<u32> {
    std::env::var("QOBUZ_CONNECT_TRACKS")
        .map(|ids| {
            ids.split(',')
                .filter_map(|id| id.trim().parse().ok())
                .collect()
        })
        .unwrap_or_default()
}

fn device() -> Device {
    let name = "qobuz-connect controller";
    Device {
        uuid: device_uuid(name),
        name: name.to_owned(),
        brand: "qobuz-connect".to_owned(),
        model: "controller example".to_owned(),
        kind: DeviceType::Computer,
        max_audio_quality: AudioQuality::HiresLevel3,
        volume_remote_control: false,
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
