//! LAN handshake of the native Qobuz apps: an mDNS advertisement and three HTTP calls through which an app picks the device and hands it the session and tokens of its own user.

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use http_body_util::{BodyExt as _, Full};
use hyper::body::Incoming;
use hyper::header::{CONTENT_TYPE, HeaderValue};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use mdns_sd::{ServiceDaemon, ServiceInfo};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::Error;
use crate::device::Device;
use crate::proto::qconnect::{AudioQuality, DeviceType};
use crate::transport::Credentials;

const SERVICE: &str = "_qobuz-connect._tcp.local.";
const SDK_VERSION: &str = "0.9.6";
const DEFAULT_ENDPOINT: &str = "wss://play.qobuz.com/ws";

/// What an app hands over when it picks the device: its session and the tokens minted for its user.
#[derive(Debug, Clone)]
pub struct Handover {
    pub session_id: String,
    pub credentials: Credentials,
    /// Unix time at which the socket token expires.
    pub expires: u64,
    pub api_jwt: String,
    pub api_expires: u64,
}

/// The device as the apps see it on the LAN. Dropping it withdraws the advertisement and closes the server.
pub struct Discovery {
    port: u16,
    session: Arc<Mutex<Option<String>>>,
    server: JoinHandle<()>,
    mdns: Option<(ServiceDaemon, String)>,
}

struct Shared {
    display: DisplayInfo,
    app_id: String,
    session: Arc<Mutex<Option<String>>>,
    handovers: mpsc::Sender<Handover>,
}

#[derive(Serialize)]
struct DisplayInfo {
    #[serde(rename = "type")]
    kind: &'static str,
    friendly_name: String,
    model_display_name: String,
    brand_display_name: String,
    serial_number: String,
    max_audio_quality: &'static str,
}

#[derive(Serialize)]
struct ConnectInfo<'a> {
    current_session_id: &'a str,
    app_id: &'a str,
}

#[derive(Deserialize)]
struct Token {
    endpoint: Option<String>,
    jwt: String,
    exp: u64,
}

#[derive(Deserialize)]
struct ConnectRequest {
    session_id: String,
    jwt_qconnect: Token,
    jwt_api: Token,
}

#[derive(Serialize)]
struct ConnectResponse {
    success: bool,
}

impl Discovery {
    /// Binds `port` on every interface, 0 for any free one, advertises the device and serves the apps; handovers arrive on the returned receiver. A failed advertisement is logged, the server still runs.
    pub async fn start(
        device: &Device,
        app_id: &str,
        port: u16,
    ) -> Result<(Self, mpsc::Receiver<Handover>), Error> {
        let address = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = TcpListener::bind(address)
            .await
            .map_err(|err| Error::Discovery(err.to_string()))?;
        let port = listener
            .local_addr()
            .map_err(|err| Error::Discovery(err.to_string()))?
            .port();
        let (handovers, receiver) = mpsc::channel(4);
        let session = Arc::new(Mutex::new(None));
        let shared = Arc::new(Shared {
            display: display(device),
            app_id: app_id.to_owned(),
            session: session.clone(),
            handovers,
        });
        let server = tokio::spawn(serve(listener, shared));
        let mdns = advertise(device, port)
            .map_err(|err| tracing::warn!(%err, "advertising on the LAN failed"))
            .ok();
        let discovery = Self {
            port,
            session,
            server,
            mdns,
        };
        Ok((discovery, receiver))
    }

    #[must_use]
    pub fn port(&self) -> u16 {
        self.port
    }

    /// The session the device is in, told to apps that ask; `None` before joining one.
    pub fn set_session(&self, session_uuid: Option<&[u8]>) {
        let id = session_uuid
            .and_then(|bytes| uuid::Uuid::from_slice(bytes).ok())
            .map(|uuid| uuid.hyphenated().to_string());
        if let Ok(mut session) = self.session.lock() {
            *session = id;
        }
    }
}

impl Drop for Discovery {
    fn drop(&mut self) {
        self.server.abort();
        if let Some((daemon, fullname)) = &self.mdns {
            let _ = daemon.unregister(fullname);
            let _ = daemon.shutdown();
        }
    }
}

fn advertise(device: &Device, port: u16) -> Result<(ServiceDaemon, String), mdns_sd::Error> {
    let uuid = uuid::Uuid::from_bytes(device.uuid).hyphenated().to_string();
    let properties = [
        ("path", format!("/devices/{uuid}")),
        ("type", kind_name(device.kind).to_owned()),
        ("Name", device.name.clone()),
        ("device_uuid", uuid.clone()),
        ("sdk_version", SDK_VERSION.to_owned()),
    ];
    let host = format!("{uuid}.local.");
    let info = ServiceInfo::new(
        SERVICE,
        &device.name,
        &host,
        (),
        port,
        properties.as_slice(),
    )?
    .enable_addr_auto();
    let fullname = info.get_fullname().to_owned();
    let daemon = ServiceDaemon::new()?;
    daemon.register(info)?;
    tracing::info!(port, "advertised on the LAN");
    Ok((daemon, fullname))
}

async fn serve(listener: TcpListener, shared: Arc<Shared>) {
    loop {
        let stream = match listener.accept().await {
            Ok((stream, _)) => stream,
            Err(err) => {
                tracing::warn!(%err, "accepting a LAN connection failed");
                continue;
            }
        };
        let shared = shared.clone();
        tokio::spawn(async move {
            let service = service_fn(move |request| handle(request, shared.clone()));
            if let Err(err) = http1::Builder::new()
                .serve_connection(TokioIo::new(stream), service)
                .await
            {
                tracing::debug!(%err, "LAN connection ended");
            }
        });
    }
}

async fn handle(
    request: Request<Incoming>,
    shared: Arc<Shared>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let method = request.method().clone();
    let route = request
        .uri()
        .path()
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .to_owned();
    let response = match (method, route.as_str()) {
        (Method::GET, "get-display-info") => json(&shared.display),
        (Method::GET, "get-connect-info") => {
            let session = shared
                .session
                .lock()
                .ok()
                .and_then(|guard| guard.clone())
                .unwrap_or_default();
            json(&ConnectInfo {
                current_session_id: &session,
                app_id: &shared.app_id,
            })
        }
        (Method::POST, "connect-to-qconnect") => {
            let body = request.into_body().collect().await?.to_bytes();
            match serde_json::from_slice::<ConnectRequest>(&body) {
                Ok(connect) => {
                    tracing::info!(session = %connect.session_id, "an app hands over its session");
                    let success = shared.handovers.try_send(handover(connect)).is_ok();
                    json(&ConnectResponse { success })
                }
                Err(err) => {
                    tracing::warn!(%err, "an app sent an unreadable handover");
                    status(StatusCode::BAD_REQUEST)
                }
            }
        }
        _ => status(StatusCode::NOT_FOUND),
    };
    Ok(response)
}

fn handover(connect: ConnectRequest) -> Handover {
    Handover {
        session_id: connect.session_id,
        credentials: Credentials {
            endpoint: connect
                .jwt_qconnect
                .endpoint
                .unwrap_or_else(|| DEFAULT_ENDPOINT.to_owned()),
            jwt: connect.jwt_qconnect.jwt,
        },
        expires: connect.jwt_qconnect.exp,
        api_jwt: connect.jwt_api.jwt,
        api_expires: connect.jwt_api.exp,
    }
}

fn json<T: Serialize>(value: &T) -> Response<Full<Bytes>> {
    let Ok(body) = serde_json::to_vec(value) else {
        return status(StatusCode::INTERNAL_SERVER_ERROR);
    };
    let mut response = Response::new(Full::new(Bytes::from(body)));
    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    response
}

fn status(status: StatusCode) -> Response<Full<Bytes>> {
    let mut response = Response::new(Full::new(Bytes::new()));
    *response.status_mut() = status;
    response
}

fn display(device: &Device) -> DisplayInfo {
    DisplayInfo {
        kind: kind_name(device.kind),
        friendly_name: device.name.clone(),
        model_display_name: device.model.clone(),
        brand_display_name: device.brand.clone(),
        serial_number: uuid::Uuid::from_bytes(device.uuid).hyphenated().to_string(),
        max_audio_quality: quality_name(device.max_audio_quality),
    }
}

/// The apps' names for output types and qualities, the ones the web player lists as well.
fn kind_name(kind: DeviceType) -> &'static str {
    match kind {
        DeviceType::Speaker => "SPEAKER",
        DeviceType::Streamer => "STREAMER",
        DeviceType::Tv => "TV",
        DeviceType::Soundbar => "SOUNDBAR",
        DeviceType::Computer => "COMPUTER",
        DeviceType::Mobile => "MOBILE",
        DeviceType::Cast => "CAST",
        DeviceType::Headphones => "HEADPHONES",
        DeviceType::Tablet => "TABLET",
        DeviceType::Unknown => "UNKNOWN",
    }
}

fn quality_name(quality: AudioQuality) -> &'static str {
    match quality {
        AudioQuality::Mp3 => "MP3",
        AudioQuality::Cd => "CD",
        AudioQuality::HiresLevel1 => "HIRES_L1",
        AudioQuality::HiresLevel2 | AudioQuality::HiresLevel3 => "HIRES_L2",
        AudioQuality::Unknown => "UNKNOWN",
    }
}
