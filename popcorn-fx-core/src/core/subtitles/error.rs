use derive_more::Display;
use thiserror::Error;

use crate::core::subtitles::model::SubtitleType;

/// The specialized subtitle result.
pub type Result<T> = std::result::Result<T, SubtitleError>;

/// The subtitle specific errors.
#[derive(PartialEq, Debug, Clone, Error)]
pub enum SubtitleError {
    #[error("Failed to create valid url, {0}")]
    InvalidUrl(String),
    #[error("Failed to retrieve available subtitles, {0}")]
    SearchFailed(String),
    /// Indicates that an error occurred while downloading the subtitle file.
    /// It contains the the `file_id` and `error_message`.
    #[error("Failed to download subtitle {0}, {1}")]
    DownloadFailed(String, String),
    /// Indicates that an IO error occurred while handling the subtitle.
    /// It contains the `filepath` and `error_message`.
    #[error("Failed to write subtitle file to {0}, {1}")]
    IO(String, String),
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
        assert_eq!("Parsing failed with lorem".to_string(), SubtitleParseError::Failed("lorem".to_string()).to_string());
        assert_eq!("Extension lol is not supported".to_string(), SubtitleParseError::ExtensionNotSupported("lol".to_string()).to_string());
        assert_eq!("File contains invalid time, 13".to_string(), SubtitleParseError::InvalidTime("13".to_string()).to_string());
    }
}