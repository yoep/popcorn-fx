use std::fmt::Debug;

use async_trait::async_trait;
use derive_more::Display;
#[cfg(any(test, feature = "testing"))]
use mockall::mock;
use thiserror::Error;

use crate::core::{Callbacks, CoreCallback};
#[cfg(any(test, feature = "testing"))]
use crate::core::CallbackHandle;
use crate::core::media::MediaIdentifier;

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
    #[error("failed to retrieve token")]
    Token,
    /// Indicates that the authorization URI couldn't be opened.
    #[error("authorization uri couldn't be opened")]
    AuthorizationUriOpen,
}

/// Represents errors that can occur during tracking operations.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum TrackingError {
    /// The tracker provider is not authorized to execute the operation.
    #[error("tracker provider is not authorized to execute the operation")]
    Unauthorized,
    /// An error occurred while exchanging data with the tracker.
    #[error("an error occurred while exchanging data with the tracker")]
    Retrieval,
    /// An error occurred while parsing the tracking data.
    #[error("an error occurred while parsing the tracking data")]
    Parsing,
}

/// A type alias for a function that opens an authorization URI.
pub type OpenAuthorization = Box<dyn Fn(String) -> bool + Send + Sync>;

/// Type alias for the callback function for tracking events.
pub type TrackingCallback = CoreCallback<TrackingEvent>;

/// Represents events related to tracking.
#[derive(Debug, Clone, Display)]
pub enum TrackingEvent {
    /// Indicates a change in authorization state.
    #[display(fmt = "Authorization state changed to {}", _0)]
    AuthorizationStateChanged(bool),
}

/// The `TrackingProvider` trait allows tracking of watched media items with third-party media tracking providers.
#[async_trait]
pub trait TrackingProvider: Debug + Callbacks<TrackingEvent> + Send + Sync {
    /// Registers a callback function for opening authorization URIs.
    fn register_open_authorization(&self, open_callback: OpenAuthorization);

    /// Verify if this tracking provider has been authorized.
    ///
    /// # Returns
    ///
    /// Returns `true` when the user has authorized this tracker, otherwise `false`.
    fn is_authorized(&self) -> bool;

    /// Authorizes access to the tracking provider.
    async fn authorize(&self) -> Result<(), AuthorizationError>;

    /// Disconnects from the tracking provider.
    async fn disconnect(&self);

    /// Retrieves the list of watched movies from the tracking provider.
    ///
    /// # Returns
    ///
    /// Returns a vector of boxed `MediaIdentifier` instances representing watched movies.
    async fn watched_movies(&self) -> Result<Vec<Box<dyn MediaIdentifier>>, TrackingError>;
}

#[cfg(any(test, feature = "testing"))]
mock! {
    #[derive(Debug)]
    pub TrackingProvider {}
    
    #[async_trait]
    impl TrackingProvider for TrackingProvider {
        fn register_open_authorization(&self, open_callback: OpenAuthorization);
        fn is_authorized(&self) -> bool;
        async fn authorize(&self) -> Result<(), AuthorizationError>;
        async fn disconnect(&self);
        async fn watched_movies(&self) -> Result<Vec<Box<dyn MediaIdentifier>>, TrackingError>;
    }
    
    impl Callbacks<TrackingEvent> for TrackingProvider {
        fn add(&self, callback: CoreCallback<TrackingEvent>) -> CallbackHandle;
        fn remove(&self, handle: CallbackHandle);
    }    
}