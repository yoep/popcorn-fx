use crate::core::subtitles::cue::SubtitleCue;
pub use crate::core::subtitles::parsers::srt::SrtParser;
pub use crate::core::subtitles::parsers::style_parser::StyleParser;
pub use crate::core::subtitles::parsers::vtt::VttParser;
use crate::core::subtitles::{Result, SubtitleParseError};
use async_trait::async_trait;
use std::fmt::Debug;
use tokio::fs::File;

mod srt;
mod style_parser;
mod vtt;

const NEWLINE: &str = "\n";

/// A subtitle parser which is able to convert a [File] into a [Subtitle] or visa-versa.
#[async_trait]
pub trait Parser: Debug + Send + Sync {
    /// Parse the given file to subtitle cues.
    /// Invalid lines within the given buffer will be ignored by the parser and logged as a warning.
    async fn parse_file(&self, file: File) -> Result<Vec<SubtitleCue>>;

    /// Parse the given data [String] to subtitle cues.
    /// Invalid lines within the given buffer will be ignored by the parser and logged as a warning.
    async fn parse_string(&self, value: &String) -> Result<Vec<SubtitleCue>>;

    /// Convert the given [SubtitleCue]'s to the raw format of the extension.
    /// This is always represented as a plain text value.
    ///
    /// * `cues` - The array of [SubtitleCue] consisting of at least one cue to prevent corruption of the output.
    ///
    /// It returns the plain text value on successful conversion, else the [SubtitleParseError].
    fn convert(&self, cues: &Vec<SubtitleCue>) -> std::result::Result<String, SubtitleParseError>;
}
