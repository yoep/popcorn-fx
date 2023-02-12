use log::{debug, trace, warn};
use regex::{Captures, Regex};

/// Subtitle matcher which matches the media info against the available [SubtitleInfo].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubtitleMatcher {
    name: Option<String>,
    quality: Option<i32>,
}

impl SubtitleMatcher {
    /// Create a new subtitle matcher for the given name and quality.
    pub fn from_string(name: Option<String>, quality: Option<String>) -> Self {
        trace!("Creating new subtitle matcher from name: {:?} and quality: {:?}", &name, &quality);
        let parsed_quality = match quality {
            None => None,
            Some(quality_value) => Self::extract_quality(quality_value.as_str())
        };

        Self {
            name,
            quality: parsed_quality,
        }
    }

    /// Create a new subtitle matcher from the given quality as an integer.
    pub fn from_int(name: Option<String>, quality: Option<i32>) -> Self {
        Self {
            name,
            quality,
        }
    }

    pub fn name(&self) -> Option<&str> {
        match &self.name {
            None => None,
            Some(e) => Some(e.as_str())
        }
    }

    pub fn quality(&self) -> Option<&i32> {
        match &self.quality {
            None => None,
            Some(e) => Some(e)
        }
    }

    fn extract_quality(quality_value: &str) -> Option<i32> {
        let quality_regex = Regex::new("([0-9]{3,4})(p)?").expect("Quality regex should be valid");
        match quality_regex.captures(quality_value) {
            None => {
                warn!("Subtitle matcher quality didn't match any quality pattern");
                None
            }
            Some(matcher) => {
                Self::parse_quality_matcher(matcher)
            }
        }
    }

    /// Parse the given quality matcher to an integer.
    fn parse_quality_matcher(matcher: Captures) -> Option<i32> {
        let quality_text = matcher.get(1).expect("Quality text should have matched").as_str();

        trace!("Trying to parse subtitle quality value {}", &quality_text);
        match quality_text.parse::<i32>() {
            Ok(quality) => {
                debug!("Subtitle matcher quality parsed as {}", &quality);
                Some(quality)
            }
            Err(ex) => {
                warn!("Invalid quality value found, {}", ex);
                None
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_from_string() {
        init_logger();
        let name = Some("lorem".to_string());
        let quality = Some("1080p".to_string());
        let expected_result = SubtitleMatcher {
            name: name.clone(),
            quality: Some(1080),
        };

        let result = SubtitleMatcher::from_string(name, quality);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_from_string_with_int_value_only() {
        init_logger();
        let name = Some("ipsum".to_string());
        let quality = Some("720".to_string());
        let expected_result = SubtitleMatcher {
            name: name.clone(),
            quality: Some(720),
        };

        let result = SubtitleMatcher::from_string(name, quality);

        assert_eq!(expected_result, result)
    }
}