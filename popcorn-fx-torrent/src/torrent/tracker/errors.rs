use crate::torrent::tracker::TrackerHandle;
use serde_bencode::Error;
use thiserror::Error;
use tokio::time::error::Elapsed;
use url::{ParseError, Url};

pub type Result<T> = std::result::Result<T, TrackerError>;

#[derive(Debug, Clone, Error, PartialEq)]
pub enum TrackerError {
    #[error("the tracker handle {0} is invalid")]
    InvalidHandle(TrackerHandle),
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
    #[error("tracker url \"{0}\" is invalid")]
    InvalidUrl(String),
    #[error("the connection to the tracker at \"{0}\" has timed out")]
    Timeout(Url),
    #[error("unable to start trackers, info hash is missing")]
    InfoHashMissing,
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
