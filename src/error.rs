use std::fmt;

use tokio_tungstenite::tungstenite;

/// Failures of the connection to the Qobuz cloud.
#[derive(Debug)]
pub enum Error {
    /// The WebSocket connection could not be established.
    Connect(tungstenite::Error),
    /// The transport task has ended, nothing can be sent any more.
    Closed,
    /// The server has not registered this device as a renderer yet.
    NotRegistered,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect(err) => write!(f, "connecting to Qobuz Connect failed: {err}"),
            Self::Closed => f.write_str("the Qobuz Connect transport is closed"),
            Self::NotRegistered => f.write_str("the device is not registered as a renderer yet"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Connect(err) => Some(err),
            Self::Closed | Self::NotRegistered => None,
        }
    }
}

impl From<tungstenite::Error> for Error {
    fn from(err: tungstenite::Error) -> Self {
        Self::Connect(err)
    }
}
