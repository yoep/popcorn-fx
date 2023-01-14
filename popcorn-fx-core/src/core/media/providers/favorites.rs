use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use async_trait::async_trait;
use itertools::Itertools;
use log::{error, trace};
use tokio::sync::Mutex;

use crate::core::media::{Category, Genre, MediaDetails, MediaError, MediaOverview, MediaType, SortBy};
use crate::core::media::favorites::FavoriteService;
use crate::core::media::providers::MediaProvider;

const FILTER_MOVIES_KEY: &str = "movies";
const FILTER_SHOWS_KEY: &str = "tv";
const SORT_YEAR_KEY: &str = "year";
const SORT_TITLE_KEY: &str = "title";
const SORT_RATING_KEY: &str = "rating";

/// The [MediaProvider] for liked media items.
#[derive(Debug)]
pub struct FavoritesProvider {
    favorites: Arc<FavoriteService>,
    providers: Mutex<Vec<Arc<Box<dyn MediaProvider>>>>,
}

impl FavoritesProvider {
    pub fn new(favorites: &Arc<FavoriteService>, providers: Vec<&Arc<Box<dyn MediaProvider>>>) -> Self {
        Self {
            favorites: favorites.clone(),
            providers: Mutex::new(providers.into_iter()
                .map(|e| e.clone())
                .collect()),
        }
    }

    fn filter_movies(media: &Box<dyn MediaOverview>, genre: &Genre) -> bool {
        genre.key().as_str() != FILTER_MOVIES_KEY || media.media_type() == MediaType::Movie
    }

    fn filter_shows(media: &Box<dyn MediaOverview>, genre: &Genre) -> bool {
        genre.key().as_str() != FILTER_SHOWS_KEY || media.media_type() == MediaType::Show
    }

    fn filter_keywords(media: &Box<dyn MediaOverview>, keywords: &String) -> bool {
        let normalized_keywords = keywords.trim().to_lowercase();

        if normalized_keywords.is_empty() {
            true
        } else {
            media.title().to_lowercase().contains(&normalized_keywords)
        }
    }

    fn sort_by(sort_by: &SortBy, a: &Box<dyn MediaOverview>, b: &Box<dyn MediaOverview>) -> Ordering {
        let initial_ord = a.media_type().cmp(&b.media_type());

        if initial_ord != Ordering::Equal {
            initial_ord
        } else {
            return match sort_by.key().as_str() {
                SORT_YEAR_KEY => Self::sort_by_year(a, b),
                SORT_RATING_KEY => Self::sort_by_rating(a, b),
                SORT_TITLE_KEY => Self::sort_by_title(a, b),
                _ => Self::sort_by_watched(a, b),
            };
        }
    }

    fn sort_by_watched(a: &Box<dyn MediaOverview>, b: &Box<dyn MediaOverview>) -> Ordering {
        if a.is_watched() == b.is_watched() {
            Ordering::Equal
        } else if *a.is_watched() {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }

    fn sort_by_year(a: &Box<dyn MediaOverview>, b: &Box<dyn MediaOverview>) -> Ordering {
        a.year().cmp(b.year()).reverse()
    }

    fn sort_by_rating(a: &Box<dyn MediaOverview>, b: &Box<dyn MediaOverview>) -> Ordering {
        let a_rating = a.rating();
        let b_rating = b.rating();

        if a_rating.is_some() && b_rating.is_some() {
            return a_rating.expect("rating should be present")
                .cmp(b_rating.expect("rating should be present"))
                .reverse();
        } else if a_rating.is_some() {
            return Ordering::Less;
        } else if b_rating.is_some() {
            return Ordering::Greater;
        }

        Ordering::Equal
    }

    fn sort_by_title(a: &Box<dyn MediaOverview>, b: &Box<dyn MediaOverview>) -> Ordering {
        a.title().cmp(&b.title())
    }

    fn media_type_to_category(media_type: MediaType) -> Option<Category> {
        match media_type {
            MediaType::Movie => Some(Category::MOVIES),
            MediaType::Show => Some(Category::SERIES),
            _ => {
                error!("Media type {} doesn't support any categories", media_type);
                None
            }
        }
    }

    async fn retrieve_media_details(&self, media: Box<dyn MediaOverview>) -> Result<Box<dyn MediaDetails>, MediaError> {
        match Self::media_type_to_category(media.media_type()) {
            Some(category) => {
                let providers = self.providers.lock().await;

                for provider in providers.iter() {
                    let imdb_id = media.imdb_id();

                    if provider.supports(&category) {
                        trace!("Using favorite sub provider {} for retrieving details of {}", &provider, &imdb_id);
                        return provider.retrieve_details(&imdb_id).await;
                    }
                }

                Err(MediaError::ProviderNotFound(category.to_string()))
            }
            None => Err(MediaError::NoAvailableProviders)
        }
    }
}

impl Display for FavoritesProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FavoritesProvider")
    }
}

#[async_trait]
impl MediaProvider for FavoritesProvider {
    fn supports(&self, category: &Category) -> bool {
        category == &Category::FAVORITES
    }

    fn reset_api(&self) {
        // no-op
    }

    async fn retrieve(&self, genre: &Genre, sort_by: &SortBy, keywords: &String, page: u32) -> crate::core::media::Result<Vec<Box<dyn MediaOverview>>> {
        // only return one page with all favorites
        if page > 1 {
            return Ok(vec![]);
        }

        match self.favorites.all() {
            Ok(favorites) => {
                Ok(favorites.into_iter()
                    .filter(|e| Self::filter_movies(e, genre))
                    .filter(|e| Self::filter_shows(e, genre))
                    .filter(|e| Self::filter_keywords(e, keywords))
                    .sorted_by(|a, b| Self::sort_by(sort_by, a, b))
                    .collect())
            }
            Err(e) => Err(e)
        }
    }

    async fn retrieve_details(&self, imdb_id: &String) -> crate::core::media::Result<Box<dyn MediaDetails>> {
        match self.favorites.find_id(imdb_id) {
            Some(e) => self.retrieve_media_details(e).await,
            None => Err(MediaError::FavoriteNotFound(imdb_id.clone()))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::core::config::Application;
    use crate::core::media::{Images, MovieOverview, ShowOverview};
    use crate::core::media::providers::MovieProvider;
    use crate::core::storage::Storage;
    use crate::testing::{init_logger, test_resource_directory};

    use super::*;

    #[tokio::test]
    async fn test_retrieve_return_stored_favorites() {
        init_logger();
        let resource_directory = test_resource_directory();
        let genre = Genre::all();
        let sort_by = SortBy::new("watched".to_string(), String::new());
        let keywords = "".to_string();
        let storage = Arc::new(Storage::from_directory(resource_directory.to_str().expect("expected resource path to be valid")));
        let favorites = Arc::new(FavoriteService::new(&storage));
        let provider = FavoritesProvider::new(&favorites, vec![]);

        let result = provider.retrieve(&genre, &sort_by, &keywords, 1)
            .await
            .expect("expected the favorites to have been returned");

        assert_eq!(1, result.len())
    }

    #[tokio::test]
    async fn test_retrieve_details() {
        init_logger();
        let imdb_id = "tt1156398";
        let resource_directory = test_resource_directory();
        let settings = Arc::new(Application::default());
        let storage = Arc::new(Storage::from_directory(resource_directory.to_str().expect("expected resource path to be valid")));
        let favorites = Arc::new(FavoriteService::new(&storage));
        let movie_provider = Arc::new(Box::new(MovieProvider::new(&settings, &favorites)) as Box<dyn MediaProvider>);
        let provider = FavoritesProvider::new(&favorites, vec![&movie_provider]);

        let result = provider.retrieve_details(&imdb_id.to_string())
            .await
            .expect("expected the details to have been retrieved");

        assert_eq!(imdb_id.to_string(), result.imdb_id())
    }

    #[test]
    fn test_filter_movies_when_genre_is_movies_and_type_is_movies_should_return_true() {
        let genre = Genre::new(FILTER_MOVIES_KEY.to_string(), String::new());
        let media: Box<dyn MediaOverview> = Box::new(MovieOverview::new(
            String::new(),
            String::new(),
            String::new(),
        ));

        let result = FavoritesProvider::filter_movies(&media, &genre);

        assert_eq!(true, result)
    }

    #[test]
    fn test_filter_movies_when_genre_is_movies_and_type_is_show_should_return_false() {
        let genre = Genre::new(FILTER_MOVIES_KEY.to_string(), String::new());
        let media: Box<dyn MediaOverview> = Box::new(ShowOverview::new(
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            1,
            Images::none(),
            None,
        ));

        let result = FavoritesProvider::filter_movies(&media, &genre);

        assert_eq!(false, result)
    }

    #[test]
    fn test_filter_keywords_when_keywords_is_empty_should_return_true() {
        let keywords = " ".to_string();
        let media: Box<dyn MediaOverview> = Box::new(MovieOverview::new(
            "lorem ipsum".to_string(),
            String::new(),
            String::new(),
        ));

        let result = FavoritesProvider::filter_keywords(&media, &keywords);

        assert_eq!(true, result)
    }

    #[test]
    fn test_filter_keywords_when_keyword_matches_should_return_true() {
        let keywords = "Lor".to_string();
        let media: Box<dyn MediaOverview> = Box::new(MovieOverview::new(
            "lorem ipsum".to_string(),
            String::new(),
            String::new(),
        ));

        let result = FavoritesProvider::filter_keywords(&media, &keywords);

        assert_eq!(true, result)
    }

    #[test]
    fn test_filter_keywords_when_keyword_is_different_should_return_false() {
        let keywords = "zombie".to_string();
        let media: Box<dyn MediaOverview> = Box::new(MovieOverview::new(
            "lorem ipsum".to_string(),
            String::new(),
            String::new(),
        ));

        let result = FavoritesProvider::filter_keywords(&media, &keywords);

        assert_eq!(false, result)
    }

    #[test]
    fn test_sort_by_should_order_movie_before_show() {
        let sort_by = SortBy::new(String::new(), String::new());
        let movie = Box::new(MovieOverview::new(
            String::new(),
            String::new(),
            String::new(),
        )) as Box<dyn MediaOverview>;
        let show = Box::new(ShowOverview::new(
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            0,
            Images::none(),
            None,
        )) as Box<dyn MediaOverview>;

        let result = FavoritesProvider::sort_by(&sort_by, &movie, &show);

        assert_eq!(Ordering::Less, result)
    }
}