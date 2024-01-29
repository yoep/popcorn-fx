use std::collections::vec_deque::Iter;
use std::collections::VecDeque;

use derive_more::Display;
use log::{debug, info};

use crate::core::media::MediaIdentifier;
use crate::core::torrents::{TorrentFileInfo, TorrentInfo};

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

    /// Retrieves the next media item in the playlist without removing it.
    ///
    /// Returns `Some` containing a reference to the next media item if there is one,
    /// or `None` if the playlist is empty.
    pub fn next_as_ref(&self) -> Option<&PlaylistItem> {
        self.items.front()
    }

    /// Returns an iterator over the media items in the playlist.
    pub fn iter(&self) -> Iter<'_, PlaylistItem> {
        self.items.iter()
    }
}

impl From<PlaylistItem> for Playlist {
    fn from(value: PlaylistItem) -> Self {
        let mut playlist = Playlist::default();
        playlist.add(value);
        playlist
    }
}

impl FromIterator<PlaylistItem> for Playlist {
    fn from_iter<T: IntoIterator<Item=PlaylistItem>>(iter: T) -> Self {
        let mut playlist = Self::default();
        for item in iter {
            playlist.add(item);
        }
        playlist
    }
}

#[derive(Debug, Display)]
#[display(fmt = "url: {:?}, title: {}, thumb: {:?}, media: {:?}, quality: {:?}, subtitles_enabled: {}", url, title, thumb, media, quality, subtitles_enabled)]
pub struct PlaylistItem {
    pub url: Option<String>,
    pub title: String,
    pub thumb: Option<String>,
    pub parent_media: Option<Box<dyn MediaIdentifier>>,
    pub media: Option<Box<dyn MediaIdentifier>>,
    pub torrent_info: Option<TorrentInfo>,
    pub torrent_file_info: Option<TorrentFileInfo>,
    pub quality: Option<String>,
    pub auto_resume_timestamp: Option<u64>,
    pub subtitles_enabled: bool,
}

impl Clone for PlaylistItem {
    fn clone(&self) -> Self {
        let cloned_parent_media = match &self.parent_media {
            None => None,
            Some(media) => {
                media.clone_identifier()
            }
        };
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
            parent_media: cloned_parent_media,
            media: cloned_media,
            torrent_info: self.torrent_info.clone(),
            torrent_file_info: self.torrent_file_info.clone(),
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
    use crate::core::media::MovieOverview;
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
            parent_media: None,
            media: Some(media.clone()),
            torrent_info: None,
            torrent_file_info: None,
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
        let playlist_item = PlaylistItem {
            url: None,
            title: "".to_string(),
            thumb: None,
            parent_media: None,
            media: Some(Box::new(MovieOverview::new(
                "ipsum".to_string(),
                imdb_id.to_string(),
                "2015".to_string()))),
            torrent_info: None,
            torrent_file_info: None,
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
            parent_media: None,
            media: Some(media.clone()),
            torrent_info: None,
            torrent_file_info: None,
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
            parent_media: None,
            media: Some(media.clone()),
            torrent_info: None,
            torrent_file_info: None,
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
            parent_media: None,
            media: Some(media.clone()),
            torrent_info: None,
            torrent_file_info: None,
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
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        };

        let result = Playlist::from(item.clone());

        assert!(result.items.contains(&item))
    }

    #[test]
    fn test_playlist_iter() {
        let title = "FooBar123";
        let playlist = Playlist::from(PlaylistItem {
            url: None,
            title: title.to_string(),
            thumb: None,
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        });

        let result = playlist.iter();

        assert_eq!(1, result.len());
        for e in result {
            assert_eq!(title, e.title.as_str())
        }
    }

    #[test]
    fn test_playlist_from_iter() {
        let title = "FooBar123";

        let result: Playlist = vec![PlaylistItem {
            url: None,
            title: title.to_string(),
            thumb: None,
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        }]
            .into_iter()
            .collect();

        assert_eq!(1, result.items.len());
        assert_eq!(title, result.items.get(0).unwrap().title.as_str());
    }
}