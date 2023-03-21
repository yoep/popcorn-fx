use thiserror::Error;

/// The media result type containing [MediaError] on failures.
pub type Result<T> = std::result::Result<T, MediaError>;

/// The errors which are thrown by the media package.
#[derive(Error, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MediaError {
    /// The favorite items failed to load
    #[error("favorites failed to load, {0}")]
    FavoritesLoadingFailed(String),
    /// The requested favorite item couldn't be found
    #[error("favorite with ID {0} couldn't be found")]
    FavoriteNotFound(String),
    #[error("failed to add favorite for {0}, {1}")]
    FavoriteAddFailed(String, String),
    /// The watched items failed to load
    #[error("watched failed to load, {0}")]
    WatchedLoadingFailed(String),
    /// The given media item is not supported
    #[error("media type of {0} is unsupported")]
    MediaTypeNotSupported(String),
    /// There are no media providers available to query
    #[error("there are no available providers to query")]
    NoAvailableProviders,
    #[error("The provider request connection failed")]
    ProviderConnectionFailed,
    #[error("request failed with status {0}")]
    ProviderRequestFailed(u16),
    #[error("failed to parse response, {0}")]
    ProviderParsingFailed(String),
    #[error("a provider for {0} is already registered")]
    ProviderAlreadyExists(String),
    /// No provider could be found for the requested category.
    #[error("no provider could be found for {0}")]
    ProviderNotFound(String),
    #[error("auto-resume failed to load, {0}")]
    AutoResumeLoadingFailed(String),
}