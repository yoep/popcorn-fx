use std::fs::File;

use chrono::NaiveTime;
use regex::Regex;

use crate::core::subtitles::cue::SubtitleCue;
use crate::core::subtitles::errors::SubtitleParseError;
use crate::core::subtitles::parsers::{NEWLINE, Parser, StyleParser};
use crate::core::subtitles::parsers::utils::time_from_millis;

const HEADER: &str = "WEBVTT";
const TIME_INDICATOR: &str = "-->";
const TIME_FORMAT: &str = "%H:%M:%S.%3f";

pub struct VttParser {
    time_regex: Regex,
    style_parser: StyleParser,
}

impl VttParser {
    /// Create a new vtt parser instance.
    pub fn new() -> Self {
        Self {
            time_regex: Regex::new(TIME_FORMAT).expect("Time format should be valid"),
            style_parser: StyleParser::new(),
        }
    }

    fn convert_time_to_string(time: NaiveTime) -> String {
        time.format(TIME_FORMAT).to_string()
    }
}

impl Parser for VttParser {
    fn parse_file(&self, _file: File) -> Vec<SubtitleCue> {
        todo!()
    }

    fn parse_string(&self, _value: &String) -> Vec<SubtitleCue> {
        todo!()
    }

    fn parse_raw(&self, cues: &Vec<SubtitleCue>) -> Result<String, SubtitleParseError> {
        let mut output = format!("{}\n\n", HEADER);

        for cue in cues.iter() {
            let id = cue.id().clone();
            let start_time = time_from_millis(cue.start_time().clone());
            let end_time = time_from_millis(cue.end_time().clone());

            output.push_str(id.as_str());
            output.push_str(NEWLINE);
            output.push_str(format!("{} {} {}", Self::convert_time_to_string(start_time), TIME_INDICATOR, Self::convert_time_to_string(end_time)).as_str());
            output.push_str(NEWLINE);

            for line in cue.lines().iter() {
                output.push_str(self.style_parser.to_line_string(line).as_str());
                output.push_str(NEWLINE);
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod test {
    use crate::core::subtitles::cue::{StyledText, SubtitleLine};

    use super::*;

    #[test]
    fn test_parse_raw() {
        let cues = vec![SubtitleCue::new(
            "1".to_string(),
            30000,
            48100,
            vec![
                SubtitleLine::new(vec![StyledText::new("lorem".to_string(), true, false, false)]),
                SubtitleLine::new(vec![StyledText::new("ipsum".to_string(), false, false, false)]),
            ])
        ];
        let parser = VttParser::new();
        let expected_result = format!("{}

1
00:00:30.000 --> 00:00:48.100
<i>lorem</i>
ipsum
", HEADER);

        let result = parser.parse_raw(&cues);

        assert_eq!(expected_result, result.expect("Expected the parsing to have succeeded"))
    }
}