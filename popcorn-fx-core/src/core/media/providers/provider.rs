use std::fmt::{Debug, Display};
use std::fmt::Formatter;

use async_trait::async_trait;
use mockall::automock;

use crate::core::media;
use crate::core::media::{Category, Genre, MediaDetails, MediaOverview, MediaType, SortBy};

/// A common definition of a `Media` item provider.
/// It provides details about certain `Media` items based on the `Category` it supports.
#[automock]
#[async_trait]
pub trait MediaProvider: Debug + Display + Send + Sync {
    /// Verifies if the provider supports the given `Category`.
    ///
    /// # Arguments
    ///
    /// * `category` - The `Category` to check support for.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the provider supports the given `Category`.
    fn supports(&self, category: &Category) -> bool;

    /// Resets the API statistics and re-enables all disabled APIs.
    fn reset_api(&self);

    /// Retrieves a page of `MediaOverview` items based on the given criteria.
    ///
    /// The media items only contain basic information to present as an overview.
    ///
    /// # Arguments
    ///
    /// * `genre` - The genre of the media items to retrieve.
    /// * `sort_by` - The sorting criteria for the retrieved media items.
    /// * `keywords` - The keywords to search for in the media items.
    /// * `page` - The page number of the results to retrieve.
    ///
    /// # Returns
    ///
    /// A `Result` containing the retrieved page of `MediaOverview` items on success, or a `ProviderError` on failure.
    async fn retrieve(&self, genre: &Genre, sort_by: &SortBy, keywords: &String, page: u32) -> media::Result<Vec<Box<dyn MediaOverview>>>;
}

#[automock]
#[async_trait]
pub trait MediaDetailsProvider: Debug + Display + Send + Sync {
    /// Verifies if the provider supports the given `MediaType`.
    ///
    /// # Arguments
    ///
    /// * `media_type` - The `MediaType` to check support for.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the provider supports the given `MediaType`.
    fn supports(&self, media_type: &MediaType) -> bool;

    /// Resets the API statistics and re-enables all disabled APIs.
    fn reset_api(&self);

    /// Retrieves the `MediaDetails` for the given IMDB ID item.
    ///
    /// The media item will contain all the information for a media description and playback.
    ///
    /// # Arguments
    ///
    /// * `imdb_id` - The IMDB ID of the media item to retrieve.
    ///
    /// # Returns
    ///
    /// A `Result` containing the retrieved `MediaDetails` on success, or a `ProviderError` on failure.
    async fn retrieve_details(&self, imdb_id: &str) -> media::Result<Box<dyn MediaDetails>>;
}

impl Display for MockMediaProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockMediaProvider")
    }
}

impl Display for MockMediaDetailsProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockMediaDetailsProvider")
    }
}