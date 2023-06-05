use log::{debug, trace, warn};

use crate::core::media;
use crate::core::media::{Category, Genre, MediaDetails, MediaError, MediaIdentifier, MediaOverview, MediaType, SortBy};
use crate::core::media::providers::{MediaDetailsProvider, MediaProvider};
use crate::core::media::providers::enhancers::Enhancer;

/// Manages the available [MediaProvider]'s that can be used to retrieve [Media] items.
/// Multiple providers for the same [Category] can be registered to overrule an existing one.
///
/// # Example new instance
///
/// Use the [ProviderManagerBuilder] to build new instance of this manager.
/// ```no_run
/// use popcorn_fx_core::core::media::providers::ProviderManagerBuilder;
/// let manager = ProviderManagerBuilder::new()
///     .with_provider(ProviderA::new())
///     .with_enhancer(EnhancerX::new())
///     .build();
/// ```
#[derive(Debug)]
pub struct ProviderManager {
    /// The media providers
    media_providers: Vec<Box<dyn MediaProvider>>,
    details_providers: Vec<Box<dyn MediaDetailsProvider>>,
    /// The enhancers
    enhancers: Vec<Box<dyn Enhancer>>,
}

impl ProviderManager {
    pub fn builder() -> ProviderManagerBuilder {
        ProviderManagerBuilder::new()
    }

    /// Retrieve a page of [MediaOverview] items based on the given criteria.
    /// The media items only contain basic information to present as an overview.
    ///
    /// It returns the retrieves page on success, else the [providers::ProviderError].
    pub async fn retrieve(&self, category: &Category, genre: &Genre, sort_by: &SortBy, keywords: &String, page: u32) -> media::Result<Vec<Box<dyn MediaOverview>>> {
        trace!("Retrieving provider for category {}", category);
        match self.provider(category) {
            None => Err(MediaError::ProviderNotFound(category.to_string())),
            Some(provider) => {
                trace!("Retrieving provider page {} for category {} with {:?}", page, category, provider);
                provider.retrieve(genre, sort_by, keywords, page).await
            }
        }
    }

    /// Retrieve the [MediaDetails] for the given IMDB ID item.
    /// The media item will contain all information for a media description and playback.
    ///
    /// It returns the details on success, else the [providers::ProviderError].
    pub async fn retrieve_details(&self, media: &Box<dyn MediaIdentifier>) -> media::Result<Box<dyn MediaDetails>> {
        let media_type = media.media_type();
        match self.details_provider(&media_type) {
            None => Err(MediaError::ProviderNotFound(media_type.to_string())),
            Some(provider) => {
                match provider.retrieve_details(media.imdb_id()).await {
                    Ok(media) => {
                        Ok(self.enhance_media_item(&Category::from(media_type), media).await)
                    }
                    Err(e) => Err(e)
                }
            }
        }
    }

    /// Reset the api statics and re-enable all disabled api's.
    pub fn reset_api(&self, category: &Category) {
        trace!("Starting reset of api provider for category {}", category);
        match self.provider(category) {
            None => {
                warn!("Unable to reset api, category {} is not supported", category)
            }
            Some(provider) => {
                debug!("Resetting api provider {}", provider);
                provider.reset_api()
            }
        }
    }

    async fn enhance_media_item(&self, category: &Category, mut media: Box<dyn MediaDetails>) -> Box<dyn MediaDetails> {
        for enhancer in self.enhancers.iter().filter(|e| e.supports(category)) {
            debug!("Enhancing media item {} with {}", media.imdb_id(), enhancer);
            media = enhancer.enhance_details(media).await;
        }

        media
    }

    /// Retrieves the `MediaProvider` for the given `Category`.
    ///
    /// # Arguments
    ///
    /// * `category` - The `Category` for which to retrieve the `MediaProvider`.
    ///
    /// # Returns
    ///
    /// The `MediaProvider` if one is registered for the given `Category`, otherwise `None`.
    fn provider<'a>(&'a self, category: &Category) -> Option<&'a Box<dyn MediaProvider>> {
        self.media_providers.iter()
            .find(|&provider| provider.supports(category))
    }

    /// Retrieves the `MediaDetailsProvider` for the given `MediaType`.
    ///
    /// # Arguments
    ///
    /// * `media_type` - The `MediaType` for which to retrieve the `MediaDetailsProvider`.
    ///
    /// # Returns
    ///
    /// The `MediaDetailsProvider` if one is registered for the given `MediaType`, otherwise `None`.
    fn details_provider<'a>(&'a self, media_type: &MediaType) -> Option<&'a Box<dyn MediaDetailsProvider>> {
        self.details_providers.iter()
            .find(|&provider| provider.supports(media_type))
    }
}

unsafe impl Send for ProviderManager {}

unsafe impl Sync for ProviderManager {}

/// The builder for the [ProviderManager] instance.
#[derive(Debug, Default)]
pub struct ProviderManagerBuilder {
    media_providers: Vec<Box<dyn MediaProvider>>,
    details_providers: Vec<Box<dyn MediaDetailsProvider>>,
    enhancers: Vec<Box<dyn Enhancer>>,
}

impl ProviderManagerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_provider(mut self, provider: Box<dyn MediaProvider>) -> Self {
        self.media_providers.push(provider);
        self
    }

    pub fn with_details_provider(mut self, details_provider: Box<dyn MediaDetailsProvider>) -> Self {
        self.details_providers.push(details_provider);
        self
    }

    pub fn with_enhancer(mut self, enhancer: Box<dyn Enhancer>) -> Self {
        self.enhancers.push(enhancer);
        self
    }

    pub fn build(self) -> ProviderManager {
        ProviderManager {
            media_providers: self.media_providers,
            details_providers: self.details_providers,
            enhancers: self.enhancers,
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use tokio::runtime::Runtime;
    use tokio::sync::Mutex;

    use crate::core::config::ApplicationConfig;
    use crate::core::media::{Episode, ShowDetails, ShowOverview};
    use crate::core::media::providers::enhancers::MockEnhancer;
    use crate::core::media::providers::MockMediaDetailsProvider;
    use crate::core::media::providers::ShowProvider;
    use crate::testing::init_logger;

    use super::*;

    #[tokio::test]
    async fn test_retrieve_when_provider_not_found() {
        let sort_by = SortBy::new(String::new(), String::new());
        let manager = ProviderManagerBuilder::new()
            .build();

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
        let settings = Arc::new(Mutex::new(ApplicationConfig::builder()
            .storage(temp_path)
            .build()));
        let provider: Box<dyn MediaProvider> = Box::new(ShowProvider::new(&settings, false));
        let manager = ProviderManagerBuilder::new()
            .with_provider(provider)
            .build();

        let result = manager.provider(&Category::Series);

        assert!(result.is_some(), "Expected a supported provider to have been found")
    }

    #[test]
    fn test_get_not_supported_category() {
        let manager = ProviderManagerBuilder::new().build();

        let result = manager.provider(&Category::Movies);

        assert!(result.is_none(), "Expected no supported provider to have been found")
    }

    #[test]
    fn test_enhance_details() {
        init_logger();
        let imdb_id = "tt000001";
        let thumb = "http://localhost/thumb.png";
        let media_identifier = Box::new(ShowOverview {
            imdb_id: imdb_id.to_string(),
            tvdb_id: "".to_string(),
            title: "".to_string(),
            year: "".to_string(),
            num_seasons: 0,
            images: Default::default(),
            rating: None,
        }) as Box<dyn MediaIdentifier>;
        let mut provider = MockMediaDetailsProvider::new();
        provider.expect_supports()
            .returning(|e: &MediaType| e == &MediaType::Show);
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
        let manager = ProviderManager::builder()
            .with_details_provider(Box::new(provider))
            .with_enhancer(Box::new(enhancer))
            .build();
        let runtime = Runtime::new().unwrap();

        let media = runtime.block_on(manager.retrieve_details(&media_identifier))
            .expect("expected a media item to be returned")
            .into_any()
            .downcast::<ShowDetails>()
            .unwrap();

        let episode = media.episodes.get(0).expect("expected at least one episode");
        assert_eq!(Some(thumb.to_string()), episode.thumb)
    }
}