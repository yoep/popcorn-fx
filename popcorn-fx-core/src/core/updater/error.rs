use crate::core::updater::UpdateState;
use std::io;
use thiserror::Error;

/// The result type for the updater package.
pub type Result<T> = std::result::Result<T, Error>;

/// These error indicate that an issue arose while handling an update action.
#[derive(Debug, Error)]
pub enum Error {
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
    #[error("The update file \"{1}\" couldn't be downloaded, status: {0}")]
    DownloadFailed(String, String),
    /// Indicates that an issue occurred during an io operation
    #[error("io error occurred, {0}")]
    Io(io::Error),
    #[error("No update available to start")]
    UpdateNotAvailable(UpdateState),
    #[error("Failed to extract patch data, {0}")]
    ExtractionFailed(String),
    #[error("The archive location has already been set")]
    ArchiveLocationAlreadyExists,
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::InvalidUpdateChannel(a), Self::InvalidUpdateChannel(b)) => a == b,
            (Self::InvalidApplicationVersion(_, _), Self::InvalidApplicationVersion(_, _)) => true,
            (Self::InvalidRuntimeVersion(_, _), Self::InvalidRuntimeVersion(_, _)) => true,
            (Self::UnknownVersion, Self::UnknownVersion) => true,
            (Self::Response(a), Self::Response(b)) => a == b,
            (Self::InvalidDownloadUrl(a), Self::InvalidDownloadUrl(b)) => a == b,
            (Self::PlatformUpdateUnavailable, Self::PlatformUpdateUnavailable) => true,
            (Self::DownloadFailed(_, _), Self::DownloadFailed(_, _)) => true,
            (Self::Io(_), Self::Io(_)) => true,
            (Self::UpdateNotAvailable(_), Self::UpdateNotAvailable(_)) => true,
            (Self::ExtractionFailed(_), Self::ExtractionFailed(_)) => true,
            (Self::ArchiveLocationAlreadyExists, Self::ArchiveLocationAlreadyExists) => true,
            _ => false,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_io() {
        let err = Error::Io(io::Error::new(io::ErrorKind::Other, "test"));
        assert_eq!(err.to_string(), "io error occurred, test");
    }
}
