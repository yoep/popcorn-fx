use std::fs::File;

use crate::core::subtitles::cue::SubtitleCue;
use crate::core::subtitles::errors::SubtitleParseError;
pub use crate::core::subtitles::parsers::srt::SrtParser;
pub use crate::core::subtitles::parsers::style_parser::StyleParser;
pub use crate::core::subtitles::parsers::vtt::VttParser;

mod srt;
mod vtt;
mod style_parser;

/// A subtitle parser which is able to convert a [File] into a [Subtitle] or visa-versa.
pub trait Parser: Send + Sync {
    /// Parse the given file to subtitle cues.
    /// Invalid lines within the given buffer will be ignored by the parser and logged as a warning.
    fn parse_file(&self, file: File) -> Vec<SubtitleCue>;

    /// Parse the given data [String] to subtitle cues.
    /// Invalid lines within the given buffer will be ignored by the parser and logged as a warning.
    fn parse_string(&self, value: &String) -> Vec<SubtitleCue>;

    /// Parse the given cues to the raw output of the extension.
    fn parse_raw(&self, cues: &Vec<SubtitleCue>) -> Result<String, SubtitleParseError>;
}