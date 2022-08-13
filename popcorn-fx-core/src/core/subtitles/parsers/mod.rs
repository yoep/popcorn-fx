use std::fs::File;
use std::io::BufReader;

use regex::{Captures, Regex};

use crate::core::subtitles::cue::{StyledText, SubtitleCue, SubtitleLine};
use crate::core::subtitles::errors::SubtitleParseError;

pub mod srt;

const TEXT_PATTERN: &str = "(<([^>]*)>)?([^<]+)(</([^>]*)>)?";
const STYLE_ITALIC: &str = "i";
const STYLE_BOLD: &str = "b";
const STYLE_UNDERLINE: &str = "u";

/// A subtitle parser which is able to convert a [File] into a [Subtitle] or visa-versa.
pub trait Parser: Send + Sync {
    /// Parse the given file to subtitle cues.
    /// Invalid lines within the given buffer will be ignored by the parser and logged as a warning.
    fn parse_file(&self, file: File) -> Vec<SubtitleCue>;

    /// Parse the given data [String] to subtitle cues.
    /// Invalid lines within the given buffer will be ignored by the parser and logged as a warning.
    fn parse_string(&self, value: &String) -> Vec<SubtitleCue>;

    /// Parse the given cues to the raw output of the extension.
    fn parse_raw(&self, cues: &Vec<SubtitleCue>) -> Result<BufReader<String>, SubtitleParseError>;
}

/// The style parser parses text from and to subtitle files based on the basic styles.
/// Complex styles, e.g. colors, are ignored/removed from the line.
/// <p>
/// The following styles are supported:
/// <ul>
///     <li>Italic - i</li>
///     <li>Bold - b</li>
///     <li>Underline - u</li>
/// </ul>
#[derive(Debug)]
pub struct StyleParser {
    regex: Regex,
}

impl StyleParser {
    pub fn new() -> Self {
        Self {
            regex: Regex::new(TEXT_PATTERN).unwrap()
        }
    }

    pub fn parse_line_style(&self, line: &String) -> SubtitleLine {
        let mut texts: Vec<StyledText> = vec![];

        for caps in self.regex.captures_iter(line.as_str()) {
            let text = caps.get(3)
                .map(|e| e.as_str())
                .map(|e| e.replace("\n", ""))
                .map(|e| e.to_string())
                .or_else(|| Some(String::new()))
                .unwrap();
            let style = self.retrieve_style_indicator(&caps);

            if !text.is_empty() {
                texts.push(StyledText::new(text, style == STYLE_ITALIC, style == STYLE_BOLD, style == STYLE_UNDERLINE));
            }
        }

        SubtitleLine::new(texts)
    }

    fn retrieve_style_indicator(&self, caps: &Captures) -> String {
        caps.get(2)
            .or_else(|| caps.get(5))
            .map(|e| e.as_str())
            .map(|e| e.to_string())
            .or_else(|| Some(String::new()))
            .unwrap()
            .to_lowercase()
    }
}

#[cfg(test)]
mod test {
    use crate::core::subtitles::cue::{StyledText, SubtitleLine};
    use crate::core::subtitles::parsers::StyleParser;

    #[test]
    fn test_parse_line_style_multiple_styles() {
        let line = "<i>lorem</i> ipsum <b>dolor</b>".to_string();
        let parser = StyleParser::new();
        let expected_result = SubtitleLine::new(vec![
            StyledText::new("lorem".to_string(), true, false, false),
            StyledText::new(" ipsum ".to_string(), false, false, false),
            StyledText::new("dolor".to_string(), false, true, false),
        ]);

        let result = parser.parse_line_style(&line);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_retrieve_style_indicator_find_start_indicator() {
        let line = "<i>lorem</i>";
        let parser = StyleParser::new();
        let caps = parser.regex.captures(&line).unwrap();
        let expected_result = "i".to_string();

        let result = parser.retrieve_style_indicator(&caps);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_retrieve_style_indicator_find_end_indicator() {
        let line = "lorem</b>";
        let parser = StyleParser::new();
        let caps = parser.regex.captures(&line).unwrap();
        let expected_result = "b".to_string();

        let result = parser.retrieve_style_indicator(&caps);

        assert_eq!(expected_result, result)
    }
}