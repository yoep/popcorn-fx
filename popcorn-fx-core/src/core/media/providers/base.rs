use derive_more::Display;
use log::{debug, trace, warn};
use reqwest::{Client, Response, Url};
use reqwest::redirect::Policy;
use serde::de::DeserializeOwned;

use crate::core::media::{Genre, providers, SortBy};
use crate::core::media::providers::error::ProviderError;
use crate::core::media::providers::error::ProviderError::NoAvailableProviders;

const SORT_QUERY: &str = "sort";
const ORDER_QUERY: &str = "order";
const GENRE_QUERY: &str = "genre";
const KEYWORDS_QUERY: &str = "keywords";
const ORDER_QUERY_VALUE: &str = "-1";

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
            client: Client::builder()
                .redirect(Policy::limited(3))
                .build()
                .unwrap(),
            uri_providers: uris.into_iter()
                .map(|e| UriProvider::new(e))
                .collect(),
        }
    }

    /// Reset the api stats which will allow each known uri to be retried.
    pub fn reset_api_stats(&mut self) {
        for provider in self.uri_providers.iter_mut() {
            provider.reset();
        }
    }

    pub async fn retrieve_provider_page<T>(&mut self, resource: &str, genre: &Genre, sort: &SortBy, keywords: &String, page: i32) -> providers::Result<Vec<T>>
        where T : DeserializeOwned {
        let available_providers: Vec<&mut UriProvider> = self.uri_providers.iter_mut()
            .filter(|e| !e.disabled)
            .collect();

        if available_providers.is_empty() {
            warn!("No available uri providers found for resource {}", resource);
            return Err(NoAvailableProviders);
        }

        for provider in available_providers {
            trace!("Using provider {}", provider);
            let url = Self::create_uri(provider.uri(), resource, genre, sort, keywords, page);

            debug!("Requesting media from {}", &url);
            match self.client.clone().get(url).send().await {
                Ok(response) => return Self::handle_response::<T>(response).await,
                Err(err) => {
                    warn!("Failed to retrieve media data, {}", err);
                    provider.disable();
                }
            }
        }

        Err(NoAvailableProviders)
    }

    async fn handle_response<T>(response: Response) -> providers::Result<Vec<T>>
        where T : DeserializeOwned {
        let status_code = &response.status();

        if status_code.is_success() {
            match response.json::<Vec<T>>().await {
                Ok(e) => Ok(e),
                Err(e) => Err(ProviderError::ParsingFailed(e.to_string()))
            }
        } else {
            warn!("Request failed with {}, {}", response.status(), response.text().await.expect("expected the response body to be returned"));
            Err(ProviderError::RequestFailed(status_code.as_u16()))
        }
    }

    fn create_uri(host: &String, resource: &str, genre: &Genre, sort: &SortBy, keywords: &String, page: i32) -> Url {
        let uri = format!("{}/{}/{}", host, resource, page);
        let mut query_params: Vec<(&str, &str)> = vec![];

        query_params.push((SORT_QUERY, genre.key().as_str()));
        query_params.push((ORDER_QUERY, ORDER_QUERY_VALUE));
        query_params.push((GENRE_QUERY, genre.key().as_str()));
        query_params.push((SORT_QUERY, sort.key().as_str()));
        query_params.push((KEYWORDS_QUERY, keywords.as_str()));

        Url::parse_with_params(uri.as_str(), &query_params)
            .expect("Expected the provider uri to be valid")
    }
}

#[derive(Debug, Display)]
#[display(fmt = "uri: {}, disabled: {}, failed_attempts: {}", uri, disabled, failed_attempts)]
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

    fn reset(&mut self) {
        self.disabled = false;
        self.failed_attempts = 0;
    }

    fn disable(&mut self) {
        self.disabled = true;
        self.failed_attempts += 1;
    }

    fn uri(&self) -> &String {
        &self.uri
    }
}