use std::borrow::BorrowMut;
use std::sync::Arc;

use async_trait::async_trait;
use itertools::*;
use log::{debug, error, info, warn};
use tokio::sync::Mutex;

use crate::core::config::Application;
use crate::core::media::{Category, Genre, Movie, providers, SortBy};
use crate::core::media::providers::{BaseProvider, Provider};
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
        let uris = Self::available_uris(settings);

        Self {
            base: Arc::new(Mutex::new(BaseProvider::new(uris))),
        }
    }

    fn available_uris(settings: &Arc<Application>) -> Vec<String> {
        let api_server = settings.settings().server().api_server();
        let mut uris: Vec<String> = vec![];

        match api_server {
            None => {}
            Some(e) => uris.push(e.clone())
        }

        match settings.properties().provider(PROVIDER_NAME.to_string()) {
            Ok(e) => {
                for uri in e.uris() {
                    uris.push(uri.clone());
                }
            }
            Err(err) => error!("Failed to retrieve provider info, {}", err)
        };

        uris
    }
}

#[async_trait]
impl Provider<Movie> for MovieProvider {
    fn supports(&self, category: &Category) -> bool {
        category == &Category::MOVIES
    }

    fn reset_api(&self) {
        let base_arc = &self.base.clone();
        let runtime = tokio::runtime::Runtime::new().expect("expected a runtime to have been created");
        let mut base = runtime.block_on(base_arc.lock());

        base.reset_api_stats();
    }

    async fn retrieve(&self, genre: &Genre, sort_by: &SortBy, keywords: &String, page: u32) -> providers::Result<Page<Movie>> {
        let base_arc = &self.base.clone();
        let mut base = base_arc.lock().await;

        match base.borrow_mut().retrieve_provider_page::<Movie>(SEARCH_RESOURCE_NAME, genre, sort_by, &keywords, page).await {
            Ok(e) => {
                info!("Retrieved a total of {} movies, [{{{}}}]", e.len(), e.iter()
                .map(|e| e.to_string())
                .join("}, {"));

                Ok(Page::from_content(e))
            }
            Err(e) => {
                warn!("Failed to retrieve movie items, {}", e);
                Err(e)
            }
        }
    }

    async fn retrieve_details(&self, imdb_id: &String) -> providers::Result<Movie> {
        let base_arc = &self.base.clone();
        let mut base = base_arc.lock().await;

        match base.borrow_mut().retrieve_details::<Movie>(DETAILS_RESOURCE_NAME, imdb_id).await {
            Ok(e) => {
                debug!("Retrieved movie details {}", &e);
                Ok(e)
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
    use std::collections::HashMap;

    use crate::core::config::{PopcornProperties, PopcornSettings, ProviderProperties, ServerSettings, SubtitleProperties, SubtitleSettings, UiSettings};
    use crate::test::init_logger;

    use super::*;

    #[test]
    fn test_available_uris_provider_available() {
        init_logger();
        let api_server = "http://lorem".to_string();
        let provider = "http://ipsum".to_string();
        let settings = Arc::new(Application::new(
            PopcornProperties::new_with_providers(SubtitleProperties::default(), HashMap::from([
                (PROVIDER_NAME.to_string(), ProviderProperties::new(
                    vec![provider.clone()],
                    vec![],
                    vec![],
                ))
            ])),
            PopcornSettings::new(SubtitleSettings::default(), UiSettings::default(),
                                 ServerSettings::new(api_server.clone())),
        ));
        let expected_result = vec![
            api_server,
            provider,
        ];

        let result = MovieProvider::available_uris(&settings);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_available_uris_provider_not_available() {
        init_logger();
        let api_server = "https://www.google.com".to_string();
        let settings = Arc::new(Application::new(
            PopcornProperties::new_with_providers(SubtitleProperties::default(), HashMap::new()),
            PopcornSettings::new(SubtitleSettings::default(), UiSettings::default(),
                                 ServerSettings::new(api_server.clone())),
        ));
        let expected_result = vec![
            api_server,
        ];

        let result = MovieProvider::available_uris(&settings);

        assert_eq!(expected_result, result)
    }

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

        let result = provider.retrieve_details(&imdb_id)
            .await
            .expect("expected the details to have been returned");

        assert_eq!(&imdb_id, result.imdb_id())
    }
}