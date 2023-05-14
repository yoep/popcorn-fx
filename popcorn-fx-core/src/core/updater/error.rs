use thiserror::Error;

use crate::core::updater::UpdateState;

/// The result type for the updater package.
pub type Result<T> = std::result::Result<T, UpdateError>;

/// These error indicate that an issue arose while handling an update action.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum UpdateError {
    #[error("The update channel \"{0}\" is invalid and cannot be queried")]
    InvalidUpdateChannel(String),
    #[error("The application version value \"{0}\" is invalid, {1}")]
    InvalidApplicationVersion(String, String),
    #[error("The runtime version value \"{0}\" is invalid, {1}")]
    InvalidRuntimeVersion(String, String),
    #[error("Unable to start update process, no version info is known at this time")]
    UnknownVersion,
    #[error("Received invalid update channel response, {0}")]
    Response(String),
    #[error("The specified download url {0} is invalid")]
    InvalidDownloadUrl(String),
    #[error("No update files are available for the current platform")]
    PlatformUpdateUnavailable,
    /// Indicates that the download failed with the `StatusCode` and `Filename`
    #[error("The update couldn't be downloaded")]
    DownloadFailed(String, String),
    /// Indicates that an issue occurred during an io operation
    #[error("Failed to write update file to {0}")]
    IO(String),
    #[error("No update available to start")]
    UpdateNotAvailable(UpdateState),
    #[error("Failed to extract patch data, {0}")]
    ExtractionFailed(String),
    #[error("The archive location has already been set")]
    ArchiveLocationAlreadyExists,
}