use std::fs::File;
use std::io::{BufRead, BufReader, Read};

use chrono::{NaiveDateTime, NaiveTime, Timelike};
use derive_more::Display;
use log::{trace, warn};
use regex::Regex;

use crate::core::subtitles::cue::{SubtitleCue, SubtitleCueBuilder};
use crate::core::subtitles::errors::SubtitleParseError;
use crate::core::subtitles::parsers::{Parser, StyleParser};

const TIME_SEPARATOR: &str = "-->";
const TIME_PATTERN: &str = "(\\d{1,2}:\\d{2}:\\d{2},\\d{3}) --> (\\d{1,2}:\\d{2}:\\d{2},\\d{3})";
const TIME_FORMAT: &str = "%H:%M:%S.%3f";

pub struct SrtParser {
    time_regex: Regex,
    style_parser: StyleParser,
}

impl SrtParser {
    /// Create a new srt parser instance.
    pub fn new() -> Self {
        Self {
            time_regex: Regex::new(TIME_PATTERN).unwrap(),
            style_parser: StyleParser::new(),
        }
    }

    fn parse<R: Read>(&self, reader: &mut BufReader<R>) -> Vec<SubtitleCue> {
        let mut stage = ParserStage::IDENTIFIER;
        let mut cue_builder = SubtitleCueBuilder::new();
        let mut line_index = 0;
        let mut cues: Vec<SubtitleCue> = vec![];
        let mut continue_reading = true;

        while continue_reading {
            let mut line = String::new();
            let len = reader.read_line(&mut line).unwrap();

            // check if we've reached the end of the current cue
            if line.trim().is_empty() {
                stage = stage.next();
            }

            match stage {
                ParserStage::IDENTIFIER => {
                    cue_builder = self.read_identifier(&line);
                    stage = stage.next();
                }
                ParserStage::TIME => {
                    self.read_time(&mut cue_builder, &line, &line_index);
                    stage = stage.next();
                }
                ParserStage::TEXT => {
                    cue_builder.add_line(self.style_parser.parse_line_style(&line));
                }
                ParserStage::FINISH => {
                    cues.push(cue_builder.build());
                    stage = stage.next();
                }
            }

            continue_reading = len > 0;
            line_index += 1;
        }

        if stage == ParserStage::TEXT {
            cues.push(cue_builder.build());
        }

        cues
    }

    fn read_identifier(&self, line: &String) -> SubtitleCueBuilder {
        let mut builder = SubtitleCueBuilder::new();
        builder.id(line.clone().trim().to_string());
        builder
    }

    fn read_time(&self, builder: &mut SubtitleCueBuilder, line: &String, line_index: &i32) {
        match self.time_regex.captures(line) {
            Some(caps) => {
                let start_time = caps.get(1)
                    .map(|e| e.as_str())
                    .map(|e| e.replace(",", "."))
                    .map(|e| {
                        trace!("Parsing start time {}", e);
                        NaiveTime::parse_from_str(e.as_str(), TIME_FORMAT)
                    })
                    .map(|e| {
                        match e {
                            Ok(time) => SrtParser::convert_to_millis(&time),
                            Err(err) => {
                                warn!("Start time is invalid for line {}, {}, value: {}", line_index, err, line);
                                0
                            }
                        }
                    })
                    .or_else(|| Some(0))
                    .unwrap();
                let end_time = caps.get(2)
                    .map(|e| e.as_str())
                    .map(|e| e.replace(",", "."))
                    .map(|e| {
                        trace!("Parsing end time {}", e);
                        NaiveTime::parse_from_str(e.as_str(), TIME_FORMAT)
                    })
                    .map(|e| {
                        match e {
                            Ok(time) => Self::convert_to_millis(&time),
                            Err(err) => {
                                warn!("End time is invalid for line {}, {}, value: {}", line_index, err, line);
                                0
                            }
                        }
                    })
                    .or_else(|| Some(0))
                    .unwrap();

                builder
                    .start_time(start_time)
                    .end_time(end_time);
            }
            None => {}
        };
    }

    fn convert_to_millis(time: &NaiveTime) -> u64 {
        let hour = time.hour() as u64;
        let minutes = (hour * 60) + (time.minute() as u64);
        let seconds = (minutes * 60) + (time.second() as u64);
        let millis = time.nanosecond() as u64;

        (seconds * 1000) + (millis / 1000000)
    }

    fn convert_time_to_string(time: NaiveTime) -> String {
        time.format(TIME_FORMAT)
            .to_string()
            .replace(".", ",")
    }

    fn from_millis(time: u64) -> NaiveTime {
        NaiveDateTime::from_timestamp_millis(time as i64)
            .expect("Time went in the past")
            .time()
    }
}

impl Parser for SrtParser {
    fn parse_file(&self, file: File) -> Vec<SubtitleCue> {
        let mut reader = BufReader::new(file);
        self.parse(&mut reader)
    }

    fn parse_string(&self, value: &String) -> Vec<SubtitleCue> {
        let mut reader = BufReader::new(value.as_bytes());
        self.parse(&mut reader)
    }

    fn parse_raw(&self, cues: &Vec<SubtitleCue>) -> Result<String, SubtitleParseError> {
        let newline = "\n";
        let mut output = String::new();

        for cue in cues {
            let id = cue.id().clone();
            let start_time = Self::from_millis(cue.start_time().clone());
            let end_time = Self::from_millis(cue.end_time().clone());

            output.push_str(id.as_str());
            output.push_str(newline);
            output.push_str(format!("{} {} {}", Self::convert_time_to_string(start_time), TIME_SEPARATOR, Self::convert_time_to_string(end_time)).as_str());
            output.push_str(newline);

            for line in cue.lines().iter() {
                output.push_str(self.style_parser.to_line_string(line).as_str());
                output.push_str(newline);
            }
        }

        Ok(output)
    }
}

#[derive(Debug, Display, PartialEq)]
enum ParserStage {
    IDENTIFIER,
    TIME,
    TEXT,
    FINISH,
}

impl ParserStage {
    fn next(&self) -> ParserStage {
        match self {
            ParserStage::IDENTIFIER => ParserStage::TIME,
            ParserStage::TIME => ParserStage::TEXT,
            ParserStage::TEXT => ParserStage::FINISH,
            _ => ParserStage::IDENTIFIER
        }
    }
}

#[cfg(test)]
mod test {
    use crate::core::subtitles::cue::{StyledText, SubtitleLine};
    use crate::test::init_logger;

    use super::*;

    #[test]
    fn test_srt_parser_read_identifier() {
        let parser = SrtParser::new();
        let identifier = "my-identifier".to_string();
        let mut expected_result = SubtitleCueBuilder::new();
        expected_result.id(identifier.clone());

        let result = parser.read_identifier(&identifier);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_srt_parser_parse_single_cue() {
        init_logger();
        let mut reader = BufReader::new("1
00:00:30,296 --> 00:00:34,790
<i>Drink up, me hearties, yo ho</i>".as_bytes());
        let parser = SrtParser::new();
        let expected_result: SubtitleCue = SubtitleCue::new("1".to_string(), 30296, 34790, vec![
            SubtitleLine::new(vec![StyledText::new("Drink up, me hearties, yo ho".to_string(), true, false, false)])
        ]);

        let result = parser.parse(&mut reader);

        assert_eq!(vec![expected_result], result);
    }

    #[test]
    fn test_srt_parser_parse_multiple_cues() {
        init_logger();
        let mut reader = BufReader::new("1526
02:12:21,051 --> 02:12:22,951
This is the path
you've chosen, is it?

1527
02:12:26,757 --> 02:12:28,952
The <i>Black Pearl</i> is yours.".as_bytes());
        let parser = SrtParser::new();
        let expected_result: Vec<SubtitleCue> = vec![
            SubtitleCue::new("1526".to_string(), 7941051, 7942951, vec![
                SubtitleLine::new(vec![StyledText::new("This is the path".to_string(), false, false, false)]),
                SubtitleLine::new(vec![StyledText::new("you've chosen, is it?".to_string(), false, false, false)]),
            ]),
            SubtitleCue::new("1527".to_string(), 7946757, 7948952, vec![
                SubtitleLine::new(vec![
                    StyledText::new("The ".to_string(), false, false, false),
                    StyledText::new("Black Pearl".to_string(), true, false, false),
                    StyledText::new(" is yours.".to_string(), false, false, false),
                ]),
            ]),
        ];

        let result = parser.parse(&mut reader);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_parser_stage_next_identifier() {
        let stage = ParserStage::IDENTIFIER;

        let result = stage.next();

        assert_eq!(ParserStage::TIME, result)
    }

    #[test]
    fn test_parser_stage_next_time() {
        let stage = ParserStage::TIME;

        let result = stage.next();

        assert_eq!(ParserStage::TEXT, result)
    }

    #[test]
    fn test_parser_stage_next_text() {
        let stage = ParserStage::TEXT;

        let result = stage.next();

        assert_eq!(ParserStage::FINISH, result)
    }

    #[test]
    fn test_parse_raw() {
        init_logger();
        let cues = vec![SubtitleCue::new(
            "1".to_string(),
            30000,
            48100,
            vec![SubtitleLine::new(
                vec![StyledText::new("lorem".to_string(), true, false, false)])])];
        let parser = SrtParser::new();
        let expected_result = "1
00:00:30,000 --> 00:00:48,100
<i>lorem</i>
".to_string();

        let result = parser.parse_raw(&cues);

        assert_eq!(expected_result, result.expect("Expected the parse_raw to succeed"))
    }
}