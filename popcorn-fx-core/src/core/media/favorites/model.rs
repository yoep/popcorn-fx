use chrono::Local;
use log::{debug, trace};
use serde::{Deserialize, Serialize};

use crate::core::media::{MediaIdentifier, MovieOverview, ShowOverview};

const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S.%f";

pub enum FavoriteEvent {

}

/// The favorites/liked media items of the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favorites {
    movies: Vec<MovieOverview>,
    shows: Vec<ShowOverview>,
    last_cache_update: String,
}

impl Favorites {
    pub fn new(movies: Vec<MovieOverview>, shows: Vec<ShowOverview>) -> Self {
        Self {
            movies,
            shows,
            last_cache_update: Self::current_datetime(),
        }
    }

    pub fn new_with_last_cache_update(movies: Vec<MovieOverview>, shows: Vec<ShowOverview>, last_cache_update: String) -> Self {
        Self {
            movies,
            shows,
            last_cache_update,
        }
    }

    /// Create a new empty instance of favorites.
    pub fn empty() -> Self {
        Self {
            movies: vec![],
            shows: vec![],
            last_cache_update: Self::current_datetime(),
        }
    }

    /// Retrieve the current liked movies of the user.
    ///
    /// It returns a reference to the array of movies.
    pub fn movies(&self) -> &Vec<MovieOverview> {
        &self.movies
    }

    /// Retrieve the current liked shows of the user.
    ///
    /// It returns a reference to the array of movies.
    pub fn shows(&self) -> &Vec<ShowOverview> {
        &self.shows
    }

    /// Add the given movie to the favorites.
    /// Duplicates will be automatically ignored.
    pub fn add_movie(&mut self, media: &MovieOverview) {
        if !self.contains(&media.imdb_id()) {
            trace!("Adding media {} to favorites", &media);
            self.movies.push(media.clone())
        }
    }

    /// Add the given show to the favorites.
    /// Duplicates will be automatically ignored.
    pub fn add_show(&mut self, media: &ShowOverview) {
        if !self.contains(&media.imdb_id()) {
            trace!("Adding media {} to favorites", &media);
            self.shows.push(media.clone())
        }
    }

    /// Verify if the favorites contain the given ID.
    pub fn contains(&self, imdb_id: &String) -> bool {
        self.movies.iter().any(|e| e.imdb_id().eq(imdb_id))
            || self.shows.iter().any(|e| e.imdb_id().eq(imdb_id))
    }

    /// Remove the media item from the favorites based on the given ID.
    pub fn remove_id(&mut self, imdb_id: &String) {
        let movie = self.movies.iter()
            .position(|e| e.imdb_id().eq(imdb_id));
        let show = self.shows.iter()
            .position(|e| e.imdb_id().eq(imdb_id));

        match movie {
            None => {}
            Some(e) => {
                debug!("Removing movie {} from favorites", imdb_id);
                self.movies.remove(e);
            }
        }

        match show {
            None => {}
            Some(e) => {
                debug!("Removing show {} from favorites", imdb_id);
                self.shows.remove(e);
            }
        }
    }

    fn current_datetime() -> String {
        let now = Local::now();
        now.format(DATETIME_FORMAT).to_string()
    }
}

#[cfg(test)]
mod test {
    use crate::core::media::Images;

    use super::*;

    #[test]
    fn test_empty() {
        let movies: Vec<MovieOverview> = vec![];
        let shows: Vec<ShowOverview> = vec![];

        let result = Favorites::empty();

        assert_eq!(movies, result.movies);
        assert_eq!(shows, result.shows);
    }

    #[test]
    fn test_add_movie_when_not_yet_present_should_add_movie() {
        let movie = MovieOverview::new(
            String::new(),
            String::from("tt12345678"),
            String::new(),
        );
        let mut favorites = Favorites::empty();

        favorites.add_movie(&movie);
        let result = favorites.movies();

        assert!(result.contains(&movie))
    }

    #[test]
    fn test_add_movie_when_already_present_should_ignore() {
        let movie = MovieOverview::new(
            String::new(),
            String::from("tt12345678"),
            String::new(),
        );
        let mut favorites = Favorites::new(
            vec![movie.clone()],
            vec![],
        );

        favorites.add_movie(&movie);
        let result = favorites.movies();

        assert_eq!(1, result.len())
    }

    #[test]
    fn test_add_show_when_not_yet_present_should_add_movie() {
        let show = ShowOverview::new(
            String::from("tt12345678"),
            String::new(),
            String::new(),
            String::new(),
            1,
            Images::none(),
            None,
        );
        let mut favorites = Favorites::empty();

        favorites.add_show(&show);
        let result = favorites.shows();

        assert!(result.contains(&show))
    }

    #[test]
    fn test_add_show_when_already_present_should_ignore() {
        let show = ShowOverview::new(
            String::from("tt12345678"),
            String::new(),
            String::new(),
            String::new(),
            1,
            Images::none(),
            None,
        );
        let mut favorites = Favorites::new(
            vec![],
            vec![show.clone()],
        );

        favorites.add_show(&show);
        let result = favorites.shows();

        assert_eq!(1, result.len())
    }
}