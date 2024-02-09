use std::thread;

use chrono::Duration;
use derive_more::Display;
use log::{debug, error, trace, warn};
use reqwest::{Client, Response, Url};
use reqwest::redirect::Policy;
use serde::de::DeserializeOwned;

use crate::core::cache::{CacheOptions, CacheType};
use crate::core::media::{Genre, MediaError, SortBy};

const SORT_QUERY: &str = "sort";
const ORDER_QUERY: &str = "order";
const GENRE_QUERY: &str = "genre";
const KEYWORDS_QUERY: &str = "keywords";
const ORDER_QUERY_VALUE: &str = "-1";

/// A basic provider which provides common functionality for each provider.
/// It is meant to be used within other providers and not on it's own.
///
/// ```no_run
/// use popcorn_fx_core::core::media::providers::BaseProvider;
///
/// struct MyProvider {
///   base: BaseProvider,
/// }
///
/// impl MyProvider {
///     pub fn new(xxx: xxx) -> Self {
///         Self {
///             base: BaseProvider::new(xxx, false)
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
    ///
    /// # Arguments
    ///
    /// * `uris` - The available host URIs to use for this provider.
    /// * `insecure` - A flag indicating whether to accept invalid certificates.
    ///
    /// # Returns
    ///
    /// A new `BaseProvider` instance.
    pub fn new(uris: Vec<String>, insecure: bool) -> Self {
        Self {
            client: Client::builder()
                .redirect(Policy::limited(3))
                .danger_accept_invalid_certs(insecure)
                .build()
                .expect("Client should have been created"),
            uri_providers: uris.into_iter()
                .map(UriProvider::new)
                .collect(),
        }
    }

    /// Reset the api stats which will allow each known uri to be retried.
    pub fn reset_api_stats(&mut self) {
        for provider in self.uri_providers.iter_mut() {
            provider.reset();
        }
    }

    /// Retrieve the `[T]` for the given resource.
    /// The retrieval will try all known APIs and disable the ones which are unavailable along the way.
    ///
    /// # Arguments
    ///
    /// * `resource` - The resource to retrieve.
    /// * `genre` - The genre of the resource.
    /// * `sort` - The sorting criteria for the retrieved data.
    /// * `keywords` - The search keywords.
    /// * `page` - The page number.
    ///
    /// # Returns
    ///
    /// An array of `[T]` items on success, or a `providers::ProviderError` if there was an error.
    pub async fn retrieve_provider_page<T>(&mut self, resource: &str, genre: &Genre, sort: &SortBy, keywords: &String, page: u32) -> crate::core::media::Result<Vec<T>>
        where T: DeserializeOwned {
        let client = self.client.clone();
        let available_providers: Vec<&mut UriProvider> = self.available_providers();

        if available_providers.is_empty() {
            warn!("No available uri providers found for resource {}", resource);
            return Err(MediaError::NoAvailableProviders);
        }

        for provider in available_providers {
            trace!("Using search provider {}", provider);
            match Self::create_search_uri(provider.uri(), resource, genre, sort, keywords, page) {
                None => {
                    debug!("Disabling invalid provider {}", provider);
                    provider.disable();
                }
                Some(url) => {
                    debug!("Searching media at {}", &url);
                    match Self::send_request_with_provider(&client, &url, provider).await {
                        None => {}
                        Some(e) => return e,
                    }
                }
            }
        }

        Err(MediaError::NoAvailableProviders)
    }

    /// Retrieve details for the given resource.
    ///
    /// # Arguments
    ///
    /// * `resource` - The resource to retrieve details for.
    /// * `id` - The ID of the resource.
    ///
    /// # Returns
    ///
    /// The details of the resource, or a `providers::ProviderError` if there was an error.
    pub async fn retrieve_details<T>(&mut self, resource: &str, id: &str) -> crate::core::media::Result<T>
        where T: DeserializeOwned {
        let client = self.client.clone();
        let available_providers: Vec<&mut UriProvider> = self.available_providers();

        if available_providers.is_empty() {
            warn!("No available uri providers found for resource {}", resource);
            return Err(MediaError::NoAvailableProviders);
        }

        for provider in available_providers {
            trace!("Using details provider {}", provider);
            match Self::create_details_uri(provider.uri(), resource, id) {
                None => {
                    debug!("Disabling invalid provider {}", provider);
                    provider.disable();
                }
                Some(url) => {
                    debug!("Fetching details from {}", &url);
                    match Self::send_request_with_provider(&client, &url, provider).await {
                        None => {}
                        Some(e) => return e,
                    }
                }
            }
        }

        Err(MediaError::NoAvailableProviders)
    }

    /// Get the default cache options.
    ///
    /// # Returns
    ///
    /// The default `CacheOptions` instance.
    pub fn default_cache_options() -> CacheOptions {
        CacheOptions {
            cache_type: CacheType::CacheLast,
            expires_after: Duration::days(7),
        }
    }

    async fn send_request_with_provider<T>(client: &Client, url: &Url, provider: &mut UriProvider) -> Option<crate::core::media::Result<T>>
        where T: DeserializeOwned {
        while !provider.disabled {
            match Self::send_request::<T>(&client, &url).await {
                // if we got an OK, return instantly the result
                Ok(e) => return Some(Ok(e)),
                // if we got an error, we check what kind of error it is
                Err(e) => {
                    trace!("Provider {} returned an error", provider);
                    match e {
                        // if it's a connection error, instantly disable the provider
                        MediaError::ProviderConnectionFailed => provider.disable(),
                        // any other error might be temporary such as 502
                        // so we increase the failed attempts and try again
                        _ => {
                            let delay = std::time::Duration::from_millis(500);
                            trace!("Request was unsuccessful, retrying in {} millis", delay.as_millis());
                            thread::sleep(delay);
                            provider.increase_failure()
                        }
                    }
                }
            }
        }

        None
    }

    async fn send_request<T>(client: &Client, url: &Url) -> crate::core::media::Result<T>
        where T: DeserializeOwned {
        match client.get(url.clone()).send().await {
            Ok(response) => {
                Self::handle_response::<T>(response, url).await
            }
            Err(err) => {
                warn!("Failed to retrieve media details, {}", err);
                Err(MediaError::ProviderConnectionFailed)
            }
        }
    }

    async fn handle_response<T>(response: Response, url: &Url) -> crate::core::media::Result<T>
        where T: DeserializeOwned {
        let status_code = &response.status();

        if status_code.is_success() {
            match response.json::<T>().await {
                Ok(e) => Ok(e),
                Err(e) => Err(MediaError::ProviderParsingFailed(e.to_string()))
            }
        } else {
            warn!("Request {} failed with status {}, {}", url.as_str(), response.status(), response.text().await.expect("expected the response body to be returned"));
            Err(MediaError::ProviderRequestFailed(url.to_string(), status_code.as_u16()))
        }
    }

    fn available_providers(&mut self) -> Vec<&mut UriProvider> {
        self.uri_providers.iter_mut()
            .filter(|e| !e.disabled)
            .collect()
    }

    fn create_search_uri(host: &String, resource: &str, genre: &Genre, sort: &SortBy, keywords: &str, page: u32) -> Option<Url> {
        let mut query_params: Vec<(&str, &str)> = vec![];

        query_params.push((ORDER_QUERY, ORDER_QUERY_VALUE));
        query_params.push((GENRE_QUERY, genre.key()));
        query_params.push((SORT_QUERY, sort.key()));
        query_params.push((KEYWORDS_QUERY, keywords));

        match Url::parse_with_params(host.as_str(), &query_params) {
            Ok(mut e) => {
                trace!("Creating search url for host: {}, resource: {}, page: {}", host, resource, page);
                e.path_segments_mut().expect("segments should be mutable")
                    .pop_if_empty()
                    .push(resource)
                    .push(&page.to_string());

                Some(e)
            }
            Err(e) => {
                error!("Host api \"{}\" is invalid, {}", host, e);
                None
            }
        }
    }

    fn create_details_uri(host: &String, resource: &str, id: &str) -> Option<Url> {
        match Url::parse(host.as_str()) {
            Ok(mut e) => {
                trace!("Creating details url for host: {}, resource: {}, id: {}", host, resource, id);
                e.path_segments_mut().expect("segments should be mutable")
                    .pop_if_empty()
                    .push(resource)
                    .push(id);

                Some(e)
            }
            Err(e) => {
                error!("Host api \"{}\" is invalid, {}", host, e);
                None
            }
        }
    }
}

#[derive(Debug, Clone, Display)]
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

    fn increase_failure(&mut self) {
        self.failed_attempts += 1;
        trace!("Provider {} failures increased to {}", self.uri, self.failed_attempts);
        if self.failed_attempts == 3 {
            self.disable()
        }
    }

    fn reset(&mut self) {
        self.disabled = false;
        self.failed_attempts = 0;
    }

    fn disable(&mut self) {
        debug!("Disabling uri provider {}", self);
        self.disabled = true;
        self.failed_attempts += 1;
    }

    fn uri(&self) -> &String {
        &self.uri
    }
}

#[cfg(test)]
mod test {
    use httpmock::Method::GET;
    use httpmock::MockServer;

    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_create_search_uri() {
        init_logger();
        let host = "https://lorem.com/api/v1/".to_string();
        let resource = "movies";
        let genre = Genre::all();
        let sort_by = SortBy::new("trending".to_string(), String::new());
        let keywords = "pirates".to_string();
        let page = 2;
        let expected_result = "https://lorem.com/api/v1/movies/2?order=-1&genre=all&sort=trending&keywords=pirates";

        let result = BaseProvider::create_search_uri(&host, resource, &genre, &sort_by, &keywords, page)
            .expect("Expected the created url to be valid");

        assert_eq!(expected_result, result.as_str())
    }

    #[test]
    fn test_create_details_uri() {
        init_logger();
        let host = "https://lorem.com/api/v1/".to_string();
        let resource = "movie";
        let id = "tt9764362".to_string();
        let expected_result = "https://lorem.com/api/v1/movie/tt9764362";

        let result = BaseProvider::create_details_uri(&host, resource, &id)
            .expect("Expected the created url to be valid");

        assert_eq!(expected_result, result.as_str())
    }

    #[tokio::test]
    async fn test_handle_failed_response() {
        init_logger();
        let path = "/error";
        let status_code = 503;
        let server = MockServer::start();
        server.mock(|mock, then| {
            mock.method(GET)
                .path(path);
            then.status(status_code);
        });
        let url = Url::parse(server.url(path).as_str()).unwrap();
        let provider = BaseProvider::new(vec![server.url("")], false);

        let response = provider.client.get(url.clone())
            .send()
            .await
            .unwrap();

        let result = BaseProvider::handle_response::<()>(response, &url).await;

        if let Err(e) = result {
            assert_eq!(MediaError::ProviderRequestFailed(url.to_string(), status_code), e);
        } else {
            assert!(false, "expected a MediaError to be returned");
        }
    }
}