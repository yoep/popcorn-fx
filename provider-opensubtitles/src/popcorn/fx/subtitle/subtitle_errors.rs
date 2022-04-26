use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum SubtitleError {
    ParsingFailed
}

impl Display for SubtitleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SubtitleError::ParsingFailed =>
                write!(f, "Failed to parse subtitle"),
        }
    }
}