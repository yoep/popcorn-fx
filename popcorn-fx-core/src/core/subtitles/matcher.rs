use log::warn;
use regex::{Captures, Regex};

/// Subtitle matcher which matches the media info against the available [SubtitleInfo].
#[derive(Debug, Clone)]
pub struct SubtitleMatcher {
    name: Option<String>,
    quality: Option<i32>,
}

impl SubtitleMatcher {
    /// Create a new subtitle matcher for the given name and quality.
    pub fn new(name: Option<String>, quality: Option<String>) -> Self {
        let parsed_quality = match quality {
            None => None,
            Some(x) => {
                let quality_regex = Regex::new("([0-9]{3,4})p").unwrap();
                match quality_regex.captures(x.as_str()) {
                    None => None,
                    Some(matcher) => {
                        Self::parse_quality_matcher(matcher)
                    }
                }
            }
        };

        Self {
            name,
            quality: parsed_quality,
        }
    }
    
    pub fn name(&self) -> Option<&String> {
        match &self.name {
            None => None,
            Some(e) => Some(e)
        }
    } 
    
    pub fn quality(&self) -> Option<&i32> {
        match &self.quality {
            None => None,
            Some(e) => Some(e)
        }
    }

    /// Parse the given quality matcher to an integer.
    fn parse_quality_matcher(matcher: Captures) -> Option<i32> {
        let quality_text = matcher.get(0).unwrap().as_str();

        match quality_text.parse::<i32>() {
            Ok(quality) => Some(quality),
            Err(ex) => {
                warn!("invalid quality value found, {}", ex);
                None
            }
        }
    }
}