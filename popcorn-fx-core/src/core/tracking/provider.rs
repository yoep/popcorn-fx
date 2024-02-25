use std::fmt::Debug;

use async_trait::async_trait;
use derive_more::Display;
#[cfg(any(test, feature = "testing"))]
use mockall::mock;
use thiserror::Error;

use crate::core::{Callbacks, CoreCallback};
#[cfg(any(test, feature = "testing"))]
use crate::core::CallbackHandle;

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

/// A type alias for a function that opens an authorization URI.
pub type OpenAuthorization = Box<dyn Fn(String) -> bool + Send + Sync>;

pub type TrackingCallback = CoreCallback<TrackingEvent>;

#[derive(Debug, Clone, Display)]
pub enum TrackingEvent {
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
    /// It returns `true` when the user has authorized this tracker, else `false`.
    fn is_authorized(&self) -> bool;

    /// Authorizes access to the tracking provider.
    async fn authorize(&self) -> Result<(), AuthorizationError>;
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
    }
    
    impl Callbacks<TrackingEvent> for TrackingProvider {
        fn add(&self, callback: CoreCallback<TrackingEvent>) -> CallbackHandle;
        fn remove(&self, handle: CallbackHandle);
    }    
}