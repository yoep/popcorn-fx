use std::fmt::{Display, Formatter};

pub type Result<T> = std::result::Result<T, MediaError>;

/// The errors which are thrown by the media package.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MediaError {
    /// The favorite items failed to load
    FavoritesLoadingFailed(String),
    /// The requested favorite item couldn't be found
    FavoriteNotFound(String),
    /// There are no media providers available to query
    NoAvailableProviders,
    ProviderRequestFailed(u16),
    ProviderParsingFailed(String),
    ProviderAlreadyExists(String),
    /// No provider could be found for the requested category.
    ProviderNotFound(String),
}

impl Display for MediaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaError::FavoritesLoadingFailed(error) => write!(f, "favorites failed to load, {}", error),
            MediaError::FavoriteNotFound(id) => write!(f, "favorite with ID {} couldn't be found", id),
            MediaError::NoAvailableProviders => write!(f, "there are no available providers to query"),
            MediaError::ProviderRequestFailed(status) => write!(f, "request failed with status {}", status),
            MediaError::ProviderParsingFailed(error) => write!(f, "failed to parse response, {}", error),
            MediaError::ProviderAlreadyExists(category) => write!(f, "a provider for {} is already registered", category),
            MediaError::ProviderNotFound(category) => write!(f, "no provider could be found for {}", category),
        }
    }
}