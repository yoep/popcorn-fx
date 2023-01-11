use async_trait::async_trait;

use crate::core::media::{Category, Genre, MediaDetails, MediaOverview, SortBy};
use crate::core::media::providers::MediaProvider;
use crate::core::Page;

/// The [MediaProvider] for liked media items.
#[derive(Debug)]
pub struct FavoritesProvider {}

impl FavoritesProvider {}

#[async_trait]
impl MediaProvider for FavoritesProvider {
    fn supports(&self, category: &Category) -> bool {
        category == &Category::FAVORITES
    }

    fn reset_api(&self) {
        // no-op
    }

    async fn retrieve(&self, genre: &Genre, sort_by: &SortBy, keywords: &String, page: u32) -> crate::core::media::providers::Result<Page<Box<dyn MediaOverview>>> {
        todo!()
    }

    async fn retrieve_details(&self, imdb_id: &String) -> crate::core::media::providers::Result<Box<dyn MediaDetails>> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_() {

    }
}