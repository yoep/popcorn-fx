use std::fs::File;

use chrono::NaiveTime;
use log::{debug, trace};
use regex::Regex;

use crate::core::subtitles::cue::SubtitleCue;
use crate::core::subtitles::error::SubtitleParseError;
use crate::core::subtitles::parsers::{NEWLINE, Parser, StyleParser};
use crate::core::utils::time::parse_time_from_millis;

const HEADER: &str = "WEBVTT";
const TIME_INDICATOR: &str = "-->";
const TIME_FORMAT: &str = "%H:%M:%S.%3f";

#[derive(Debug)]
pub struct VttParser {
    _time_regex: Regex,
    style_parser: StyleParser,
}

impl VttParser {
    fn convert_time_to_string(time: NaiveTime) -> String {
        time.format(TIME_FORMAT).to_string()
    }
}

impl Default for VttParser {
    fn default() -> Self {
        Self {
            _time_regex: Regex::new(TIME_FORMAT).expect("VTT time format should be valid"),
            style_parser: StyleParser::new(),
        }
    }
}

impl Parser for VttParser {
    fn parse_file(&self, _file: File) -> Vec<SubtitleCue> {
        todo!()
    }

    fn parse_string(&self, _value: &String) -> Vec<SubtitleCue> {
        todo!()
    }

    fn convert(&self, cues: &Vec<SubtitleCue>) -> Result<String, SubtitleParseError> {
        trace!("Starting conversion to VTT");
        let mut output = format!("{}{}{}", HEADER, NEWLINE, NEWLINE);

        for cue in cues.iter() {
            let id = cue.id().clone();
            let start_time = parse_time_from_millis(cue.start_time().clone());
            let end_time = parse_time_from_millis(cue.end_time().clone());

            output.push_str(id.as_str());
            output.push_str(NEWLINE);
            output.push_str(
                format!(
                    "{} {} {}",
                    Self::convert_time_to_string(start_time),
                    TIME_INDICATOR,
                    Self::convert_time_to_string(end_time)
                )
                .as_str(),
            );
            output.push_str(NEWLINE);

            for line in cue.lines().iter() {
                output.push_str(self.style_parser.to_line_string(line).as_str());
                output.push_str(NEWLINE);
            }

            output.push_str(NEWLINE);
        }

        debug!("Conversion to VTT completed");
        Ok(output)
    }
}

#[cfg(test)]
mod test {
    use crate::core::subtitles::cue::{StyledText, SubtitleLine};
    use crate::testing::read_test_file_to_string;

    use super::*;

    #[test]
    fn test_parse_raw() {
        let cues = vec![
            SubtitleCue::new(
                "1".to_string(),
                30000,
                48100,
                vec![
                    SubtitleLine::new(vec![StyledText::new(
                        "lorem".to_string(),
                        true,
                        false,
                        false,
                    )]),
                    SubtitleLine::new(vec![StyledText::new(
                        "ipsum".to_string(),
                        false,
                        false,
                        false,
                    )]),
                ],
            ),
            SubtitleCue::new(
                "2".to_string(),
                60000,
                60500,
                vec![SubtitleLine::new(vec![StyledText::new(
                    "dolor".to_string(),
                    false,
                    false,
                    false,
                )])],
            ),
        ];
        let parser = VttParser::default();
        let expected_result =
            read_test_file_to_string("conversion-example.vtt").replace("\r\n", "\n");

        let result = parser.convert(&cues);

        assert_eq!(
            expected_result,
            result.expect("Expected the parsing to have succeeded")
        )
    }
}
