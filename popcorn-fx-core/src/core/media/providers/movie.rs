use std::sync::Arc;

use async_trait::async_trait;

use crate::core::config::Application;
use crate::core::media::{Category, Genre, Movie, SortBy};
use crate::core::media::providers::{BaseProvider, Provider};
use crate::core::Page;

pub struct MovieProvider {
    base: BaseProvider,
    settings: Arc<Application>,
}

impl MovieProvider {
    pub fn new(settings: &Arc<Application>) -> Self {
        let uris = Self::available_uris(settings);

        Self {
            base: BaseProvider::new(uris),
            settings: settings.clone(),
        }
    }

    fn available_uris(settings: &Arc<Application>) -> Vec<String> {
        let api_server = settings.settings().server().api_server();
        let mut uris: Vec<String> = vec![];

        if api_server.is_some() {
            uris.push(api_server.unwrap().clone());
        }

        uris
    }
}

#[async_trait]
impl Provider<Movie> for MovieProvider {
    fn supports(&self, category: &Category) -> bool {
        category == &Category::MOVIES
    }

    async fn retrieve(&self, genre: Genre, sort_by: SortBy, page: i32) -> Page<Movie> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_retrieve() {
        let settings = Arc::new(Application::default());
        let provider = MovieProvider::new(&settings);

        let result = provider.retrieve(Genre::all(), SortBy::new("trending".to_string(), "".to_string()), 1).await;


    }
}