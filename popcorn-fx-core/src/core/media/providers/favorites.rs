use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use async_trait::async_trait;
use itertools::Itertools;
use log::{debug, trace};

use crate::core::media::favorites::FavoriteService;
use crate::core::media::providers::MediaProvider;
use crate::core::media::watched::WatchedService;
use crate::core::media::{Category, Genre, MediaOverview, MediaType, SortBy};

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
    /// use popcorn_fx_core::core::media::favorites::FavoriteService;
    /// use popcorn_fx_core::core::media::providers::FavoritesProvider;
    /// use popcorn_fx_core::core::media::watched::WatchedService;
    ///
    /// fn example(favorites: Arc<Box<dyn FavoriteService>>, watched_service: Arc<Box<dyn WatchedService>>) -> FavoritesProvider {    ///
    ///     FavoritesProvider::new(favorites.clone(), watched_service.clone())
    /// }
    /// ```
    ///
    /// This example demonstrates how to create a new `FavoritesProvider` instance by providing the necessary shared favorites and watched services.
    /// The `favorites` and `watched_service` parameters are of type `Arc<Box<dyn FavoriteService>>` and `Arc<Box<dyn WatchedService>>`, respectively.
    /// Ensure that you clone these services if they are used in other parts of your application.
    pub fn new(
        favorites: Arc<Box<dyn FavoriteService>>,
        watched_service: Arc<Box<dyn WatchedService>>,
    ) -> Self {
        Self {
            favorites,
            watched_service,
        }
    }

    fn filter_movies(media: &Box<dyn MediaOverview>, genre: &Genre) -> bool {
        genre.key() != FILTER_MOVIES_KEY || media.media_type() == MediaType::Movie
    }

    fn filter_shows(media: &Box<dyn MediaOverview>, genre: &Genre) -> bool {
        genre.key() != FILTER_SHOWS_KEY || media.media_type() == MediaType::Show
    }

    fn filter_keywords(media: &Box<dyn MediaOverview>, keywords: &String) -> bool {
        let normalized_keywords = keywords.trim().to_lowercase();

        if normalized_keywords.is_empty() {
            true
        } else {
            media.title().to_lowercase().contains(&normalized_keywords)
        }
    }

    fn sort_by(
        &self,
        sort_by: &SortBy,
        a: &EnhancedFavoriteItem,
        b: &EnhancedFavoriteItem,
    ) -> Ordering {
        let initial_ord = a.favorite.media_type().cmp(&b.favorite.media_type());

        if initial_ord != Ordering::Equal {
            initial_ord
        } else {
            match sort_by.key() {
                SORT_YEAR_KEY => Self::sort_by_year(&a.favorite, &b.favorite),
                SORT_RATING_KEY => Self::sort_by_rating(&a.favorite, &b.favorite),
                SORT_TITLE_KEY => Self::sort_by_title(&a.favorite, &b.favorite),
                _ => self.sort_by_watched(a, b),
            }
        }
    }

    /// Sort the given items based on the watched state.
    /// Items not seen will be put in front of the list, items seen at the back of the list.
    fn sort_by_watched(&self, a: &EnhancedFavoriteItem, b: &EnhancedFavoriteItem) -> Ordering {
        trace!(
            "Sorting media item based on watched state for {} & {}",
            a.favorite,
            b.favorite
        );

        if a.watched == b.watched {
            Ordering::Equal
        } else if a.watched {
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
            return a_rating
                .expect("rating should be present")
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

    async fn reset_api(&self) {
        // no-op
    }

    async fn retrieve(
        &self,
        genre: &Genre,
        sort_by: &SortBy,
        keywords: &String,
        page: u32,
    ) -> crate::core::media::Result<Vec<Box<dyn MediaOverview>>> {
        // only return one page with all favorites
        if page > 1 {
            trace!("Favorites provider returns all favorites on page 1, additional pages will always return an empty list");
            return Ok(vec![]);
        }

        match self.favorites.all().await {
            Ok(favorites) => {
                let total_favorites = favorites.len();
                trace!("Filtering a total of {} favorites", total_favorites);
                let mut items = vec![];

                for favorite in favorites
                    .into_iter()
                    .filter(|e| Self::filter_movies(e, genre))
                    .filter(|e| Self::filter_shows(e, genre))
                    .filter(|e| Self::filter_keywords(e, keywords))
                {
                    let watched = self.watched_service.is_watched(favorite.imdb_id()).await;
                    items.push(EnhancedFavoriteItem { favorite, watched });
                }

                let filtered: Vec<Box<dyn MediaOverview>> = items
                    .into_iter()
                    .sorted_by(|a, b| self.sort_by(sort_by, a, b))
                    .map(|e| e.favorite)
                    .collect();
                debug!(
                    "Retrieved a total of {} favorites out of {}",
                    filtered.len(),
                    total_favorites
                );
                Ok(filtered)
            }
            Err(e) => Err(e),
        }
    }
}

struct EnhancedFavoriteItem {
    favorite: Box<dyn MediaOverview>,
    watched: bool,
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use crate::core::event::EventPublisher;
    use crate::core::media;
    use crate::core::media::favorites::{FXFavoriteService, MockFavoriteService};
    use crate::core::media::watched::test::MockWatchedService;
    use crate::core::media::watched::DefaultWatchedService;
    use crate::core::media::{Images, MovieOverview, ShowOverview};
    use crate::init_logger;
    use crate::testing::copy_test_file;

    use super::*;

    #[tokio::test]
    async fn test_retrieve_return_stored_favorites() {
        init_logger!();
        let imdb_id = "tt21215466";
        let genre = Genre::all();
        let sort_by = SortBy::new("watched".to_string(), String::new());
        let keywords = "".to_string();
        let mut watched_service = MockWatchedService::new();
        watched_service.expect_is_watched().return_const(true);
        let mut favorites = MockFavoriteService::new();
        favorites
            .expect_all()
            .returning(|| -> media::Result<Vec<Box<dyn MediaOverview>>> {
                Ok(vec![Box::new(MovieOverview::new(
                    String::new(),
                    imdb_id.to_string(),
                    String::new(),
                ))])
            });
        let provider = FavoritesProvider::new(
            Arc::new(Box::new(favorites)),
            Arc::new(Box::new(watched_service)),
        );

        let result = provider
            .retrieve(&genre, &sort_by, &keywords, 1)
            .await
            .expect("expected the favorites to have been returned");

        assert_eq!(1, result.len())
    }

    #[tokio::test]
    async fn test_retrieve_return_stored_favorites_movies() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let genre = Genre::new("movies".to_string(), "Movies".to_string());
        let sort_by = SortBy::new("watched".to_string(), String::new());
        let keywords = "".to_string();
        copy_test_file(temp_path, "favorites2.json", Some("favorites.json"));
        let favorites = FXFavoriteService::new(temp_path);
        let mut watched_service = MockWatchedService::new();
        watched_service.expect_is_watched().return_const(false);
        let provider = FavoritesProvider::new(
            Arc::new(Box::new(favorites)),
            Arc::new(Box::new(watched_service)),
        );

        let result = provider
            .retrieve(&genre, &sort_by, &keywords, 1)
            .await
            .expect("expected the favorites to have been returned");

        assert_eq!(21, result.len())
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

    #[tokio::test]
    async fn test_sort_by_should_order_movie_before_show() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a temp directory");
        let resource_path = temp_dir.path().to_str().unwrap();
        let favorites = MockFavoriteService::new();
        let service = FavoritesProvider::new(
            Arc::new(Box::new(favorites)),
            Arc::new(Box::new(DefaultWatchedService::new(
                resource_path,
                EventPublisher::default(),
            ))),
        );
        let sort_by = SortBy::new(SORT_TITLE_KEY.to_string(), String::new());
        let movie = EnhancedFavoriteItem {
            favorite: Box::new(MovieOverview::new(
                String::new(),
                String::new(),
                String::new(),
            )) as Box<dyn MediaOverview>,
            watched: false,
        };
        let show = EnhancedFavoriteItem {
            favorite: Box::new(ShowOverview::new(
                "tt875495".to_string(),
                String::new(),
                String::new(),
                String::new(),
                0,
                Images::none(),
                None,
            )) as Box<dyn MediaOverview>,
            watched: false,
        };

        let result = service.sort_by(&sort_by, &movie, &show);

        assert_eq!(Ordering::Less, result)
    }

    #[test]
    fn test_sort_by_should_order_unwatched_before_watched() {
        init_logger!();
        let watched_id = "tt0000001".to_string();
        let movie_watched = EnhancedFavoriteItem {
            favorite: Box::new(MovieOverview::new(
                String::new(),
                watched_id.clone(),
                String::new(),
            )) as Box<dyn MediaOverview>,
            watched: true,
        };
        let movie_unwatched = EnhancedFavoriteItem {
            favorite: Box::new(MovieOverview::new(
                String::new(),
                "tt0000002".to_string(),
                String::new(),
            )) as Box<dyn MediaOverview>,
            watched: false,
        };
        let favorites = MockFavoriteService::new();
        let mut watched = MockWatchedService::new();
        watched
            .expect_is_watched()
            .returning(move |id: &str| -> bool { id == watched_id });
        let service =
            FavoritesProvider::new(Arc::new(Box::new(favorites)), Arc::new(Box::new(watched)));
        let sort_by = SortBy::new("watched".to_string(), String::new());

        let result = service.sort_by(&sort_by, &movie_watched, &movie_unwatched);

        assert_eq!(Ordering::Greater, result)
    }
}
