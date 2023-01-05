use async_trait::async_trait;

use crate::core::media::{Category, Genre, Media, SortBy};
use crate::core::Page;

#[async_trait]
pub trait Provider<T> where T : Media {
    /// Verify if the provider supports the given [Category].
    fn supports(&self, category: &Category) -> bool;

    /// Retrieve a page of media items based on the given criteria.
    async fn retrieve(&self, genre: Genre, sort_by: SortBy, page: i32) -> Page<T>;
}