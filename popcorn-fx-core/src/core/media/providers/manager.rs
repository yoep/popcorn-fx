use std::sync::Arc;

use log::warn;

use crate::core::media;
use crate::core::media::{Category, Genre, MediaDetails, MediaError, MediaOverview, SortBy};
use crate::core::media::providers::MediaProvider;

/// Manages available [MediaProvider]'s that can be used to retrieve [Media] items.
/// Multiple providers for the same [Category] can be registered to overrule an existing one.
#[derive(Debug)]
pub struct ProviderManager {
    providers: Vec<Arc<Box<dyn MediaProvider>>>,
}

impl ProviderManager {
    /// Create a new manager for [MediaProvider]'s which is empty.
    /// This manager won't support anything out-of-the-box.
    ///
    /// If you want to create an instance with providers, use [ProviderManager::with_providers] instead.
    pub fn new() -> Self {
        Self {
            providers: vec![]
        }
    }

    /// Create a new manager which the given [MediaProvider]'s.
    /// The [Arc] reference counter is owned by this manager.
    pub fn with_providers(providers: Vec<Arc<Box<dyn MediaProvider>>>) -> Self {
        Self {
            providers
        }
    }

    /// Retrieve a page of [MediaOverview] items based on the given criteria.
    /// The media items only contain basic information to present as an overview.
    ///
    /// It returns the retrieves page on success, else the [providers::ProviderError].
    pub async fn retrieve(&self, category: &Category, genre: &Genre, sort_by: &SortBy, keywords: &String, page: u32) -> media::Result<Vec<Box<dyn MediaOverview>>> {
        match self.provider(category) {
            None => Err(MediaError::ProviderNotFound(category.to_string())),
            Some(provider) => {
                provider.retrieve(genre, sort_by, keywords, page).await
            }
        }
    }

    /// Retrieve the [MediaDetails] for the given IMDB ID item.
    /// The media item will contain all information for a media description and playback.
    ///
    /// It returns the details on success, else the [providers::ProviderError].
    pub async fn retrieve_details(&self, category: &Category, imdb_id: &String) -> media::Result<Box<dyn MediaDetails>> {
        match self.provider(category) {
            None => Err(MediaError::ProviderNotFound(category.to_string())),
            Some(provider) => {
                provider.retrieve_details(imdb_id).await
            }
        }
    }

    /// Reset the api statics and re-enable all disabled api's.
    pub fn reset_api(&self, category: &Category) {
        match self.provider(category) {
            None => {
                warn!("Unable to reset api, category {} is not supported", category)
            },
            Some(provider) => {
                provider.reset_api()
            }
        }
    }

    /// Retrieve the [MediaProvider] for the given [Category].
    ///
    /// It returns the [MediaProvider] if one is registered, else [None].
    fn provider(&self, category: &Category) -> Option<&Box<dyn MediaProvider>> {
        for provider in &self.providers {
            if provider.supports(category) {
                return Some(provider);
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::core::config::Application;
    use crate::core::media::providers::ShowProvider;

    use super::*;

    #[tokio::test]
    async fn test_retrieve_when_provider_not_found() {
        let sort_by = SortBy::new(String::new(), String::new());
        let manager = ProviderManager::with_providers(vec![]);

        let result = manager.retrieve(&Category::MOVIES, &Genre::all(), &sort_by, &String::new(), 1)
            .await;

        assert!(result.is_err(), "Expected the provider to return an error");
        match result.err().unwrap() {
            MediaError::ProviderNotFound(category) => assert_eq!(Category::MOVIES.to_string(), category.to_string()),
            _ => assert!(false, "Expected error MediaError::ProviderNotFound")
        }
    }

    #[test]
    fn test_get_supported_category() {
        let settings = Arc::new(Application::default());
        let provider: Box<dyn MediaProvider> = Box::new(ShowProvider::new(&settings));
        let manager = ProviderManager::with_providers(vec![Arc::new(provider)]);

        let result = manager.provider(&Category::SERIES);

        assert!(result.is_some(), "Expected a supported provider to have been found")
    }

    #[test]
    fn test_get_not_supported_category() {
        let manager = ProviderManager::new();

        let result = manager.provider(&Category::MOVIES);

        assert!(result.is_none(), "Expected no supported provider to have been found")
    }
}