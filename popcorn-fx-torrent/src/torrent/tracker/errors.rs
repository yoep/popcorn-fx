use crate::torrent::tracker::TrackerHandle;
use crate::torrent::InfoHash;
use serde_bencode::Error;
use std::io;
use thiserror::Error;
use tokio::time::error::Elapsed;
use url::{ParseError, Url};

/// The result type of tracker operations.
pub type Result<T> = std::result::Result<T, TrackerError>;

/// Error type for tracker-related operations.
///
/// This enum groups all errors that can occur while configuring trackers,
/// connecting to them, or parsing their responses.
#[derive(Debug, Error)]
pub enum TrackerError {
    #[error("the tracker handle {0} is invalid")]
    InvalidHandle(TrackerHandle),
    #[error("tracker url \"{0}\" is invalid")]
    InvalidUrl(String),
    #[error("port {0} is invalid")]
    InvalidPort(u16),
    #[error("info hash {0} not found within tracker")]
    InfoHashNotFound(InfoHash),
    #[error("tracker has no available addresses (left)")]
    Unavailable,
    #[error("an error occurred while connecting to the tracker, {0}")]
    Connection(String),
    #[error("tracker scheme \"{0}\" is not supported")]
    UnsupportedScheme(String),
    #[error("announcement event \"{0}\" is not supported")]
    UnsupportedEvent(String),
    #[error("failed to announce to the tracker, {0}")]
    AnnounceError(String),
    #[error("an io error occurred while communicating with the tracker, {0}")]
    Io(io::Error),
    #[error("failed to parse tracker response, {0}")]
    Parse(String),
    #[error("tracker url \"{0}\" is already registered")]
    DuplicateUrl(Url),
    #[error("timed out while performing the operation")]
    Timeout,
    #[error("unable to execute the operation, no active trackers available")]
    NoTrackers,
}

impl PartialEq for TrackerError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::InvalidHandle(_), Self::InvalidHandle(_)) => true,
            (Self::InvalidUrl(_), Self::InvalidUrl(_)) => true,
            (Self::InvalidPort(_), Self::InvalidPort(_)) => true,
            (Self::InfoHashNotFound(_), Self::InfoHashNotFound(_)) => true,
            (Self::Unavailable, Self::Unavailable) => true,
            (Self::Connection(_), Self::Connection(_)) => true,
            (Self::UnsupportedScheme(_), Self::UnsupportedScheme(_)) => true,
            (Self::UnsupportedEvent(_), Self::UnsupportedEvent(_)) => true,
            (Self::AnnounceError(_), Self::AnnounceError(_)) => true,
            (Self::Io(_), Self::Io(_)) => true,
            (Self::Parse(_), Self::Parse(_)) => true,
            (Self::DuplicateUrl(_), Self::DuplicateUrl(_)) => true,
            (Self::Timeout, Self::Timeout) => true,
            (Self::NoTrackers, Self::NoTrackers) => true,
            _ => false,
        }
    }
}

impl From<io::Error> for TrackerError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<Elapsed> for TrackerError {
    fn from(err: Elapsed) -> Self {
        Self::Io(io::Error::new(io::ErrorKind::TimedOut, err.to_string()))
    }
}

impl From<reqwest::Error> for TrackerError {
    fn from(err: reqwest::Error) -> Self {
        Self::Io(io::Error::new(io::ErrorKind::Other, err.to_string()))
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
