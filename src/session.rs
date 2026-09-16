//! A device's membership in a Qobuz Connect session, joined as a controller renderer like the official apps.

use std::collections::VecDeque;
use std::future::Future;

use crate::Error;
use crate::controller::{ControllerCommand, uuid};
use crate::device::Device;
use crate::event::{self, Event, QueueEvent, RendererEvent};
use crate::proto::qconnect::{
    CtrlSrvrAskForQueueState, CtrlSrvrAskForRendererState, CtrlSrvrJoinSession, MessageType,
    QConnectMessage, QueueVersion,
};
use crate::renderer::{RendererCommand, RendererReport};
use crate::transport::{self, Credentials, Transport};

/// A joined session. Dropping it leaves the session.
pub struct Session {
    transport: Transport,
    device: Device,
    renderer_id: Option<i32>,
    uuid: Vec<u8>,
    queue_version: Option<QueueVersion>,
    active: bool,
    pending: VecDeque<(Vec<u8>, ControllerCommand)>,
    action: Option<Vec<u8>>,
}

impl Session {
    /// Connects and joins the session of the account behind the credentials.
    pub async fn join(credentials: Credentials, device: Device) -> Result<Self, Error> {
        Self::joined(Transport::connect(credentials).await?, device).await
    }

    /// Like `join`, with a token minted by `source` before every connection; see `Transport::connect_with`.
    pub async fn join_with<S, F>(source: S, device: Device) -> Result<Self, Error>
    where
        S: FnMut() -> F + Send + 'static,
        F: Future<Output = Result<Credentials, Error>> + Send + 'static,
    {
        Self::joined(Transport::connect_with(source).await?, device).await
    }

    async fn joined(transport: Transport, device: Device) -> Result<Self, Error> {
        let session = Self {
            transport,
            device,
            renderer_id: None,
            uuid: Vec::new(),
            queue_version: None,
            active: false,
            pending: VecDeque::new(),
            action: None,
        };
        session.send_join().await?;
        Ok(session)
    }

    /// Next event, `None` once the connection is gone for good.
    pub async fn recv(&mut self) -> Option<Event> {
        let event = match self.transport.recv().await? {
            transport::Event::Disconnected => Event::Disconnected,
            transport::Event::Reconnected => {
                self.renderer_id = None;
                self.active = false;
                self.pending.clear();
                self.action = None;
                let _ = self.send_join().await;
                Event::Reconnected
            }
            transport::Event::Message(message) => self.translate(*message).await,
        };
        Some(event)
    }

    /// Tells the session about this renderer.
    pub async fn report(&self, report: RendererReport) -> Result<(), Error> {
        self.send(report.into_message(self.queue_version)).await
    }

    /// Sends a controller command. Commands go out in order; a queue change carries the queue version the session tracks, and the commands behind it wait until the server has answered it with the queue event that carries the next version. Returns the action uuid of a queue change, which that answer echoes.
    pub async fn control(&mut self, command: ControllerCommand) -> Result<Option<Vec<u8>>, Error> {
        let action = command.changes_queue().then(uuid);
        self.pending
            .push_back((action.clone().unwrap_or_default(), command));
        self.dispatch().await?;
        Ok(action)
    }

    /// Makes this device the active renderer of the session.
    pub async fn activate(&mut self) -> Result<(), Error> {
        let renderer_id = self.renderer_id.ok_or(Error::NotRegistered)?;
        self.control(ControllerCommand::SetActiveRenderer(renderer_id))
            .await
            .map(|_| ())
    }

    /// Asks for the full queue; the answer arrives as a queue state event.
    pub async fn ask_queue_state(&self) -> Result<(), Error> {
        self.send(QConnectMessage {
            message_type: MessageType::CtrlSrvrAskForQueueState.into(),
            ctrl_srvr_ask_for_queue_state: Some(CtrlSrvrAskForQueueState {
                queue_version_ref: self.queue_version,
                action_uuid: uuid(),
            }),
            ..Default::default()
        })
        .await
    }

    /// Asks a renderer to report its state; the answer arrives as a renderer state event.
    pub async fn ask_renderer_state(&self, renderer_id: i32) -> Result<(), Error> {
        self.send(QConnectMessage {
            message_type: MessageType::CtrlSrvrAskForRendererState.into(),
            ctrl_srvr_ask_for_renderer_state: Some(CtrlSrvrAskForRendererState { renderer_id }),
            ..Default::default()
        })
        .await
    }

    #[must_use]
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// The renderer id the server gave this device, once registered.
    #[must_use]
    pub fn renderer_id(&self) -> Option<i32> {
        self.renderer_id
    }

    #[must_use]
    pub fn session_uuid(&self) -> &[u8] {
        &self.uuid
    }

    /// The latest queue version seen, stamped on every report and queue change.
    #[must_use]
    pub fn queue_version(&self) -> Option<&QueueVersion> {
        self.queue_version.as_ref()
    }

    /// Whether this device is the active renderer of the session.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active
    }

    async fn translate(&mut self, message: QConnectMessage) -> Event {
        if let Some(added) = &message.srvr_ctrl_add_renderer {
            let ours = added
                .device_info
                .as_ref()
                .is_some_and(|info| info.device_uuid == self.device.uuid);
            if ours {
                self.renderer_id = Some(added.renderer_id);
                return Event::Registered {
                    renderer_id: added.renderer_id,
                };
            }
        }
        if let Some(version) = message
            .srvr_rndr_set_state
            .as_ref()
            .and_then(|state| state.queue_version)
        {
            self.queue_version = Some(version);
        }
        let event = event::from_message(message);
        match &event {
            Event::Session(state) => {
                self.uuid.clone_from(&state.session_uuid);
                self.queue_version = state.queue_version;
                self.active = state.active_renderer_id.is_some()
                    && state.active_renderer_id == self.renderer_id;
                let _ = self.ask_queue_state().await;
            }
            Event::Renderer(RendererEvent::ActiveChanged { id }) => {
                self.active = id.is_some() && *id == self.renderer_id;
            }
            Event::Command(RendererCommand::SetActive(active)) => self.active = *active,
            Event::Queue(queue) => {
                if let Some(version) = queue.queue_version() {
                    self.queue_version = Some(*version);
                }
                self.answered(queue).await;
            }
            _ => {}
        }
        event
    }

    async fn answered(&mut self, queue: &QueueEvent) {
        let ours = self
            .action
            .as_deref()
            .is_some_and(|action| Some(action) == queue.action_uuid());
        if !ours {
            return;
        }
        self.action = None;
        if matches!(queue, QueueEvent::Error(_)) {
            self.pending.clear();
            let _ = self.ask_queue_state().await;
        }
        let _ = self.dispatch().await;
    }

    async fn dispatch(&mut self) -> Result<(), Error> {
        while self.action.is_none() {
            let Some((action, command)) = self.pending.pop_front() else {
                break;
            };
            self.action = command.changes_queue().then(|| action.clone());
            self.send(command.into_message(self.queue_version, &action))
                .await?;
        }
        Ok(())
    }

    async fn send_join(&self) -> Result<(), Error> {
        self.send(QConnectMessage {
            message_type: MessageType::CtrlSrvrJoinSession.into(),
            ctrl_srvr_join_session: Some(CtrlSrvrJoinSession {
                session_uuid: (!self.uuid.is_empty()).then(|| self.uuid.clone()),
                device_info: Some(self.device.to_proto()),
            }),
            ..Default::default()
        })
        .await
    }

    async fn send(&self, message: QConnectMessage) -> Result<(), Error> {
        self.transport.send(vec![message]).await
    }
}
