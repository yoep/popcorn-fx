use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::core::config::Application;
use crate::core::media::{Category, Genre, Media, SortBy};
use crate::core::media::providers::{BaseProvider, MediaProvider};
use crate::core::media::providers::utils::available_uris;
use crate::core::Page;

const PROVIDER_NAME: &str = "series";

#[derive(Debug)]
pub struct ShowProvider {
    base: Arc<Mutex<BaseProvider>>,
}

impl ShowProvider {
    pub fn new(settings: &Arc<Application>) -> Self {
        let uris = available_uris(settings, PROVIDER_NAME);

        Self {
            base: Arc::new(Mutex::new(BaseProvider::new(uris))),
        }
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

    async fn retrieve(&self, genre: &Genre, sort_by: &SortBy, keywords: &String, page: u32) -> crate::core::media::providers::Result<Page<Box<dyn Media>>> {
        todo!()
    }

    async fn retrieve_details(&self, imdb_id: &String) -> crate::core::media::providers::Result<Box<dyn Media>> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_retrieve() {
        let settings = Arc::new(Application::default());
        let provider = ShowProvider::new(&settings);
    }
}