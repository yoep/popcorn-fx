use std::fmt::{Display, Formatter};

use crate::core::subtitles::model::SubtitleType;

#[derive(PartialEq, Debug)]
pub enum SubtitleError {
    InvalidUrl(String),
    SearchFailed(String),
    DownloadFailed(String, String),
    ParsingFailed(String, String),
    ConversionFailed(SubtitleType, String),
    TypeNotSupported(SubtitleType),
    NoFilesFound(),
}

impl Display for SubtitleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SubtitleError::InvalidUrl(message) => write!(f, "Failed to create valid url, {}", message),
            SubtitleError::SearchFailed(message) => write!(f, "Failed to retrieve available subtitles, {}", message),
            SubtitleError::DownloadFailed(id, message) => write!(f, "Failed to download subtitle {}, {}", id, message),
            SubtitleError::ParsingFailed(filename, message) => write!(f, "Failed to parse file {}, {}", filename, message),
            SubtitleError::ConversionFailed(output_type, message) => write!(f, "Subtitle conversion to {} failed, {}", output_type, message),
            SubtitleError::TypeNotSupported(subtitle_type) => write!(f, "Subtitle type {} is not supported", subtitle_type),
            SubtitleError::NoFilesFound() => write!(f, "No available subtitle files found"),
        }
    }
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