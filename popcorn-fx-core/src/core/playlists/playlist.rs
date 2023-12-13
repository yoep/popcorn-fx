use std::collections::VecDeque;

use derive_more::Display;
use log::{debug, info};

use crate::core::media::MediaIdentifier;

/// A struct representing a playlist of media items.
#[derive(Debug, Default)]
pub struct Playlist {
    pub items: VecDeque<PlaylistItem>,
}

impl Playlist {
    /// Adds a media item to the playlist.
    ///
    /// # Arguments
    ///
    /// * `media` - A boxed trait object implementing `MediaOverview` to be added to the playlist.
    pub fn add(&mut self, item: PlaylistItem) {
        debug!("Adding media item {:?} to playlist", item);
        self.items.push_back(item);
    }

    /// Removes a media item from the playlist based on its IMDb ID.
    ///
    /// # Arguments
    ///
    /// * `media` - A reference to a boxed trait object implementing `MediaOverview` to be removed from the playlist.
    pub fn remove(&mut self, item: &PlaylistItem) {
        let position = self.items.iter()
            .position(|e| e == item);

        if let Some(index) = position {
            debug!("Removing media item {} from playlist", item);
            self.items.remove(index);
        } else {
            debug!("Unable to remove media {} from the playlist, item not found", item);
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
    pub fn next(&mut self) -> Option<PlaylistItem> {
        self.items.pop_front()
    }
}

impl From<PlaylistItem> for Playlist {
    fn from(value: PlaylistItem) -> Self {
        let mut playlist = Playlist::default();
        playlist.add(value);
        playlist
    }
}

#[derive(Debug, Display)]
#[display(fmt = "url: {:?}, title: {}, thumb: {:?}, media: {:?}", url, title, thumb, media)]
pub struct PlaylistItem {
    pub url: Option<String>,
    pub title: String,
    pub thumb: Option<String>,
    pub media: Option<Box<dyn MediaIdentifier>>,
    pub quality: Option<String>,
    pub auto_resume_timestamp: Option<u64>,
    pub subtitles_enabled: bool,
}

impl Clone for PlaylistItem {
    fn clone(&self) -> Self {
        let cloned_media = match &self.media {
            None => None,
            Some(media) => {
                media.clone_identifier()
            }
        };

        Self {
            url: self.url.clone(),
            title: self.title.clone(),
            thumb: self.thumb.clone(),
            media: cloned_media,
            quality: self.quality.clone(),
            auto_resume_timestamp: self.auto_resume_timestamp,
            subtitles_enabled: self.subtitles_enabled,
        }
    }
}

impl PartialEq for PlaylistItem {
    fn eq(&self, other: &Self) -> bool {
        let mut media_equal = true;
        let mut thumb_equal = true;

        if let Some(media) = &self.media {
            if let Some(other_media) = &other.media {
                media_equal = media.imdb_id() == other_media.imdb_id();
            } else {
                media_equal = false;
            }
        }
        if let Some(thumb) = &self.thumb {
            if let Some(other_thumb) = &other.thumb {
                thumb_equal = thumb == other_thumb;
            } else {
                thumb_equal = false;
            }
        }

        self.url == other.url &&
            self.title.as_str() == other.title.as_str() &&
            thumb_equal &&
            media_equal &&
            self.quality == other.quality
    }
}

#[cfg(test)]
mod test {
    use crate::core::media::{MediaOverview, MovieOverview};
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_add() {
        let mut playlist = Playlist::default();
        let imdb_id = "tt00001";
        let media = Box::new(MovieOverview::new(
            "lorem".to_string(),
            imdb_id.to_string(),
            "2019".to_string()));

        playlist.add(PlaylistItem {
            url: None,
            title: "".to_string(),
            thumb: None,
            media: Some(media.clone()),
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        });

        assert!(playlist.items.iter().position(|e| e.media.as_ref().unwrap().imdb_id() == imdb_id).is_some(), "expected the media item to have been added");
    }

    #[test]
    fn test_remove() {
        let mut playlist = Playlist::default();
        let imdb_id = "tt00013";
        let media = Box::new(MovieOverview::new(
            "ipsum".to_string(),
            imdb_id.to_string(),
            "2015".to_string())) as Box<dyn MediaOverview>;
        let playlist_item = PlaylistItem {
            url: None,
            title: "".to_string(),
            thumb: None,
            media: Some(Box::new(MovieOverview::new(
                "ipsum".to_string(),
                imdb_id.to_string(),
                "2015".to_string()))),
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        };

        playlist.add(playlist_item.clone());
        playlist.remove(&playlist_item);

        assert!(playlist.items.iter().position(|e| e.media.as_ref().unwrap().imdb_id() == imdb_id).is_none(), "expected the media item to have been removed");
    }

    #[test]
    fn test_clear() {
        let mut playlist = Playlist::default();
        let imdb_id = "tt00001";
        let media = Box::new(MovieOverview::new(
            "lorem".to_string(),
            imdb_id.to_string(),
            "2019".to_string()));

        playlist.add(PlaylistItem {
            url: None,
            title: "".to_string(),
            thumb: None,
            media: Some(media.clone()),
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        });
        assert!(playlist.items.iter().position(|e| e.media.as_ref().unwrap().imdb_id() == imdb_id).is_some(), "expected the added item to have been returned");

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

        playlist.add(PlaylistItem {
            url: None,
            title: "".to_string(),
            thumb: None,
            media: Some(media.clone()),
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        });
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

        playlist.add(PlaylistItem {
            url: None,
            title: "".to_string(),
            thumb: None,
            media: Some(media.clone()),
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        });
        let result = playlist.next();
        assert!(result.is_some(), "expected a next item to have been returned");
        assert!(!playlist.has_next(), "expected no next item to have been available")
    }

    #[test]
    fn test_from_playlist_item() {
        init_logger();
        let item = PlaylistItem {
            url: None,
            title: "FooBar".to_string(),
            thumb: None,
            media: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        };

        let result = Playlist::from(item.clone());

        assert!(result.items.contains(&item))
    }
}