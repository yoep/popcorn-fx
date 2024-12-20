use log::trace;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Watched {
    movies: Vec<String>,
    shows: Vec<String>,
}

impl Watched {
    pub fn new(movies: Vec<String>, shows: Vec<String>) -> Self {
        Self { movies, shows }
    }

    pub fn empty() -> Self {
        Self {
            movies: vec![],
            shows: vec![],
        }
    }

    pub fn contains(&self, id: &str) -> bool {
        self.movies.iter().any(|e| e.eq(id)) || self.shows.iter().any(|e| e.eq(id))
    }

    pub fn movies(&self) -> &Vec<String> {
        &self.movies
    }

    pub fn shows(&self) -> &Vec<String> {
        &self.shows
    }

    /// Add the given movie ID as watched.
    /// Duplicate items will be automatically ignored.
    ///
    /// * `id`  - The movie ID to mark as watched
    pub fn add_movie(&mut self, id: &str) {
        let id = id.to_string();
        if !self.movies.contains(&id) {
            trace!("Adding movie ID {} as watched", id);
            self.movies.push(id);
        }
    }

    /// Add the given show/episode ID as watched.
    /// Duplicate items will be automatically ignored.
    ///
    /// * `id`  - The show/episode ID to mark as watched
    pub fn add_show(&mut self, id: &str) {
        let id = id.to_string();
        if !self.shows.contains(&id) {
            trace!("Adding show ID {} as watched", &id);
            self.shows.push(id);
        }
    }

    /// Remove the given watched item ID from the list.
    /// Unknown ID's which are not within the watched items are auto ignored.
    pub fn remove(&mut self, id: &str) {
        let movie_index = self.movies.iter().position(|e| e.as_str() == id);
        let show_index = self.shows.iter().position(|e| e.as_str() == id);

        if movie_index.is_some() {
            trace!("Removing movie {} from the watched items", &id);
            self.movies.remove(movie_index.unwrap());
        }
        if show_index.is_some() {
            trace!("Removing show {} from the watched items", &id);
            self.shows.remove(show_index.unwrap());
        }
    }
}

#[cfg(test)]
mod test {
    use crate::core::media::watched::Watched;

    #[test]
    fn test_contains_id_is_watched() {
        let id = "tt457896".to_string();
        let watched = Watched::new(vec![id.clone()], vec![]);

        let result = watched.contains(id.as_str());

        assert!(result, "expected the id to be present")
    }

    #[test]
    fn test_contains_id_is_not_watched() {
        let id = "tt875421".to_string();
        let watched = Watched::empty();

        let result = watched.contains(id.as_str());

        assert!(!result, "expected the id to not have been watched")
    }
}
