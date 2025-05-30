use std::borrow::BorrowMut;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use async_trait::async_trait;
use itertools::*;
use log::{debug, info, warn};
use tokio::sync::Mutex;

use crate::core::cache::{CacheExecutionError, CacheManager};
use crate::core::config::ApplicationConfig;
use crate::core::media::providers::utils::available_uris;
use crate::core::media::providers::{BaseProvider, MediaDetailsProvider, MediaProvider};
use crate::core::media::{
    Category, Genre, MediaDetails, MediaError, MediaOverview, MediaType, ShowDetails, ShowOverview,
    SortBy,
};

const PROVIDER_NAME: &str = "series";
const SEARCH_RESOURCE_NAME: &str = "shows";
const DETAILS_RESOURCE_NAME: &str = "show";
const CACHE_NAME: &str = "shows";

/// The `ShowProvider` represents a media provider specifically designed for TV show media items.
///
/// This provider is responsible for retrieving details about TV show episodes, seasons, and other show-related information.
/// It is designed to work with the supported `Category` and `MediaType` for show media items.
///
/// # Cloning
///
/// Cloning the `ShowProvider` will create a new instance that shares the same configuration and base provider as the original.
/// This means that any modifications or disabled URIs in the original provider will be reflected in the cloned provider as well.
#[derive(Debug, Clone)]
pub struct ShowProvider {
    base: Arc<Mutex<BaseProvider>>,
    cache_manager: CacheManager,
}

impl ShowProvider {
    /// Creates a new `ShowProvider` instance.
    ///
    /// # Arguments
    ///
    /// * `settings` - The application settings for configuring the provider.
    /// * `cache_manager` - The cache manager for caching provider responses.
    /// * `insecure` - A flag indicating whether to allow insecure connections.
    ///
    /// # Returns
    ///
    /// A new `ShowProvider` instance.
    pub async fn new(
        settings: &ApplicationConfig,
        cache_manager: CacheManager,
        insecure: bool,
    ) -> Self {
        let uris = available_uris(&settings, PROVIDER_NAME).await;

        Self {
            base: Arc::new(Mutex::new(BaseProvider::new(uris, insecure))),
            cache_manager,
        }
    }

    /// Resets the internal API statistics of the provider.
    ///
    /// This method resets the API statistics of the underlying `BaseProvider`,
    /// allowing it to re-enable all disabled URIs.
    async fn internal_api_reset(&self) {
        let base_arc = &self.base.clone();
        let mut base = base_arc.lock().await;

        base.reset_api_stats();
    }
}

impl Display for ShowProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ShowProvider")
    }
}

#[async_trait]
impl MediaProvider for ShowProvider {
    fn supports(&self, category: &Category) -> bool {
        category == &Category::Series
    }

    async fn reset_api(&self) {
        self.internal_api_reset().await
    }

    async fn retrieve(
        &self,
        genre: &Genre,
        sort_by: &SortBy,
        keywords: &String,
        page: u32,
    ) -> crate::core::media::Result<Vec<Box<dyn MediaOverview>>> {
        let base_arc = &self.base.clone();
        let mut base = base_arc.lock().await;
        let cache_key = format!("{}-{}-{}-{}", genre, sort_by, keywords, page);

        self.cache_manager
            .operation()
            .name(CACHE_NAME)
            .key(cache_key)
            .options(BaseProvider::default_cache_options())
            .serializer()
            .execute(async move {
                match base
                    .borrow_mut()
                    .retrieve_provider_page::<ShowOverview>(
                        SEARCH_RESOURCE_NAME,
                        genre,
                        sort_by,
                        keywords,
                        page,
                    )
                    .await
                {
                    Ok(e) => {
                        debug!(
                            "Retrieved {} show(s), [{{{}}}]",
                            e.len(),
                            e.iter().map(|e| e.to_string()).join("}, {")
                        );
                        info!("Retrieved a total of {} shows for page {}", e.len(), page);
                        Ok(e)
                    }
                    Err(e) => {
                        warn!("Failed to retrieve show items, {}", e);
                        Err(e)
                    }
                }
            })
            .await
            .map(|e| {
                e.into_iter()
                    .map(|e| Box::new(e) as Box<dyn MediaOverview>)
                    .collect()
            })
            .map_err(|e| match e {
                CacheExecutionError::Operation(e) => e,
                CacheExecutionError::Mapping(e) => e,
                CacheExecutionError::Cache(e) => MediaError::ProviderParsingFailed(e.to_string()),
            })
    }
}

#[async_trait]
impl MediaDetailsProvider for ShowProvider {
    fn supports(&self, media_type: &MediaType) -> bool {
        media_type == &MediaType::Show
    }

    async fn reset_api(&self) {
        self.internal_api_reset().await
    }

    async fn retrieve_details(
        &self,
        imdb_id: &str,
    ) -> crate::core::media::Result<Box<dyn MediaDetails>> {
        let base_arc = &self.base.clone();
        self.cache_manager
            .operation()
            .name(CACHE_NAME)
            .key(imdb_id)
            .options(BaseProvider::default_cache_options())
            .serializer()
            .execute(async move {
                let mut base = base_arc.lock().await;
                match base
                    .borrow_mut()
                    .retrieve_details::<ShowDetails>(DETAILS_RESOURCE_NAME, imdb_id)
                    .await
                {
                    Ok(e) => {
                        debug!("Retrieved show details {}", &e);
                        Ok(e)
                    }
                    Err(e) => {
                        warn!("Failed to retrieve show details, {}", &e);
                        Err(e)
                    }
                }
            })
            .await
            .map(|e| Box::new(e) as Box<dyn MediaDetails>)
            .map_err(|e| match e {
                CacheExecutionError::Operation(e) => e,
                CacheExecutionError::Mapping(e) => e,
                CacheExecutionError::Cache(e) => MediaError::ProviderParsingFailed(e.to_string()),
            })
    }
}

#[cfg(test)]
mod test {
    use httpmock::Method::GET;

    use crate::core::cache::CacheManagerBuilder;
    use crate::core::media::MediaIdentifier;
    use crate::init_logger;
    use crate::test::start_mock_server;
    use crate::testing::read_test_file_to_string;

    use super::*;

    #[tokio::test]
    async fn test_retrieve() {
        init_logger!();
        let genre = Genre::all();
        let sort_by = SortBy::new("trending".to_string(), "".to_string());
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (server, settings) = start_mock_server(&temp_dir);
        server.mock(|when, then| {
            when.method(GET)
                .path("/shows/1")
                .query_param("sort", "trending".to_string())
                .query_param("order", "-1".to_string())
                .query_param("genre", "all".to_string())
                .query_param("keywords", "".to_string());
            then.status(200)
                .header("content-type", "application/json")
                .body(read_test_file_to_string("show-search.json"));
        });
        let cache_manager = CacheManagerBuilder::default()
            .storage_path(temp_path)
            .build();
        let provider = ShowProvider::new(&settings, cache_manager, false).await;

        let result = provider
            .retrieve(&genre, &sort_by, &String::new(), 1)
            .await
            .expect("expected no error to have occurred");

        assert!(result.len() > 0, "Expected media items to have been found")
    }

    #[tokio::test]
    async fn test_retrieve_details() {
        init_logger!();
        let imdb_id = "tt2861424".to_string();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (server, settings) = start_mock_server(&temp_dir);
        server.mock(|when, then| {
            when.method(GET).path("/show/tt2861424");
            then.status(200)
                .header("content-type", "application/json")
                .body(read_test_file_to_string("show-details.json"));
        });
        let cache_manager = CacheManagerBuilder::default()
            .storage_path(temp_path)
            .build();
        let provider = ShowProvider::new(&settings, cache_manager, false).await;

        let result = provider
            .retrieve_details(&imdb_id)
            .await
            .expect("expected the details to have been returned")
            .into_any()
            .downcast::<ShowDetails>()
            .expect("expected media to be a show");

        assert_eq!(imdb_id, result.imdb_id())
    }
}
