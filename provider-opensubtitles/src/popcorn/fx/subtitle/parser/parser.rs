use std::io::{Read, Write};

use crate::popcorn::fx::subtitle::model::{SubtitleCue, SubtitleType};
use crate::popcorn::fx::subtitle::subtitle_errors::SubtitleError;

/// Subtitle parser which can parse a subtitle file to subtitles cues and write subtitles cues
/// back to the original file.
pub trait Parser {
    /// Verify if the parser supports the given subtitle.
    ///
    /// * `subtitle_type` - The type of the subtitle that needs to be supported.
    ///
    /// It returns true when the subtitle type is supported by the parser, else false.
    fn supports(subtitle_type: SubtitleType) -> bool;

    /// Parse the input reader to a list of cues.
    /// It returns a list of cues on success, else the error that occurred.
    fn parse(input: dyn Read) -> Result<Vec<SubtitleCue>, SubtitleError>;

    /// Write the given cues back to the original subtitle format.
    ///
    /// * `cues` - The cues to write to the subtitle format.
    ///
    /// It returns the output buffer on success, else the error that occurred.
    fn write(cues: Vec<SubtitleCue>) -> Result<Box<dyn Write>, SubtitleError>;
}