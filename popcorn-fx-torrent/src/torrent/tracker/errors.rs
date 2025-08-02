use crate::torrent::tracker::TrackerHandle;
use crate::torrent::InfoHash;
use serde_bencode::Error;
use thiserror::Error;
use tokio::time::error::Elapsed;
use url::{ParseError, Url};

/// The result type of tracker operations.
pub type Result<T> = std::result::Result<T, TrackerError>;

/// The tracker specific errors.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum TrackerError {
    #[error("the tracker handle {0} is invalid")]
    InvalidHandle(TrackerHandle),
    #[error("tracker url \"{0}\" is invalid")]
    InvalidUrl(String),
    #[error("info hash {0} not found within tracker")]
    InfoHashNotFound(InfoHash),
    #[error("tracker has no available addresses (left)")]
    Unavailable,
    #[error("an error occurred while connecting to the tracker, {0}")]
    Connection(String),
    #[error("tracker scheme \"{0}\" is not supported")]
    UnsupportedScheme(String),
    #[error("failed to announce to the tracker, {0}")]
    AnnounceError(String),
    #[error("an io error occurred while communicating with the tracker, {0}")]
    Io(String),
    #[error("failed to parse tracker response, {0}")]
    Parse(String),
    #[error("tracker url \"{0}\" is already registered")]
    DuplicateUrl(Url),
    #[error("the connection to the tracker at \"{0}\" has timed out")]
    Timeout(Url),
    #[error("unable to execute the operation, no active trackers available")]
    NoTrackers,
}

impl From<std::io::Error> for TrackerError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<Elapsed> for TrackerError {
    fn from(error: Elapsed) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<ParseError> for TrackerError {
    fn from(error: ParseError) -> Self {
        Self::InvalidUrl(error.to_string())
    }
}

impl From<serde_bencode::error::Error> for TrackerError {
    fn from(error: Error) -> Self {
        Self::Parse(error.to_string())
    }
}

impl From<reqwest::Error> for TrackerError {
    fn from(error: reqwest::Error) -> Self {
        Self::Io(error.to_string())
    }
}
