use std::fmt;

use tokio_tungstenite::tungstenite;

/// Failures of the connection to the Qobuz cloud.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The WebSocket connection could not be established.
    Connect(tungstenite::Error),
    /// The transport task has ended, nothing can be sent any more.
    Closed,
    /// The server has not registered this device as a renderer yet.
    NotRegistered,
    /// No Qobuz Connect token could be obtained, with the reason.
    Token(String),
    /// The LAN server could not start, with the reason.
    Discovery(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect(err) => write!(f, "connecting to Qobuz Connect failed: {err}"),
            Self::Closed => f.write_str("the Qobuz Connect transport is closed"),
            Self::NotRegistered => f.write_str("the device is not registered as a renderer yet"),
            Self::Token(reason) => write!(f, "getting a Qobuz Connect token failed: {reason}"),
            Self::Discovery(reason) => {
                write!(f, "serving Qobuz Connect on the LAN failed: {reason}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Connect(err) => Some(err),
            Self::Closed | Self::NotRegistered | Self::Token(_) | Self::Discovery(_) => None,
        }
    }
}

impl From<tungstenite::Error> for Error {
    fn from(err: tungstenite::Error) -> Self {
        Self::Connect(err)
    }
}
