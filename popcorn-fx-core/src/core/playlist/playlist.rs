use std::collections::VecDeque;

use log::{debug, info};

use crate::core::media::MediaOverview;

/// A struct representing a playlist of media items.
#[derive(Debug, Default)]
pub struct Playlist {
    items: VecDeque<Box<dyn MediaOverview>>,
}

impl Playlist {
    /// Adds a media item to the playlist.
    ///
    /// # Arguments
    ///
    /// * `media` - A boxed trait object implementing `MediaOverview` to be added to the playlist.
    pub fn add(&mut self, media: Box<dyn MediaOverview>) {
        debug!("Adding media item {:?} to playlist", media);
        self.items.push_back(media);
    }

    /// Removes a media item from the playlist based on its IMDb ID.
    ///
    /// # Arguments
    ///
    /// * `media` - A reference to a boxed trait object implementing `MediaOverview` to be removed from the playlist.
    pub fn remove(&mut self, media: &Box<dyn MediaOverview>) {
        let position = self.items.iter()
            .position(|e| e.imdb_id() == media.imdb_id());

        if let Some(index) = position {
            debug!("Removing media item {} from playlist", media.imdb_id());
            self.items.remove(index);
        } else {
            debug!("Unable to remove media {} from the playlist, item not found", media.imdb_id());
        }
    }

    /// Clears all media items from the playlist.
    pub fn clear(&mut self) {
        debug!("Clearing playlist");
        self.items.clear();
        info!("Playlist has been cleared");
    }

    /// Checks if there is a next media item in the playlist.
    ///
    /// Returns `true` if there is at least one item in the playlist, otherwise `false`.
    pub fn has_next(&self) -> bool {
        !self.items.is_empty()
    }

    /// Retrieves and removes the next media item from the playlist.
    ///
    /// Returns `Some` containing the boxed trait object implementing `MediaOverview` if there is a next item,
    /// or `None` if the playlist is empty.
    pub fn next(&mut self) -> Option<Box<dyn MediaOverview>> {
        self.items.pop_front()
    }
}

#[cfg(test)]
mod test {
    use crate::core::media::MovieOverview;

    use super::*;

    #[test]
    fn test_add() {
        let mut playlist = Playlist::default();
        let imdb_id = "tt00001";
        let media = Box::new(MovieOverview::new(
            "lorem".to_string(),
            imdb_id.to_string(),
            "2019".to_string()));

        playlist.add(media.clone());

        assert!(playlist.items.iter().position(|e| e.imdb_id() == imdb_id).is_some(), "expected the media item to have been added");
    }

    #[test]
    fn test_remove() {
        let mut playlist = Playlist::default();
        let imdb_id = "tt00013";
        let media = Box::new(MovieOverview::new(
            "ipsum".to_string(),
            imdb_id.to_string(),
            "2015".to_string())) as Box<dyn MediaOverview>;

        playlist.add(Box::new(MovieOverview::new(
            "ipsum".to_string(),
            imdb_id.to_string(),
            "2015".to_string())));
        playlist.remove(&media);

        assert!(playlist.items.iter().position(|e| e.imdb_id() == imdb_id).is_none(), "expected the media item to have been removed");
    }

    #[test]
    fn test_clear() {
        let mut playlist = Playlist::default();
        let imdb_id = "tt00001";
        let media = Box::new(MovieOverview::new(
            "lorem".to_string(),
            imdb_id.to_string(),
            "2019".to_string()));

        playlist.add(media.clone());
        assert!(playlist.items.iter().position(|e| e.imdb_id() == imdb_id).is_some(), "expected the added item to have been returned");

        playlist.clear();
        let result = playlist.items;
        assert_eq!(0, result.len(), "expected the playlist to have been cleared")
    }

    #[test]
    fn test_has_next() {
        let mut playlist = Playlist::default();
        let imdb_id = "tt00001";
        let media = Box::new(MovieOverview::new(
            "lorem".to_string(),
            imdb_id.to_string(),
            "2019".to_string()));

        playlist.add(media.clone());
        assert!(playlist.has_next(), "expected a next item to have been available");

        playlist.clear();
        assert!(!playlist.has_next(), "expected no next item to have been available")
    }

    #[test]
    fn test_next() {
        let mut playlist = Playlist::default();
        let imdb_id = "tt00001";
        let media = Box::new(MovieOverview::new(
            "lorem".to_string(),
            imdb_id.to_string(),
            "2019".to_string()));

        playlist.add(media.clone());
        let result = playlist.next();
        assert!(result.is_some(), "expected a next item to have been returned");
        assert!(!playlist.has_next(), "expected no next item to have been available")
    }
}