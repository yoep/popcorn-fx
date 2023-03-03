use std::fmt::{Display, Formatter};

use thiserror::Error;

use crate::core::subtitles::model::SubtitleType;

/// The specialized subtitle result.
pub type Result<T> = std::result::Result<T, SubtitleError>;

#[derive(PartialEq, Debug, Clone, Error)]
pub enum SubtitleError {
    #[error("Failed to create valid url, {0}")]
    InvalidUrl(String),
    #[error("Failed to retrieve available subtitles, {0}")]
    SearchFailed(String),
    #[error("Failed to download subtitle {0}, {1}")]
    DownloadFailed(String, String),
    #[error("Failed to write subtitle file to {0}")]
    WritingFailed(String),
    #[error("Failed to parse file {0}, {1}")]
    ParseFileError(String, String),
    #[error("Failed to parse url, {0}")]
    ParseUrlError(String),
    #[error("Subtitle conversion to {0} failed, {1}")]
    ConversionFailed(SubtitleType, String),
    #[error("Subtitle type {0} is not supported")]
    TypeNotSupported(SubtitleType),
    #[error("No available subtitle files found")]
    NoFilesFound,
    #[error("File {0} is invalid, {1}")]
    InvalidFile(String, String),
}

#[derive(PartialEq, Debug)]
pub enum SubtitleParseError {
    Failed(String),
    ExtensionNotSupported(String),
    InvalidTime(String),
}

impl Display for SubtitleParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SubtitleParseError::Failed(message) => write!(f, "Parsing failed with {}", message),
            SubtitleParseError::ExtensionNotSupported(extension) => write!(f, "Extension {} is not supported", extension),
            SubtitleParseError::InvalidTime(message) => write!(f, "File contains invalid time, {}", message),
        }
    }
}