use std::fs::File;

use crate::core::subtitles::cue::SubtitleCue;
use crate::core::subtitles::errors::SubtitleParseError;
pub use crate::core::subtitles::parsers::srt::SrtParser;
pub use crate::core::subtitles::parsers::style_parser::StyleParser;
pub use crate::core::subtitles::parsers::vtt::VttParser;

mod srt;
mod vtt;
mod style_parser;
mod utils;

const NEWLINE: &str = "\n";

/// A subtitle parser which is able to convert a [File] into a [Subtitle] or visa-versa.
pub trait Parser: Send + Sync {
    /// Parse the given file to subtitle cues.
    /// Invalid lines within the given buffer will be ignored by the parser and logged as a warning.
    fn parse_file(&self, file: File) -> Vec<SubtitleCue>;

    /// Parse the given data [String] to subtitle cues.
    /// Invalid lines within the given buffer will be ignored by the parser and logged as a warning.
    fn parse_string(&self, value: &String) -> Vec<SubtitleCue>;

    /// Convert the given [SubtitleCue]'s to the raw format of the extension.
    /// This is always represented as a plain text value.
    ///
    /// * `cues` - The array of [SubtitleCue] consisting of at least one cue to prevent corruption of the output.
    ///
    /// It returns the plain text value on successful conversion, else the [SubtitleParseError].
    fn convert(&self, cues: &Vec<SubtitleCue>) -> Result<String, SubtitleParseError>;
}
