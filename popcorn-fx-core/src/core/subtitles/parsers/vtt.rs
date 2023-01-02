use std::fs::File;

use regex::Regex;

use crate::core::subtitles::cue::SubtitleCue;
use crate::core::subtitles::errors::SubtitleParseError;
use crate::core::subtitles::parsers::{Parser, StyleParser};

const HEADER: &str = "WEBVTT";
const TIME_INDICATOR: &str = " --> ";
const TIME_PATTERN: &str = "HH:mm:ss.SSS";

pub struct VttParser {
    time_regex: Regex,
    style_parser: StyleParser,
}

impl VttParser {
    /// Create a new vtt parser instance.
    pub fn new() -> Self {
        Self {
            time_regex: Regex::new(TIME_PATTERN).unwrap(),
            style_parser: StyleParser::new(),
        }
    }
}

impl Parser for VttParser {
    fn parse_file(&self, file: File) -> Vec<SubtitleCue> {
        todo!()
    }

    fn parse_string(&self, value: &String) -> Vec<SubtitleCue> {
        todo!()
    }

    fn parse_raw(&self, cues: &Vec<SubtitleCue>) -> Result<String, SubtitleParseError> {
        todo!()
    }
}