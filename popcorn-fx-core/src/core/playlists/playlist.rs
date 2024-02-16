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

/// Represents an item in a playlist, which can be a media file, a stream URL, or other media content.
#[derive(Debug, Display)]
#[display(fmt = "url: {:?}, title: {}, caption: {:?}, thumb: {:?}, media: {:?}, quality: {:?}, subtitles_enabled: {}", url, title, caption, thumb, media, quality, subtitles_enabled)]
pub struct PlaylistItem {
    /// The URL of the playlist item, if available.
    pub url: Option<String>,
    /// The title of the playlist item.
    pub title: String,
    /// A caption or description for the playlist item, if available.
    pub caption: Option<String>,
    /// The thumbnail URL of the playlist item, if available.
    pub thumb: Option<String>,
    /// The parent media identifier associated with the playlist item, if available.
    pub parent_media: Option<Box<dyn MediaIdentifier>>,
    /// The media identifier associated with the playlist item, if available.
    pub media: Option<Box<dyn MediaIdentifier>>,
    /// Information about the torrent associated with the playlist item, if available.
    pub torrent_info: Option<TorrentInfo>,
    /// Information about the torrent file associated with the playlist item, if available.
    pub torrent_file_info: Option<TorrentFileInfo>,
    /// The quality of the playlist item, if available.
    pub quality: Option<String>,
    /// The timestamp for auto-resume functionality, if available.
    pub auto_resume_timestamp: Option<u64>,
    /// Indicates whether subtitles are enabled for the playlist item.
    pub subtitles_enabled: bool,
}

impl PlaylistItem {
    /// Creates a new builder for constructing a `PlaylistItem`.
    pub fn builder() -> PlaylistItemBuilder {
        PlaylistItemBuilder::builder()
    }
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
            caption: self.caption.clone(),
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

/// A builder for constructing a `PlaylistItem`.
///
/// By default, `subtitles_enabled` is set to `false` if not provided before calling the `build` function.
#[derive(Debug, Default)]
pub struct PlaylistItemBuilder {
    url: Option<String>,
    title: Option<String>,
    caption: Option<String>,
    thumb: Option<String>,
    parent_media: Option<Box<dyn MediaIdentifier>>,
    media: Option<Box<dyn MediaIdentifier>>,
    torrent_info: Option<TorrentInfo>,
    torrent_file_info: Option<TorrentFileInfo>,
    quality: Option<String>,
    auto_resume_timestamp: Option<u64>,
    subtitles_enabled: Option<bool>,
}

impl PlaylistItemBuilder {
    /// Creates a new instance of `PlaylistItemBuilder`.
    pub fn builder() -> Self {
        Self::default()
    }

    /// Sets the URL of the playlist item.
    pub fn url<T: ToString>(mut self, url: T) -> Self {
        self.url = Some(url.to_string());
        self
    }

    /// Sets the title of the playlist item.
    pub fn title<T: ToString>(mut self, title: T) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Sets the caption of the playlist item.
    pub fn caption<T: ToString>(mut self, caption: T) -> Self {
        self.caption = Some(caption.to_string());
        self
    }

    /// Sets the thumbnail URL of the playlist item.
    pub fn thumb<T: ToString>(mut self, thumb: T) -> Self {
        self.thumb = Some(thumb.to_string());
        self
    }

    /// Sets the parent media identifier associated with the playlist item.
    pub fn parent_media(mut self, parent_media: Box<dyn MediaIdentifier>) -> Self {
        self.parent_media = Some(parent_media);
        self
    }

    /// Sets the media identifier associated with the playlist item.
    pub fn media(mut self, media: Box<dyn MediaIdentifier>) -> Self {
        self.media = Some(media);
        self
    }

    /// Sets the torrent information associated with the playlist item.
    pub fn torrent_info(mut self, torrent_info: TorrentInfo) -> Self {
        self.torrent_info = Some(torrent_info);
        self
    }

    /// Sets the torrent file information associated with the playlist item.
    pub fn torrent_file_info(mut self, torrent_file_info: TorrentFileInfo) -> Self {
        self.torrent_file_info = Some(torrent_file_info);
        self
    }

    /// Sets the quality of the playlist item.
    pub fn quality<T: ToString>(mut self, quality: T) -> Self {
        self.quality = Some(quality.to_string());
        self
    }

    /// Sets the auto-resume timestamp of the playlist item.
    pub fn auto_resume_timestamp(mut self, auto_resume_timestamp: u64) -> Self {
        self.auto_resume_timestamp = Some(auto_resume_timestamp);
        self
    }

    /// Sets whether subtitles are enabled for the playlist item.
    pub fn subtitles_enabled(mut self, subtitles_enabled: bool) -> Self {
        self.subtitles_enabled = Some(subtitles_enabled);
        self
    }

    /// Builds the `PlaylistItem` from the builder.
    ///
    /// # Panics
    ///
    /// Panics if `title` are not set.
    pub fn build(self) -> PlaylistItem {
        PlaylistItem {
            url: self.url,
            title: self.title.expect("title is not set"),
            caption: self.caption,
            thumb: self.thumb,
            parent_media: self.parent_media,
            media: self.media,
            torrent_info: self.torrent_info,
            torrent_file_info: self.torrent_file_info,
            quality: self.quality,
            auto_resume_timestamp: self.auto_resume_timestamp,
            subtitles_enabled: self.subtitles_enabled.unwrap_or_else(|| false),
        }
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
            caption: None,
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
            caption: None,
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
            caption: None,
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
            caption: None,
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
            caption: None,
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
            caption: None,
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
            caption: None,
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
            caption: None,
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