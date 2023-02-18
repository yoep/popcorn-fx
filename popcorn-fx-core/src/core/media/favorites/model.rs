use chrono::Local;
use log::{debug, trace};
use serde::{Deserialize, Serialize};

use crate::core::media::{MediaIdentifier, MovieOverview, ShowOverview};

const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S.%f";

/// The favorites/liked media items of the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favorites {
    /// The liked movies of the user
    pub movies: Vec<MovieOverview>,
    /// The liked shows of the user
    pub shows: Vec<ShowOverview>,
    /// The last time this cache has been updated
    pub last_cache_update: String,
}

impl Favorites {
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
        if !self.contains(media.imdb_id()) {
            trace!("Adding media {} to favorites", &media);
            self.movies.push(media.clone())
        }
    }

    /// Add the given show to the favorites.
    /// Duplicates will be automatically ignored.
    pub fn add_show(&mut self, media: &ShowOverview) {
        if !self.contains(media.imdb_id()) {
            trace!("Adding media {} to favorites", &media);
            self.shows.push(media.clone())
        }
    }

    /// Verify if the favorites contain the given media item ID.
    /// It returns `true` when the id is liked, else `false`.
    pub fn contains(&self, imdb_id: &str) -> bool {
        self.movies.iter().any(|e| e.imdb_id() == imdb_id)
            || self.shows.iter().any(|e| e.imdb_id() == imdb_id)
    }

    /// Remove the media item from the favorites based on the given ID.
    pub fn remove_id(&mut self, imdb_id: &str) {
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

impl Default for Favorites {
    fn default() -> Self {
        Self {
            movies: vec![],
            shows: vec![],
            last_cache_update: Self::current_datetime(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::core::media::Images;

    use super::*;

    #[test]
    fn test_default() {
        let movies: Vec<MovieOverview> = vec![];
        let shows: Vec<ShowOverview> = vec![];

        let result = Favorites::default();

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
        let mut favorites = Favorites::default();

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
        let mut favorites = Favorites {
            movies: vec![movie.clone()],
            shows: vec![],
            last_cache_update: "2023-01-01T22:00:00.129617500".to_string(),
        };

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
        let mut favorites = Favorites::default();

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
        let mut favorites = Favorites {
            movies: vec![],
            shows: vec![show.clone()],
            last_cache_update: "2023-01-01T22:00:00.129617500".to_string(),
        };

        favorites.add_show(&show);
        let result = favorites.shows();

        assert_eq!(1, result.len())
    }
}