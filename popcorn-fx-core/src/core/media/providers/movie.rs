use std::borrow::BorrowMut;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use async_trait::async_trait;
use itertools::*;
use log::{debug, info, warn};
use tokio::sync::Mutex;

use crate::core::config::Application;
use crate::core::media::{Category, Favorable, Genre, MediaDetails, MediaOverview, MovieDetails, MovieOverview, SortBy};
use crate::core::media::favorites::FavoriteService;
use crate::core::media::providers::{BaseProvider, MediaProvider};
use crate::core::media::providers::utils::available_uris;

const PROVIDER_NAME: &str = "movies";
const SEARCH_RESOURCE_NAME: &str = "movies";
const DETAILS_RESOURCE_NAME: &str = "movie";

/// The [MediaProvider] for movie media items.
#[derive(Debug)]
pub struct MovieProvider {
    base: Arc<Mutex<BaseProvider>>,
    favorite_service: Arc<FavoriteService>,
}

impl MovieProvider {
    pub fn new(settings: &Arc<Application>, favorite_service: &Arc<FavoriteService>) -> Self {
        let uris = available_uris(settings, PROVIDER_NAME);

        Self {
            base: Arc::new(Mutex::new(BaseProvider::new(uris))),
            favorite_service: favorite_service.clone(),
        }
    }
}

impl Display for MovieProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MovieProvider")
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

    async fn retrieve(&self, genre: &Genre, sort_by: &SortBy, keywords: &String, page: u32) -> crate::core::media::Result<Vec<Box<dyn MediaOverview>>> {
        let base_arc = &self.base.clone();
        let favorite_service = self.favorite_service.clone();
        let mut base = base_arc.lock().await;

        match base.borrow_mut().retrieve_provider_page::<MovieOverview>(SEARCH_RESOURCE_NAME, genre, sort_by, &keywords, page).await {
            Ok(e) => {
                info!("Retrieved a total of {} movies, [{{{}}}]", e.len(), e.iter()
                .map(|e| e.to_string())
                .join("}, {"));
                let movies: Vec<Box<dyn MediaOverview>> = e.into_iter()
                    .map(|mut e| {
                        e.update_liked(favorite_service.is_liked(&e));
                        e
                    })
                    .map(|e| Box::new(e) as Box<dyn MediaOverview>)
                    .collect();

                Ok(movies)
            }
            Err(e) => {
                warn!("Failed to retrieve movie items, {}", e);
                Err(e)
            }
        }
    }

    async fn retrieve_details(&self, imdb_id: &String) -> crate::core::media::Result<Box<dyn MediaDetails>> {
        let base_arc = &self.base.clone();
        let mut base = base_arc.lock().await;

        match base.borrow_mut().retrieve_details::<MovieDetails>(DETAILS_RESOURCE_NAME, imdb_id).await {
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
    use std::collections::HashMap;
    use std::env::temp_dir;

    use httpmock::Method::GET;
    use httpmock::MockServer;

    use crate::core::config::{PopcornProperties, PopcornSettings, ProviderProperties, SubtitleProperties};
    use crate::core::media::{Images, MediaIdentifier, Rating};
    use crate::core::storage::Storage;
    use crate::testing::{init_logger, read_test_file};

    use super::*;

    fn start_mock_server() -> (MockServer, Arc<Application>) {
        let server = MockServer::start();
        let settings = Arc::new(Application::new(
            PopcornProperties::new_with_providers(SubtitleProperties::default(), create_providers(&server)),
            PopcornSettings::default(),
        ));

        (server, settings)
    }

    #[tokio::test]
    async fn test_retrieve() {
        init_logger();
        let (server, settings) = start_mock_server();
        let temp_dir = temp_dir();
        let genre = Genre::all();
        let sort_by = SortBy::new("trending".to_string(), "".to_string());
        let storage = Arc::new(Storage::from_directory(temp_dir.as_path().to_str().unwrap()));
        let favorites = Arc::new(FavoriteService::new(&storage));
        let provider = MovieProvider::new(&settings, &favorites);
        let expected_result = MovieOverview::new_detailed(
            "Lorem Ipsum".to_string(),
            "tt9764362".to_string(),
            "2022".to_string(),
            Some(Rating::new_with_metadata(
                72,
                18,
                1270,
                0,
                0,
            )),
            Images::new(
                "http://image.tmdb.org/t/p/w500/poster.jpg".to_string(),
                "http://image.tmdb.org/t/p/w500/fanart.jpg".to_string(),
                "http://image.tmdb.org/t/p/w500/banner.jpg".to_string(),
            ),
        );
        server.mock(|when, then| {
            when.method(GET)
                .path("/movies/1")
                .query_param("sort", "trending".to_string())
                .query_param("order", "-1".to_string())
                .query_param("genre", "all".to_string())
                .query_param("keywords", "".to_string());
            then.status(200)
                .header("content-type", "application/json")
                .body(read_test_file("movie-search.json"));
        });

        let result = provider.retrieve(&genre, &sort_by, &String::new(), 1)
            .await
            .expect("expected media items to have been returned");

        assert!(result.len() > 0, "Expected at least one item to have been found");
        let movie_result = result.get(0).unwrap();
        assert_eq!(expected_result.imdb_id(), movie_result.imdb_id());
        assert_eq!(expected_result.title(), movie_result.title());
    }
    //
    // #[tokio::test]
    // async fn test_retrieve_details() {
    //     init_logger();
    //     let imdb_id = "tt14138650".to_string();
    //     let settings = Arc::new(Application::default());
    //     let provider = MovieProvider::new(&settings);
    //
    //     let result = provider.retrieve_details(&imdb_id)
    //         .await
    //         .expect("expected the details to have been returned")
    //         .into_any()
    //         .downcast::<MovieDetails>()
    //         .expect("expected media to be a movie");
    //
    //     assert_eq!(imdb_id, result.imdb_id())
    // }

    fn create_providers(server: &MockServer) -> HashMap<String, ProviderProperties> {
        let mut map: HashMap<String, ProviderProperties> = HashMap::new();
        map.insert(PROVIDER_NAME.to_string(), ProviderProperties::new(
            vec![
                server.url("")
            ],
            vec![],
            vec![],
        ));
        map
    }
}