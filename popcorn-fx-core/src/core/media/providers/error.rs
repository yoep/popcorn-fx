use std::fmt::{Display, Formatter};

pub type Result<T> = std::result::Result<T, ProviderError>;

/// The errors which are thrown by the media providers.
#[derive(Debug)]
pub enum ProviderError {
    NoAvailableProviders,
    RequestFailed(u16),
    ParsingFailed(String),
    ProviderAlreadyExists(String),
    ProviderNotFound(String)
}

impl Display for ProviderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderError::NoAvailableProviders => write!(f, "there are no available providers to query"),
            ProviderError::RequestFailed(status) => write!(f, "request failed with status {}", status),
            ProviderError::ParsingFailed(error) => write!(f, "failed to parse response, {}", error),
            ProviderError::ProviderAlreadyExists(category) => write!(f, "a provider for {} is already registered", category),
            ProviderError::ProviderNotFound(category) => write!(f, "no provider could be found for {}", category),
        }
    }
}