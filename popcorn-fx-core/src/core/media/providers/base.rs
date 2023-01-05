use reqwest::{Client, ClientBuilder};
use reqwest::redirect::Policy;

/// A basic provider which provides common functionality for each provider.
/// It is meant to be used within other providers and not on it's own.
///
/// ```rust
/// use popcorn_fx_core::core::media::providers::BaseProvider;
/// 
/// struct MyProvider {
///   base: BaseProvider
/// }
///
/// impl MyProvider {
///     pub fn new(xxx: xxx) -> Self {
///         Self {
///             base: BaseProvider::new(xxx)
///         }
///     }
/// }
/// ```
#[derive(Debug)]
pub struct BaseProvider {
    client: Client,
    uri_providers: Vec<UriProvider>,
}

impl BaseProvider {
    /// Create a new base provider.
    /// * uris  - The available host uri's to use for this provider.
    pub fn new(uris: Vec<String>) -> Self {
        Self {
            client: ClientBuilder::new()
                .redirect(Policy::limited(3))
                .build()
                .unwrap(),
            uri_providers: uris.into_iter()
                .map(|e| UriProvider::new(e))
                .collect(),
        }
    }

    /// Reset the api stats which will allow each known uri to be retried.
    pub fn reset_api_stats(&self) {}
}

#[derive(Debug)]
struct UriProvider {
    uri: String,
    disabled: bool,
    failed_attempts: i32,
}

impl UriProvider {
    fn new(uri: String) -> Self {
        Self {
            uri,
            disabled: false,
            failed_attempts: 0,
        }
    }
}