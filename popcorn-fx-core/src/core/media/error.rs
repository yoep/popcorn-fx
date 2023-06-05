use thiserror::Error;

/// The media result type containing [MediaError] on failures.
pub type Result<T> = std::result::Result<T, MediaError>;

/// The errors thrown by the media package.
#[derive(Error, Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MediaError {
    /// Failed to load the favorite items.
    #[error("failed to load favorites: {0}")]
    FavoritesLoadingFailed(String),
    /// The requested favorite item couldn't be found.
    #[error("favorite with ID {0} not found")]
    FavoriteNotFound(String),
    /// Failed to add a favorite item.
    #[error("failed to add favorite for {0}: {1}")]
    FavoriteAddFailed(String, String),
    /// Failed to load the watched items.
    #[error("failed to load watched items: {0}")]
    WatchedLoadingFailed(String),
    /// The given media item is not supported.
    #[error("unsupported media type: {0}")]
    MediaTypeNotSupported(String),
    /// There are no available media providers to query.
    #[error("no available providers to query")]
    NoAvailableProviders,
    /// Failed to establish a connection with the media provider.
    #[error("provider connection failed")]
    ProviderConnectionFailed,
    /// The request to the media provider failed with a specific status code.
    #[error("request to {0} failed with status {1}")]
    ProviderRequestFailed(String, u16),
    /// Failed to parse the response from the media provider.
    #[error("failed to parse response: {0}")]
    ProviderParsingFailed(String),
    /// A provider for a specific category is already registered.
    #[error("provider for {0} already exists")]
    ProviderAlreadyExists(String),
    /// No provider could be found for the requested category.
    #[error("no provider found for {0}")]
    ProviderNotFound(String),
    /// Failed to load auto-resume data.
    #[error("failed to load auto-resume data: {0}")]
    AutoResumeLoadingFailed(String),
}