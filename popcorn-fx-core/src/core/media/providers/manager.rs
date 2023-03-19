use std::sync::Arc;

use log::{debug, warn};

use crate::core::media;
use crate::core::media::{Category, Genre, MediaDetails, MediaError, MediaOverview, SortBy};
use crate::core::media::providers::enhancers::Enhancer;
use crate::core::media::providers::MediaProvider;

/// Manages available [MediaProvider]'s that can be used to retrieve [Media] items.
/// Multiple providers for the same [Category] can be registered to overrule an existing one.
#[derive(Debug, Default)]
pub struct ProviderManager {
    /// The media providers
    providers: Vec<Arc<Box<dyn MediaProvider>>>,
    /// The enhancers
    enhancers: Vec<Arc<Box<dyn Enhancer>>>,
}

impl ProviderManager {
    /// Add the media providers used by this manager.
    /// The [Arc] instances should be owned by this manager.
    pub fn with_providers(mut self, providers: Vec<Arc<Box<dyn MediaProvider>>>) -> Self {
        self.providers = providers;
        self
    }

    /// Add the media item enhancers to this manager.
    /// The [Arc] instances should be owned by this manager.
    ///
    /// Each enhancer is invoked in the same order as given within this array.
    /// This means that multiple enhancers for the same [Category] can be added and will be applied by this manager when needed.
    pub fn with_enhancers(mut self, enhancers: Vec<Arc<Box<dyn Enhancer>>>) -> Self {
        self.enhancers = enhancers;
        self
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
    pub async fn retrieve_details(&self, category: &Category, imdb_id: &str) -> media::Result<Box<dyn MediaDetails>> {
        match self.provider(category) {
            None => Err(MediaError::ProviderNotFound(category.to_string())),
            Some(provider) => {
                match provider.retrieve_details(imdb_id).await {
                    Ok(media) => {
                        Ok(self.enhance_media_item(category, media).await)
                    }
                    Err(e) => Err(e)
                }
            }
        }
    }

    /// Reset the api statics and re-enable all disabled api's.
    pub fn reset_api(&self, category: &Category) {
        match self.provider(category) {
            None => {
                warn!("Unable to reset api, category {} is not supported", category)
            }
            Some(provider) => {
                provider.reset_api()
            }
        }
    }

    async fn enhance_media_item(&self, category: &Category, mut media: Box<dyn MediaDetails>) -> Box<dyn MediaDetails> {
        for enhancer in self.enhancers.iter().filter(|e| e.supports(category)) {
            debug!("Enhancing media item {} with {:?}", media.imdb_id(), enhancer);
            media = enhancer.enhance_details(media).await;
        }

        media
    }

    /// Retrieve the [MediaProvider] for the given [Category].
    ///
    /// It returns the [MediaProvider] if one is registered, else [None].
    fn provider<'a>(&'a self, category: &Category) -> Option<&'a Arc<Box<dyn MediaProvider>>> {
        self.providers.iter()
            .find(|&provider| provider.supports(category))
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use tokio::runtime::Runtime;
    use tokio::sync::Mutex;

    use crate::core::config::ApplicationConfig;
    use crate::core::media::{Episode, ShowDetails};
    use crate::core::media::providers::enhancers::MockEnhancer;
    use crate::core::media::providers::MockMediaProvider;
    use crate::core::media::providers::ShowProvider;

    use super::*;

    #[tokio::test]
    async fn test_retrieve_when_provider_not_found() {
        let sort_by = SortBy::new(String::new(), String::new());
        let manager = ProviderManager::default()
            .with_providers(vec![]);

        let result = manager.retrieve(&Category::Movies, &Genre::all(), &sort_by, &String::new(), 1)
            .await;

        assert!(result.is_err(), "Expected the provider to return an error");
        match result.err().unwrap() {
            MediaError::ProviderNotFound(category) => assert_eq!(Category::Movies.to_string(), category.to_string()),
            _ => assert!(false, "Expected error MediaError::ProviderNotFound")
        }
    }

    #[test]
    fn test_get_supported_category() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = Arc::new(Mutex::new(ApplicationConfig::new_auto(temp_path)));
        let provider: Box<dyn MediaProvider> = Box::new(ShowProvider::new(&settings));
        let manager = ProviderManager::default()
            .with_providers(vec![Arc::new(provider)]);

        let result = manager.provider(&Category::Series);

        assert!(result.is_some(), "Expected a supported provider to have been found")
    }

    #[test]
    fn test_get_not_supported_category() {
        let manager = ProviderManager::default();

        let result = manager.provider(&Category::Movies);

        assert!(result.is_none(), "Expected no supported provider to have been found")
    }

    #[test]
    fn test_enhance_details() {
        let thumb = "http://localhost/thumb.png";
        let mut provider = MockMediaProvider::new();
        provider.expect_supports()
            .returning(|e: &Category| e == &Category::Series);
        provider.expect_retrieve_details()
            .returning(|imdb_id: &str|
                Ok(Box::new(ShowDetails {
                    imdb_id: imdb_id.to_string(),
                    tvdb_id: "".to_string(),
                    title: "".to_string(),
                    year: "".to_string(),
                    num_seasons: 0,
                    images: Default::default(),
                    rating: None,
                    context_locale: "".to_string(),
                    synopsis: "".to_string(),
                    runtime: "".to_string(),
                    status: "".to_string(),
                    genres: vec![],
                    episodes: vec![
                        Episode {
                            season: 2,
                            episode: 1,
                            first_aired: 0,
                            title: "".to_string(),
                            overview: "".to_string(),
                            tvdb_id: 392256,
                            tvdb_id_value: "392256".to_string(),
                            thumb: None,
                            torrents: Default::default(),
                        }
                    ],
                    liked: None,
                })));
        let mut enhancer = MockEnhancer::new();
        enhancer.expect_supports()
            .returning(|category: &Category| category == &Category::Series);
        enhancer.expect_enhance_details()
            .returning(|e: Box<dyn MediaDetails>| {
                let mut show = e.into_any()
                    .downcast::<ShowDetails>()
                    .unwrap();
                show.episodes.get_mut(0).unwrap().thumb = Some(thumb.to_string());
                show
            });
        let manager = ProviderManager::default()
            .with_providers(vec![
                Arc::new(Box::new(provider))
            ])
            .with_enhancers(vec![
                Arc::new(Box::new(enhancer))
            ]);
        let runtime = Runtime::new().unwrap();

        let media = runtime.block_on(manager.retrieve_details(&Category::Series, "tt3581920"))
            .expect("expected a media item to be returned")
            .into_any()
            .downcast::<ShowDetails>()
            .unwrap();

        let episode = media.episodes.get(0).expect("expected at least one episode");
        assert_eq!(Some(thumb.to_string()), episode.thumb)
    }
}