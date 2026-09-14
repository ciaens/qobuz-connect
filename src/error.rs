use std::fmt;

use tokio_tungstenite::tungstenite;

/// Failures of the connection to the Qobuz cloud.
#[derive(Debug)]
pub enum Error {
    /// The WebSocket connection could not be established.
    Connect(tungstenite::Error),
    /// The transport task has ended, nothing can be sent any more.
    Closed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect(err) => write!(f, "connecting to Qobuz Connect failed: {err}"),
            Self::Closed => f.write_str("the Qobuz Connect transport is closed"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Connect(err) => Some(err),
            Self::Closed => None,
        }
    }
}

impl From<tungstenite::Error> for Error {
    fn from(err: tungstenite::Error) -> Self {
        Self::Connect(err)
    }
}
