#![forbid(unsafe_code)]
//! Qobuz Connect protocol for renderers and controllers.

mod error;
pub mod proto;
mod transport;
pub mod wire;

pub use error::Error;
pub use transport::{Credentials, Event, Transport};
