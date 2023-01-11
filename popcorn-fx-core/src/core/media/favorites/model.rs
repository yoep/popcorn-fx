use chrono::Local;
use serde::Deserialize;

use crate::core::media::{MovieOverview, ShowOverview};

const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S.%f";

/// The favorites/liked media items of the user.
#[derive(Debug, Deserialize)]
pub struct Favorites {
    movies: Vec<MovieOverview>,
    shows: Vec<ShowOverview>,
    last_cache_update: String,
}

impl Favorites {
    pub fn new(movies: Vec<MovieOverview>, shows: Vec<ShowOverview>, last_cache_update: String) -> Self {
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

    fn current_datetime() -> String {
        let now = Local::now();
        now.format(DATETIME_FORMAT).to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_empty() {
        let movies: Vec<MovieOverview> = vec![];
        let shows: Vec<ShowOverview> = vec![];

        let result = Favorites::empty();

        assert_eq!(movies, result.movies);
        assert_eq!(shows, result.shows);
    }
}