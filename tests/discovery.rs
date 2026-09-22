#![cfg(feature = "discovery")]
#![allow(clippy::unwrap_used, clippy::panic, clippy::indexing_slicing)]

use qobuz_connect::proto::qconnect::{AudioQuality, DeviceType};
use qobuz_connect::{Device, Discovery};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpStream;

fn device() -> Device {
    Device {
        uuid: [7; 16],
        name: "test renderer".to_owned(),
        brand: "qobuz-connect".to_owned(),
        model: "test".to_owned(),
        kind: DeviceType::Speaker,
        max_audio_quality: AudioQuality::HiresLevel1,
        volume_remote_control: true,
        software_version: "0".to_owned(),
    }
}

async fn call(port: u16, request: String) -> (String, serde_json::Value) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.unwrap();
    let response = String::from_utf8(response).unwrap();
    let (head, body) = response.split_once("\r\n\r\n").unwrap();
    let status = head.lines().next().unwrap().to_owned();
    let body = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    (status, body)
}

fn get(route: &str) -> String {
    format!("GET /devices/0707/{route} HTTP/1.1\r\nHost: device\r\nConnection: close\r\n\r\n")
}

#[tokio::test]
async fn describes_the_device_and_takes_a_handover() {
    let (discovery, mut handovers) = Discovery::start(&device(), "123", 0).await.unwrap();
    let port = discovery.port();

    let (status, info) = call(port, get("get-display-info")).await;
    assert_eq!(status, "HTTP/1.1 200 OK");
    assert_eq!(info["type"], "SPEAKER");
    assert_eq!(info["friendly_name"], "test renderer");
    assert_eq!(
        info["serial_number"],
        "07070707-0707-0707-0707-070707070707"
    );
    assert_eq!(info["max_audio_quality"], "HIRES_L2");

    let (_, info) = call(port, get("get-connect-info")).await;
    assert_eq!(info["current_session_id"], "");
    assert_eq!(info["app_id"], "123");
    discovery.set_session(Some(&[1; 16]));
    let (_, info) = call(port, get("get-connect-info")).await;
    assert_eq!(
        info["current_session_id"],
        "01010101-0101-0101-0101-010101010101"
    );

    let payload = r#"{"session_id":"s1","jwt_qconnect":{"endpoint":"wss://example.test/ws","jwt":"socket","exp":5},"jwt_api":{"jwt":"api","exp":6}}"#;
    let request = format!(
        "POST /devices/0707/connect-to-qconnect HTTP/1.1\r\nHost: device\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
        payload.len()
    );
    let (status, answer) = call(port, request).await;
    assert_eq!(status, "HTTP/1.1 200 OK");
    assert_eq!(answer["success"], true);
    let handover = handovers.recv().await.unwrap();
    assert_eq!(handover.session_id, "s1");
    assert_eq!(handover.credentials.endpoint, "wss://example.test/ws");
    assert_eq!(handover.credentials.jwt, "socket");
    assert_eq!(
        (
            handover.expires,
            handover.api_jwt.as_str(),
            handover.api_expires
        ),
        (5, "api", 6)
    );

    let (status, _) = call(port, get("something-else")).await;
    assert_eq!(status, "HTTP/1.1 404 Not Found");
}
