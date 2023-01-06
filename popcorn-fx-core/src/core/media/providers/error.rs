use std::fmt::{Display, Formatter};

pub type Result<T> = std::result::Result<T, ProviderError>;

/// The errors which are thrown by the media providers.
#[derive(Debug)]
pub enum ProviderError {
    NoAvailableProviders,
    RequestFailed(u16),
    ParsingFailed(String),
}

impl Display for ProviderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderError::NoAvailableProviders => write!(f, "There are no available providers to query"),
            ProviderError::RequestFailed(status) => write!(f, "Request failed with status {}", status),
            ProviderError::ParsingFailed(error) => write!(f, "Failed to parse response, {}", error),
        }
    }
}