use std::fmt::{Debug, Display};

use async_trait::async_trait;

use crate::core::media;
use crate::core::media::{Category, Genre, MediaDetails, MediaOverview, SortBy};

/// A common definition of a [Media] item provider.
/// It will provide details about certain [Media] items based on the [Category] it supports.
#[async_trait]
pub trait MediaProvider: Debug + Display + Send + Sync {
    /// Verify if the provider supports the given [Category].
    fn supports(&self, category: &Category) -> bool;

    /// Reset the api statics and re-enable all disabled api's.
    fn reset_api(&self);

    /// Retrieve a page of [MediaOverview] items based on the given criteria.
    /// The media items only contain basic information to present as an overview.
    ///
    /// It returns the retrieves page on success, else the [providers::ProviderError].
    async fn retrieve(&self, genre: &Genre, sort_by: &SortBy, keywords: &String, page: u32) -> media::Result<Vec<Box<dyn MediaOverview>>>;

    /// Retrieve the [MediaDetails] for the given IMDB ID item.
    /// The media item will contain all information for a media description and playback.
    ///
    /// It returns the details on success, else the [providers::ProviderError].
    async fn retrieve_details(&self, imdb_id: &String) -> media::Result<Box<dyn MediaDetails>>;
}