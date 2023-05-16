use thiserror::Error;

/// The media result type containing [MediaError] on failures.
pub type Result<T> = std::result::Result<T, MediaError>;

/// The errors thrown by the media package.
#[derive(Error, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MediaError {
    /// Failed to load the favorite items.
    #[error("Failed to load favorites: {0}")]
    FavoritesLoadingFailed(String),
    /// The requested favorite item couldn't be found.
    #[error("Favorite with ID {0} not found")]
    FavoriteNotFound(String),
    /// Failed to add a favorite item.
    #[error("Failed to add favorite for {0}: {1}")]
    FavoriteAddFailed(String, String),
    /// Failed to load the watched items.
    #[error("Failed to load watched items: {0}")]
    WatchedLoadingFailed(String),
    /// The given media item is not supported.
    #[error("Unsupported media type: {0}")]
    MediaTypeNotSupported(String),
    /// There are no available media providers to query.
    #[error("No available providers to query")]
    NoAvailableProviders,
    /// Failed to establish a connection with the media provider.
    #[error("Provider connection failed")]
    ProviderConnectionFailed,
    /// The request to the media provider failed with a specific status code.
    #[error("Request to {0} failed with status {1}")]
    ProviderRequestFailed(String, u16),
    /// Failed to parse the response from the media provider.
    #[error("Failed to parse response: {0}")]
    ProviderParsingFailed(String),
    /// A provider for a specific category is already registered.
    #[error("Provider for {0} already exists")]
    ProviderAlreadyExists(String),
    /// No provider could be found for the requested category.
    #[error("No provider found for {0}")]
    ProviderNotFound(String),
    /// Failed to load auto-resume data.
    #[error("Failed to load auto-resume data: {0}")]
    AutoResumeLoadingFailed(String),
}