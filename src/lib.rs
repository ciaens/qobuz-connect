#![forbid(unsafe_code)]
//! Qobuz Connect protocol for renderers and controllers.

mod controller;
mod device;
mod error;
mod event;
pub mod proto;
mod renderer;
mod session;
mod transport;
pub mod wire;

pub use controller::{Autoplay, ControllerCommand};
pub use device::Device;
pub use error::Error;
pub use event::{Event, QueueEvent, RendererEvent, SessionState};
pub use renderer::{PlayerState, RendererCommand, RendererReport};
pub use session::Session;
pub use transport::{Credentials, Event as TransportEvent, Transport};
