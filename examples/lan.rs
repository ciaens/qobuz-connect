use qobuz_connect::proto::qconnect::{AudioQuality, DeviceType};
use qobuz_connect::{Device, Discovery, Event, Session};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let port = std::env::var("QOBUZ_CONNECT_PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or(0);
    let app_id = std::env::var("QOBUZ_APP_ID").unwrap_or_else(|_| "798273057".to_owned());
    let device = device();
    let (discovery, mut handovers) = match Discovery::start(&device, &app_id, port).await {
        Ok(started) => started,
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    };
    println!(
        "serving on port {}, pick the device in a Qobuz app",
        discovery.port()
    );
    let Some(mut handover) = handovers.recv().await else {
        return;
    };
    loop {
        println!("session {} handed over", handover.session_id);
        let mut session = match Session::join(handover.credentials, device.clone()).await {
            Ok(session) => session,
            Err(err) => {
                eprintln!("{err}");
                return;
            }
        };
        loop {
            tokio::select! {
                event = session.recv() => {
                    let Some(event) = event else { return };
                    if let Event::Session(state) = &event {
                        discovery.set_session(Some(&state.session_uuid));
                    }
                    println!("{event:?}");
                }
                next = handovers.recv() => {
                    let Some(next) = next else { return };
                    handover = next;
                    break;
                }
            }
        }
    }
}

fn device() -> Device {
    Device {
        uuid: *b"qobuz-connect/la",
        name: "fake renderer".to_owned(),
        brand: "qobuz-connect".to_owned(),
        model: "lan".to_owned(),
        kind: DeviceType::Speaker,
        max_audio_quality: AudioQuality::HiresLevel3,
        volume_remote_control: true,
        software_version: env!("CARGO_PKG_VERSION").to_owned(),
    }
}
