use itertools::Itertools;
use regex::{Captures, Regex};

use crate::core::subtitles::cue::{StyledText, SubtitleLine};

const TEXT_PATTERN: &str = "(<([^>]*)>)?([^<]+)(</([^>]*)>)?";
const STYLE_ITALIC: &str = "i";
const STYLE_BOLD: &str = "b";
const STYLE_UNDERLINE: &str = "u";

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
            regex: Regex::new(TEXT_PATTERN).unwrap(),
        }
    }

    pub fn parse_line_style(&self, line: &String) -> SubtitleLine {
        let mut texts: Vec<StyledText> = vec![];

        for caps in self.regex.captures_iter(line.as_str()) {
            let text = caps
                .get(3)
                .map(|e| e.as_str())
                .map(|e| e.replace("\n", ""))
                .map(|e| e.to_string())
                .or_else(|| Some(String::new()))
                .unwrap();
            let style = self.retrieve_style_indicator(&caps);

            if !text.is_empty() {
                texts.push(StyledText::new(
                    text,
                    style == STYLE_ITALIC,
                    style == STYLE_BOLD,
                    style == STYLE_UNDERLINE,
                ));
            }
        }

        SubtitleLine::new(texts)
    }

    pub fn to_line_string(&self, line: &SubtitleLine) -> String {
        line.texts()
            .iter()
            .map(|e| Self::text_to_string(e))
            .join("")
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

    fn text_to_string(style: &StyledText) -> String {
        let mut output = style.text().clone();

        if *style.italic() {
            output = Self::insert_style(output, STYLE_ITALIC);
        }
        if *style.bold() {
            output = Self::insert_style(output, STYLE_BOLD);
        }
        if *style.underline() {
            output = Self::insert_style(output, STYLE_UNDERLINE);
        }

        output
    }

    fn insert_style(text: String, style_indicator: &str) -> String {
        format!("<{0}>{1}</{0}>", style_indicator, text)
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

    #[test]
    fn test_to_line_string() {
        let line = SubtitleLine::new(vec![
            StyledText::new("lorem".to_string(), true, false, false),
            StyledText::new(" ".to_string(), false, false, false),
            StyledText::new("ipsum".to_string(), false, true, true),
        ]);
        let parser = StyleParser::new();
        let expected_result = "<i>lorem</i> <u><b>ipsum</b></u>".to_string();

        let result = parser.to_line_string(&line);

        assert_eq!(expected_result, result)
    }
}
