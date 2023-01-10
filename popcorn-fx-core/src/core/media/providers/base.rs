use derive_more::Display;
use log::{debug, error, trace, warn};
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
                .expect("Client should have been created"),
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

    /// Retrieve the [T] for the given resource.
    /// The retrieval will try all known api's and disable the ones which are unavailable along the way.
    ///
    /// * [T] - The data model being returned.
    ///
    /// It returns an array of [T] items on success, else the [providers::ProviderError].
    pub async fn retrieve_provider_page<T>(&mut self, resource: &str, genre: &Genre, sort: &SortBy, keywords: &String, page: u32) -> providers::Result<Vec<T>>
        where T: DeserializeOwned {
        let client = self.client.clone();
        let available_providers: Vec<&mut UriProvider> = self.available_providers();

        if available_providers.is_empty() {
            warn!("No available uri providers found for resource {}", resource);
            return Err(NoAvailableProviders);
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
                    match client.get(url).send().await {
                        Ok(response) => return Self::handle_response::<Vec<T>>(response).await,
                        Err(err) => {
                            warn!("Failed to retrieve media data, {}", err);
                            provider.disable();
                        }
                    }
                }
            }
        }

        Err(NoAvailableProviders)
    }

    pub async fn retrieve_details<T>(&mut self, resource: &str, id: &String) -> providers::Result<T>
        where T: DeserializeOwned {
        let client = self.client.clone();
        let available_providers: Vec<&mut UriProvider> = self.available_providers();

        if available_providers.is_empty() {
            warn!("No available uri providers found for resource {}", resource);
            return Err(NoAvailableProviders);
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
                    match client.get(url).send().await {
                        Ok(response) => return Self::handle_response::<T>(response).await,
                        Err(err) => {
                            warn!("Failed to retrieve media details, {}", err);
                            provider.disable();
                        }
                    }
                }
            }
        }

        Err(NoAvailableProviders)
    }

    async fn handle_response<T>(response: Response) -> providers::Result<T>
        where T: DeserializeOwned {
        let status_code = &response.status();

        if status_code.is_success() {
            match response.json::<T>().await {
                Ok(e) => Ok(e),
                Err(e) => Err(ProviderError::ParsingFailed(e.to_string()))
            }
        } else {
            warn!("Request failed with {}, {}", response.status(), response.text().await.expect("expected the response body to be returned"));
            Err(ProviderError::RequestFailed(status_code.as_u16()))
        }
    }

    fn available_providers(&mut self) -> Vec<&mut UriProvider> {
        self.uri_providers.iter_mut()
            .filter(|e| !e.disabled)
            .collect()
    }

    fn create_search_uri(host: &String, resource: &str, genre: &Genre, sort: &SortBy, keywords: &String, page: u32) -> Option<Url> {
        let mut query_params: Vec<(&str, &str)> = vec![];

        query_params.push((ORDER_QUERY, ORDER_QUERY_VALUE));
        query_params.push((GENRE_QUERY, genre.key().as_str()));
        query_params.push((SORT_QUERY, sort.key().as_str()));
        query_params.push((KEYWORDS_QUERY, keywords.as_str()));

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

    fn create_details_uri(host: &String, resource: &str, id: &String) -> Option<Url> {
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

#[cfg(test)]
mod test {
    use crate::test::init_logger;

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
}