use std::fmt::Debug;

use async_trait::async_trait;

use crate::core::media::{Category, Genre, Media, SortBy};
use crate::core::media::providers;
use crate::core::Page;

#[async_trait]
pub trait Provider<T>: Debug
    where T: Media {
    /// Verify if the provider supports the given [Category].
    fn supports(&self, category: &Category) -> bool;
    
    /// Reset the api statics and re-enable all disabled api's.
    fn reset_api(&self);

    /// Retrieve a page of [Media] items based on the given criteria.
    ///
    /// It returns the retrieves page on success, else the [providers::ProviderError].
    async fn retrieve(&self, genre: &Genre, sort_by: &SortBy, keywords: &String, page: u32) -> providers::Result<Page<T>>;

    /// Retrieve the details for the given IMDB ID item.
    ///
    /// It returns the details on success, else the [providers::ProviderError].
    async fn retrieve_details(&self, imdb_id: &String) -> providers::Result<T>;
}