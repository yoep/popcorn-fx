use crate::core::media::MediaIdentifier;
use async_trait::async_trait;
use derive_more::Display;
use fx_callback::Callback;
#[cfg(test)]
pub use mock::*;
use std::fmt::Debug;
use thiserror::Error;
use url::Url;

/// Represents errors that can occur during authorization.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum AuthorizationError {
    /// Indicates that CSRF validation failed.
    #[error("CSRF validation failed")]
    CsrfFailure,
    /// Indicates failure to retrieve the authorization code.
    #[error("failed to retrieve authorization code")]
    AuthorizationCode,
    /// Indicates failure to retrieve the authorization token.
    #[error("failed to retrieve authorization token")]
    Token,
    /// Indicates that the authorization process timed out.
    #[error("authorization timed out")]
    AuthorizationTimeout,
}

/// Represents errors that can occur during tracking operations.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum TrackingError {
    /// The tracker provider is not authorized to execute the operation.
    #[error("tracker provider is not authorized to execute the operation")]
    Unauthorized,
    /// An error occurred while exchanging data with the tracker.
    #[error("an error occurred while exchanging data with the tracker")]
    Request,
    /// An error occurred while parsing the tracking data.
    #[error("an error occurred while parsing the tracking data")]
    Parsing,
}

/// Represents events related to tracking.
#[derive(Debug, Clone, Display)]
pub enum TrackingEvent {
    /// Indicates a change in authorization state.
    #[display("Authorization state changed to {}", _0)]
    AuthorizationStateChanged(bool),
    /// Indicates a new authorization uri needs to be opened.
    #[display("Opening authorization uri {}", _0)]
    OpenAuthorization(Url),
}

/// The `TrackingProvider` trait allows tracking of watched media items with third-party media tracking providers.
#[async_trait]
pub trait TrackingProvider: Debug + Callback<TrackingEvent> + Send + Sync {
    /// Verify if this tracking provider has been authorized.
    ///
    /// # Returns
    ///
    /// Returns `true` when the user has authorized this tracker, otherwise `false`.
    async fn is_authorized(&self) -> bool;

    /// Authorizes access to the tracking provider.
    async fn authorize(&self) -> Result<(), AuthorizationError>;

    /// Disconnects from the tracking provider.
    async fn disconnect(&self);

    /// Adds watched movies to the tracking provider.
    ///
    /// # Arguments
    ///
    /// * `movie_ids` - A vector of movie IDs to add to watched list.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or a `TrackingError` on failure.
    async fn add_watched_movies(&self, movie_ids: Vec<String>) -> Result<(), TrackingError>;

    /// Retrieves the list of watched movies from the tracking provider.
    ///
    /// # Returns
    ///
    /// Returns a vector of boxed `MediaIdentifier` instances representing watched movies.
    async fn watched_movies(&self) -> Result<Vec<Box<dyn MediaIdentifier>>, TrackingError>;
}

#[cfg(test)]
mod mock {
    use super::*;
    use fx_callback::{Subscriber, Subscription};
    use mockall::mock;

    mock! {
        #[derive(Debug)]
        pub TrackingProvider {}

        #[async_trait]
        impl TrackingProvider for TrackingProvider {
            async fn is_authorized(&self) -> bool;
            async fn authorize(&self) -> Result<(), AuthorizationError>;
            async fn disconnect(&self);
            async fn add_watched_movies(&self, movie_ids: Vec<String>) -> Result<(), TrackingError>;
            async fn watched_movies(&self) -> Result<Vec<Box<dyn MediaIdentifier>>, TrackingError>;
        }

        impl Callback<TrackingEvent> for TrackingProvider {
            fn subscribe(&self) -> Subscription<TrackingEvent>;
            fn subscribe_with(&self, subscriber: Subscriber<TrackingEvent>);
        }
    }
}
