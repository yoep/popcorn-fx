use std::borrow::BorrowMut;
use std::sync::Arc;

use async_trait::async_trait;
use itertools::*;
use log::{debug, info, warn};
use tokio::sync::Mutex;

use crate::core::config::Application;
use crate::core::media::{Category, Genre, Media, Movie, providers, SortBy};
use crate::core::media::providers::{BaseProvider, MediaProvider};
use crate::core::media::providers::utils::available_uris;
use crate::core::Page;

const PROVIDER_NAME: &str = "movies";
const SEARCH_RESOURCE_NAME: &str = "movies";
const DETAILS_RESOURCE_NAME: &str = "movie";

#[derive(Debug)]
pub struct MovieProvider {
    base: Arc<Mutex<BaseProvider>>,
}

impl MovieProvider {
    pub fn new(settings: &Arc<Application>) -> Self {
        let uris = available_uris(settings, PROVIDER_NAME);

        Self {
            base: Arc::new(Mutex::new(BaseProvider::new(uris))),
        }
    }
}

#[async_trait]
impl MediaProvider for MovieProvider {
    fn supports(&self, category: &Category) -> bool {
        category == &Category::MOVIES
    }

    fn reset_api(&self) {
        let base_arc = &self.base.clone();
        let runtime = tokio::runtime::Runtime::new().expect("expected a runtime to have been created");
        let mut base = runtime.block_on(base_arc.lock());

        base.reset_api_stats();
    }

    async fn retrieve(&self, genre: &Genre, sort_by: &SortBy, keywords: &String, page: u32) -> providers::Result<Page<Box<dyn Media>>> {
        let base_arc = &self.base.clone();
        let mut base = base_arc.lock().await;

        match base.borrow_mut().retrieve_provider_page::<Movie>(SEARCH_RESOURCE_NAME, genre, sort_by, &keywords, page).await {
            Ok(e) => {
                info!("Retrieved a total of {} movies, [{{{}}}]", e.len(), e.iter()
                .map(|e| e.to_string())
                .join("}, {"));
                let movies: Vec<Box<dyn Media>> = e.into_iter()
                    .map(|e| Box::new(e) as Box<dyn Media>)
                    .collect();

                Ok(Page::from_content(movies))
            }
            Err(e) => {
                warn!("Failed to retrieve movie items, {}", e);
                Err(e)
            }
        }
    }

    async fn retrieve_details(&self, imdb_id: &String) -> providers::Result<Box<dyn Media>> {
        let base_arc = &self.base.clone();
        let mut base = base_arc.lock().await;

        match base.borrow_mut().retrieve_details::<Movie>(DETAILS_RESOURCE_NAME, imdb_id).await {
            Ok(e) => {
                debug!("Retrieved movie details {}", &e);
                Ok(Box::new(e))
            }
            Err(e) => {
                warn!("Failed to retrieve movie details, {}", &e);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::test::init_logger;

    use super::*;

    #[tokio::test]
    async fn test_retrieve() {
        init_logger();
        let genre = Genre::all();
        let sort_by = SortBy::new("trending".to_string(), "".to_string());
        let settings = Arc::new(Application::default());
        let provider = MovieProvider::new(&settings);

        let result = provider.retrieve(&genre, &sort_by, &String::new(), 1)
            .await
            .expect("expected media items to have been returned");

        assert!(result.total_elements() > 0, "Expected at least one item to have been found")
    }

    #[tokio::test]
    async fn test_retrieve_details() {
        init_logger();
        let imdb_id = "tt14138650".to_string();
        let settings = Arc::new(Application::default());
        let provider = MovieProvider::new(&settings);

        let media = provider.retrieve_details(&imdb_id)
            .await
            .expect("expected the details to have been returned");
        let result = media.as_any()
            .downcast_ref::<Movie>()
            .expect("expected returned type to be a movie");

        assert_eq!(&imdb_id, result.imdb_id())
    }
}