use std::fmt::{Display, Formatter, write};
use std::fs::File;
use std::ptr::null;

use crate::popcorn::fx::backend::enum_errors::EnumError;

static SRT_TYPE: &str = "srt";
static VTT_TYPE: &str = "vtt";

#[derive(Debug, PartialEq)]
pub enum SubtitleType {
    SRT,
    VTT,
}

impl SubtitleType {
    /// Retrieve the extension of the given subtitle type.
    pub fn extension(&self) -> String {
        return match *self {
            SubtitleType::SRT => String::from(SRT_TYPE),
            SubtitleType::VTT => String::from(VTT_TYPE)
        };
    }

    /// Retrieve the subtitle type for the given extension
    pub fn value_of(extension: String) -> Result<SubtitleType, EnumError> {
        if extension.eq(SRT_TYPE) {
            return Result::Ok(SubtitleType::SRT);
        } else if extension.eq(VTT_TYPE) {
            return Result::Ok(SubtitleType::VTT);
        }

        return Result::Err(EnumError::NotFound {
            value: extension,
            enum_type: String::from("popcorn::fx::subtitle::parser::SubtitleType"),
        });
    }
}

/// A subtitle which can be rendered during a video playback.
#[derive(Debug)]
pub struct Subtitle {
    /// The detailed info of the subtitle.
    subtitle_info: Option<SubtitleInfo>,
    /// The available cues of the subtitle.
    cues: Vec<SubtitleCue>,
    /// The file from which the subtitle was constructed.
    file: Option<File>,
}

impl Display for Subtitle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return write!(f, "subtitle_info: {:?}, cues: {:?}, file: {:?}", self.subtitle_info, self.cues, self.file)
    }
}

#[derive(Debug, PartialEq)]
pub struct SubtitleInfo {}

/// A subtitle cue defines one or more subtitle lines which need to be displayed at a certain moment
/// in time during a video playback.
#[derive(Debug, PartialEq)]
pub struct SubtitleCue {
    id: String,
    start_time: u64,
    end_time: u64,
    lines: Vec<SubtitleLine>,
}

/// A subtitle line defines a single line to display during a video playback.
#[derive(Debug, PartialEq)]
pub struct SubtitleLine {
    texts: Vec<SubtitleText>,
}

#[derive(Debug, PartialEq)]
pub struct SubtitleText {
    text: String,
    italic: bool,
    bold: bool,
    underline: bool,
}

#[cfg(test)]
mod test {
    use crate::popcorn::fx::backend::enum_errors::EnumError;
    use crate::popcorn::fx::subtitle::model::{SRT_TYPE, SubtitleType, VTT_TYPE};

    #[test]
    fn test_extension_srt() {
        let subtitle_type = SubtitleType::SRT;

        let result = subtitle_type.extension();

        assert_eq!(SRT_TYPE, result)
    }

    #[test]
    fn test_extension_vtt() {
        let subtitle_type = SubtitleType::VTT;

        let result = subtitle_type.extension();

        assert_eq!(VTT_TYPE, result)
    }

    #[test]
    fn test_value_of_srt() {
        let result = SubtitleType::value_of(String::from(SRT_TYPE));

        assert_eq!(SubtitleType::SRT, result.expect("Expected the extension to have been found"));
    }

    #[test]
    fn test_value_of_vtt() {
        let result = SubtitleType::value_of(String::from(VTT_TYPE));

        assert_eq!(SubtitleType::VTT, result.expect("Expected the extension to have been found"));
    }

    #[test]
    fn test_from_not_found_return_error() {
        let my_value = String::from("lipsum");

        let result = SubtitleType::value_of(my_value.clone());

        assert_eq!(EnumError::NotFound { value: my_value.clone(), enum_type: String::from("popcorn::fx::subtitle::parser::SubtitleType") }, result.expect_err("Expected error to have been returned"));
    }
}