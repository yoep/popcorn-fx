use derive_more::Display;
use std::io;
use thiserror::Error;

use crate::core::subtitles::model::SubtitleType;

/// The specialized subtitle result.
pub type Result<T> = std::result::Result<T, SubtitleError>;

/// Represents errors specific to subtitles.
#[derive(Debug, Error)]
pub enum SubtitleError {
    /// Failed to create a valid URL.
    #[error("failed to create valid URL: {0}")]
    InvalidUrl(String),
    /// Failed to retrieve available subtitles.
    #[error("failed to retrieve available subtitles: {0}")]
    SearchFailed(String),
    /// Failed to download the subtitle file.
    #[error("failed to download subtitle {0}: {1}")]
    DownloadFailed(String, String),
    /// IO error occurred while handling the subtitle.
    #[error("an io error occurred, {0}")]
    IO(io::Error),
    /// Failed to parse the subtitle file.
    #[error("failed to parse file {0}: {1}")]
    ParseFileError(String, String),
    /// Failed to parse the subtitle URL.
    #[error("failed to parse URL: {0}")]
    ParseUrlError(String),
    /// Subtitle conversion failed.
    #[error("subtitle conversion to {0} failed: {1}")]
    ConversionFailed(SubtitleType, String),
    /// Subtitle type is not supported.
    #[error("subtitle type {0} is not supported")]
    TypeNotSupported(SubtitleType),
    /// No available subtitle files found.
    #[error("no available subtitle files found")]
    NoFilesFound,
    /// Invalid subtitle file.
    #[error("file {0} is invalid: {1}")]
    InvalidFile(String, String),
}

impl PartialEq for SubtitleError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::InvalidUrl(_), Self::InvalidUrl(_)) => true,
            (Self::SearchFailed(_), Self::SearchFailed(_)) => true,
            (Self::DownloadFailed(_, _), Self::DownloadFailed(_, _)) => true,
            (Self::IO(_), Self::IO(_)) => true,
            (Self::ParseFileError(_, _), Self::ParseFileError(_, _)) => true,
            (Self::ParseUrlError(_), Self::ParseUrlError(_)) => true,
            (Self::ConversionFailed(_, _), Self::ConversionFailed(_, _)) => true,
            (Self::TypeNotSupported(_), Self::TypeNotSupported(_)) => true,
            (Self::NoFilesFound, Self::NoFilesFound) => true,
            (Self::InvalidFile(_, _), Self::InvalidFile(_, _)) => true,
            _ => false,
        }
    }
}

impl From<io::Error> for SubtitleError {
    fn from(e: io::Error) -> Self {
        SubtitleError::IO(e)
    }
}

#[derive(PartialEq, Debug, Display)]
pub enum SubtitleParseError {
    #[display("Parsing failed with {}", _0)]
    Failed(String),
    #[display("Extension {} is not supported", _0)]
    ExtensionNotSupported(String),
    #[display("File contains invalid time, {}", _0)]
    InvalidTime(String),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_subtitle_parse_error_display() {
        assert_eq!(
            "Parsing failed with lorem".to_string(),
            SubtitleParseError::Failed("lorem".to_string()).to_string()
        );
        assert_eq!(
            "Extension lol is not supported".to_string(),
            SubtitleParseError::ExtensionNotSupported("lol".to_string()).to_string()
        );
        assert_eq!(
            "File contains invalid time, 13".to_string(),
            SubtitleParseError::InvalidTime("13".to_string()).to_string()
        );
    }
}
