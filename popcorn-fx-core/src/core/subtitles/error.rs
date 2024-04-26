use derive_more::Display;
use thiserror::Error;

use crate::core::subtitles::model::SubtitleType;

/// The specialized subtitle result.
pub type Result<T> = std::result::Result<T, SubtitleError>;

/// Represents errors specific to subtitles.
#[derive(PartialEq, Debug, Clone, Error)]
pub enum SubtitleError {
    /// Failed to create a valid URL.
    #[error("Failed to create valid URL: {0}")]
    InvalidUrl(String),
    /// Failed to retrieve available subtitles.
    #[error("Failed to retrieve available subtitles: {0}")]
    SearchFailed(String),
    /// Failed to download the subtitle file.
    #[error("Failed to download subtitle {0}: {1}")]
    DownloadFailed(String, String),
    /// IO error occurred while handling the subtitle.
    #[error("Failed to write subtitle file to {0}: {1}")]
    IO(String, String),
    /// Failed to parse the subtitle file.
    #[error("Failed to parse file {0}: {1}")]
    ParseFileError(String, String),
    /// Failed to parse the subtitle URL.
    #[error("Failed to parse URL: {0}")]
    ParseUrlError(String),
    /// Subtitle conversion failed.
    #[error("Subtitle conversion to {0} failed: {1}")]
    ConversionFailed(SubtitleType, String),
    /// Subtitle type is not supported.
    #[error("Subtitle type {0} is not supported")]
    TypeNotSupported(SubtitleType),
    /// No available subtitle files found.
    #[error("No available subtitle files found")]
    NoFilesFound,
    /// Invalid subtitle file.
    #[error("File {0} is invalid: {1}")]
    InvalidFile(String, String),
}

#[derive(PartialEq, Debug, Display)]
pub enum SubtitleParseError {
    #[display(fmt = "Parsing failed with {}", _0)]
    Failed(String),
    #[display(fmt = "Extension {} is not supported", _0)]
    ExtensionNotSupported(String),
    #[display(fmt = "File contains invalid time, {}", _0)]
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
