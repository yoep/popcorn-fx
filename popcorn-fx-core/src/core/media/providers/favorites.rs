use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use async_trait::async_trait;
use itertools::Itertools;
use log::{debug, trace};

use crate::core::media::{Category, Genre, MediaOverview, MediaType, SortBy};
use crate::core::media::favorites::FavoriteService;
use crate::core::media::providers::MediaProvider;
use crate::core::media::watched::WatchedService;

const FILTER_MOVIES_KEY: &str = "movies";
const FILTER_SHOWS_KEY: &str = "tv";
const SORT_YEAR_KEY: &str = "year";
const SORT_TITLE_KEY: &str = "title";
const SORT_RATING_KEY: &str = "rating";

/// The `FavoritesProvider` for liked media items.
#[derive(Debug)]
pub struct FavoritesProvider {
    favorites: Arc<Box<dyn FavoriteService>>,
    watched_service: Arc<Box<dyn WatchedService>>,
}

impl FavoritesProvider {
    /// Create a new `FavoritesProvider` instance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use popcorn_fx_core::core::media::providers::FavoritesProvider;
    ///
    /// fn example() -> FavoritesProvider {
    ///     let favorites = Arc::new(XXX);
    ///     let watched = Arc::new(XXX);
    ///
    ///     FavoritesProvider::new(favorites.clone(), watched.clone())
    /// }
    /// ```
    ///
    /// This example demonstrates how to create a new `FavoritesProvider` instance by providing the necessary shared favorites and watched services.
    /// The `favorites` and `watched_service` parameters are of type `Arc<Box<dyn FavoriteService>>` and `Arc<Box<dyn WatchedService>>`, respectively.
    /// Ensure that you clone these services if they are used in other parts of your application.
    pub fn new(favorites: Arc<Box<dyn FavoriteService>>, watched_service: Arc<Box<dyn WatchedService>>) -> Self {
        Self {
            favorites,
            watched_service,
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

    fn sort_by(&self, sort_by: &SortBy, a: &Box<dyn MediaOverview>, b: &Box<dyn MediaOverview>) -> Ordering {
        let initial_ord = a.media_type().cmp(&b.media_type());

        if initial_ord != Ordering::Equal {
            initial_ord
        } else {
            return match sort_by.key().as_str() {
                SORT_YEAR_KEY => Self::sort_by_year(a, b),
                SORT_RATING_KEY => Self::sort_by_rating(a, b),
                SORT_TITLE_KEY => Self::sort_by_title(a, b),
                _ => self.sort_by_watched(a, b),
            };
        }
    }

    /// Sort the given items based on the watched state.
    /// Items not seen will be put in front of the list, items seen at the back of the list.
    fn sort_by_watched(&self, a: &Box<dyn MediaOverview>, b: &Box<dyn MediaOverview>) -> Ordering {
        trace!("Sorting media item based on watched state for {} & {}", a, b);
        let a_watched = self.watched_service.is_watched(a.imdb_id());
        let b_watched = self.watched_service.is_watched(b.imdb_id());

        if a_watched == b_watched {
            Ordering::Equal
        } else if a_watched {
            Ordering::Greater
        } else {
            Ordering::Less
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
}

impl Display for FavoritesProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FavoritesProvider")
    }
}

#[async_trait]
impl MediaProvider for FavoritesProvider {
    fn supports(&self, category: &Category) -> bool {
        category == &Category::Favorites
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
                debug!("Retrieved a total of {} favorites before filtering", favorites.len());
                Ok(favorites.into_iter()
                    .filter(|e| Self::filter_movies(e, genre))
                    .filter(|e| Self::filter_shows(e, genre))
                    .filter(|e| Self::filter_keywords(e, keywords))
                    .sorted_by(|a, b| self.sort_by(sort_by, a, b))
                    .collect())
            }
            Err(e) => Err(e)
        }
    }
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;
    use tokio::sync::Mutex;

    use crate::core::config::ApplicationConfig;
    use crate::core::media;
    use crate::core::media::{Images, MovieOverview, ShowOverview};
    use crate::core::media::favorites::MockFavoriteService;
    use crate::core::media::providers::MovieProvider;
    use crate::core::media::watched::DefaultWatchedService;
    use crate::core::media::watched::MockWatchedService;
    use crate::testing::{copy_test_file, init_logger};

    use super::*;

    #[test]
    fn test_retrieve_return_stored_favorites() {
        init_logger();
        let imdb_id = "tt21215466";
        let genre = Genre::all();
        let sort_by = SortBy::new("watched".to_string(), String::new());
        let keywords = "".to_string();
        let mut favorites = MockFavoriteService::new();
        favorites.expect_all()
            .returning(|| -> media::Result<Vec<Box<dyn MediaOverview>>> {
                Ok(vec![Box::new(
                    MovieOverview::new(
                        String::new(),
                        imdb_id.to_string(),
                        String::new(),
                    )
                )])
            });
        let provider = FavoritesProvider::new(
            Arc::new(Box::new(favorites)),
            Arc::new(Box::new(MockWatchedService::new())));
        let runtime = tokio::runtime::Runtime::new().expect("expected a new runtime");

        let result = runtime.block_on(provider.retrieve(&genre, &sort_by, &keywords, 1))
            .expect("expected the favorites to have been returned");

        assert_eq!(1, result.len())
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
            "tt212154".to_string(),
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
        init_logger();
        let resource_directory = tempdir().expect("expected a temp directory");
        let resource_path = resource_directory.path().to_str().unwrap();
        let favorites = MockFavoriteService::new();
        let service = FavoritesProvider::new(
            Arc::new(Box::new(favorites)),
            Arc::new(Box::new(DefaultWatchedService::new(resource_path))));
        let sort_by = SortBy::new(SORT_TITLE_KEY.to_string(), String::new());
        let movie = Box::new(MovieOverview::new(
            String::new(),
            String::new(),
            String::new(),
        )) as Box<dyn MediaOverview>;
        let show = Box::new(ShowOverview::new(
            "tt875495".to_string(),
            String::new(),
            String::new(),
            String::new(),
            0,
            Images::none(),
            None,
        )) as Box<dyn MediaOverview>;

        let result = service.sort_by(&sort_by, &movie, &show);

        assert_eq!(Ordering::Less, result)
    }

    #[test]
    fn test_sort_by_should_order_unwatched_before_watched() {
        init_logger();
        let watched_id = "tt0000001".to_string();
        let movie_watched = Box::new(MovieOverview::new(
            String::new(),
            watched_id.clone(),
            String::new(),
        )) as Box<dyn MediaOverview>;
        let movie_unwatched = Box::new(MovieOverview::new(
            String::new(),
            "tt0000002".to_string(),
            String::new(),
        )) as Box<dyn MediaOverview>;
        let favorites = MockFavoriteService::new();
        let mut watched = MockWatchedService::new();
        watched.expect_is_watched()
            .returning(move |id: &str| -> bool {
                id == watched_id
            });
        let service = FavoritesProvider::new(
            Arc::new(Box::new(favorites)),
            Arc::new(Box::new(watched)));
        let sort_by = SortBy::new("watched".to_string(), String::new());

        let result = service.sort_by(&sort_by, &movie_watched, &movie_unwatched);

        assert_eq!(Ordering::Greater, result)
    }
}