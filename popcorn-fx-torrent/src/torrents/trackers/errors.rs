use thiserror::Error;
use tokio::time::error::Elapsed;
use url::Url;

pub type Result<T> = std::result::Result<T, TrackerError>;

#[derive(Debug, Clone, Error, PartialEq)]
pub enum TrackerError {
    #[error("tracker has no available addresses (left)")]
    Unavailable,
    #[error("an error occurred while connecting to the tracker, {0}")]
    Connection(String),
    #[error("tracker scheme \"{0}\" is not supported")]
    UnsupportedScheme(String),
    #[error("an io error occurred while communicating with the tracker, {0}")]
    Io(String),
    #[error("tracker url \"{0}\" is already registered")]
    DuplicateUrl(Url),
    #[error("unable to start trackers, info hash is missing")]
    InfoHashMissing,
}

impl From<std::io::Error> for TrackerError {
    fn from(error: std::io::Error) -> Self {
        TrackerError::Io(error.to_string())
    }
}

impl From<Elapsed> for TrackerError {
    fn from(value: Elapsed) -> Self {
        TrackerError::Io(value.to_string())
    }
}
