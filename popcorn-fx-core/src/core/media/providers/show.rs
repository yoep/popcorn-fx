use std::borrow::BorrowMut;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use async_trait::async_trait;
use itertools::*;
use log::{debug, info, warn};
use tokio::sync::Mutex;

use crate::core::config::ApplicationConfig;
use crate::core::media::{Category, Genre, MediaDetails, MediaOverview, ShowDetails, ShowOverview, SortBy};
use crate::core::media::providers::{BaseProvider, MediaProvider};
use crate::core::media::providers::utils::available_uris;

const PROVIDER_NAME: &str = "series";
const SEARCH_RESOURCE_NAME: &str = "shows";
const DETAILS_RESOURCE_NAME: &str = "show";

/// The [MediaProvider] for show media items.
#[derive(Debug)]
pub struct ShowProvider {
    base: Arc<Mutex<BaseProvider>>,
}

impl ShowProvider {
    pub fn new(settings: &Arc<ApplicationConfig>) -> Self {
        let uris = available_uris(settings, PROVIDER_NAME);

        Self {
            base: Arc::new(Mutex::new(BaseProvider::new(uris))),
        }
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
        category == &Category::SERIES
    }

    fn reset_api(&self) {
        let base_arc = &self.base.clone();
        let runtime = tokio::runtime::Runtime::new().expect("expected a runtime to have been created");
        let mut base = runtime.block_on(base_arc.lock());

        base.reset_api_stats();
    }

    async fn retrieve(&self, genre: &Genre, sort_by: &SortBy, keywords: &String, page: u32) -> crate::core::media::Result<Vec<Box<dyn MediaOverview>>> {
        let base_arc = &self.base.clone();
        let mut base = base_arc.lock().await;

        match base.borrow_mut().retrieve_provider_page::<ShowOverview>(SEARCH_RESOURCE_NAME, genre, sort_by, keywords, page).await {
            Ok(e) => {
                info!("Retrieved a total of {} shows, [{{{}}}]", e.len(), e.iter()
                .map(|e| e.to_string())
                .join("}, {"));
                let shows: Vec<Box<dyn MediaOverview>> = e.into_iter()
                    .map(|e| Box::new(e) as Box<dyn MediaOverview>)
                    .collect();

                Ok(shows)
            }
            Err(e) => {
                warn!("Failed to retrieve show items, {}", e);
                Err(e)
            }
        }
    }

    async fn retrieve_details(&self, imdb_id: &str) -> crate::core::media::Result<Box<dyn MediaDetails>> {
        let base_arc = &self.base.clone();
        let mut base = base_arc.lock().await;

        match base.borrow_mut().retrieve_details::<ShowDetails>(DETAILS_RESOURCE_NAME, imdb_id).await {
            Ok(e) => {
                debug!("Retrieved show details {}", &e);
                Ok(Box::new(e))
            }
            Err(e) => {
                warn!("Failed to retrieve show details, {}", &e);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::core::media::MediaIdentifier;
    use crate::testing::init_logger;

    use super::*;

    #[tokio::test]
    async fn test_retrieve() {
        init_logger();
        let genre = Genre::all();
        let sort_by = SortBy::new("trending".to_string(), "".to_string());
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = Arc::new(ApplicationConfig::new_auto(temp_path));
        let provider = ShowProvider::new(&settings);

        let result = provider.retrieve(&genre, &sort_by, &String::new(), 1)
            .await
            .expect("expected no error to have occurred");

        assert!(result.len() > 0, "Expected media items to have been found")
    }

    #[tokio::test]
    async fn test_retrieve_details() {
        init_logger();
        let imdb_id = "tt2861424".to_string();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = Arc::new(ApplicationConfig::new_auto(temp_path));
        let provider = ShowProvider::new(&settings);

        let result = provider.retrieve_details(&imdb_id)
            .await
            .expect("expected the details to have been returned")
            .into_any()
            .downcast::<ShowDetails>()
            .expect("expected media to be a show");

        assert_eq!(imdb_id, result.imdb_id())
    }
}