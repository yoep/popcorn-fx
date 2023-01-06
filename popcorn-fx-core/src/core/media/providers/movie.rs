use std::borrow::BorrowMut;
use std::sync::Arc;

use async_trait::async_trait;
use log::{error, info, warn};
use tokio::sync::Mutex;

use crate::core::config::Application;
use crate::core::media::{Category, Genre, Movie, providers, SortBy};
use crate::core::media::providers::{BaseProvider, Provider};
use crate::core::Page;

const PROVIDER_NAME: &str = "movies";
const RESOURCE_NAME: &str = "movies";

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

        if api_server.is_some() {
            uris.push(api_server.unwrap().clone());
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

    async fn retrieve(&self, genre: &Genre, sort_by: &SortBy, page: i32) -> providers::Result<Page<Movie>> {
        let base_arc = &self.base.clone();
        let mut base = base_arc.lock().await;

        match base.borrow_mut().retrieve_provider_page::<Movie>(RESOURCE_NAME, genre, sort_by, &String::new(), page).await {
            Ok(e) => {
                info!("Retrieved movies {:?}", e);
                Ok(Page::from_content(e))
            }
            Err(e) => {
                warn!("Failed to retrieve movie items, {}", e);
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

        let result = provider.retrieve(&genre, &sort_by, 1).await;

        assert!(result.is_ok(), "Expected the media retrieve to succeed");
        assert!(result.unwrap().total_elements() > 0, "Expected at least one item to have been found")
    }
}