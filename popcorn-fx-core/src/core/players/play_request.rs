use std::fmt::Debug;

use downcast_rs::{DowncastSync, impl_downcast};
#[cfg(any(test, feature = "testing"))]
use mockall::automock;

use crate::core::media::MediaIdentifier;
use crate::core::playlists::PlaylistItem;

/// A trait representing a play request for media playback.
#[cfg_attr(any(test, feature = "testing"), automock)]
pub trait PlayRequest: Debug + DowncastSync {
    /// Get the URL of the media to be played.
    fn url(&self) -> &str;

    /// Get the title of the media (if available).
    ///
    /// Returns an optional `String` containing the title, or `None` if not available.
    fn title(&self) -> &str;

    /// Get the URL of the thumbnail associated with the media (if available).
    ///
    /// Returns an optional `String` containing the thumbnail URL, or `None` if not available.
    fn thumbnail(&self) -> Option<String>;

    /// Get the quality information of the media (if available).
    ///
    /// Returns an optional `String` containing quality information, or `None` if not available.
    fn quality(&self) -> Option<String>;

    /// Get the auto-resume timestamp for the media playback (if available).
    ///
    /// Returns an optional `u64` representing the auto-resume timestamp in seconds, or `None` if not available.
    fn auto_resume_timestamp(&self) -> Option<u64>;

    /// Check if subtitles are enabled for the media playback.
    ///
    /// Returns `true` if subtitles are enabled, `false` otherwise.
    fn subtitles_enabled(&self) -> bool;
}
impl_downcast!(sync PlayRequest);

/// A struct representing a play request for a URL-based media.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayUrlRequest {
    /// The URL of the media to be played.
    pub url: String,
    /// The title of the media (if available).
    pub title: String,
    /// The URL of the thumbnail associated with the media (if available).
    pub thumb: Option<String>,
    /// The auto-resume timestamp for media playback (if available).
    pub auto_resume_timestamp: Option<u64>,
    /// Indicates whether subtitles are enabled for the media playback.
    pub subtitles_enabled: bool,
}

/// Implementing the `PlayRequest` trait for `PlayUrlRequest`.
impl PlayRequest for PlayUrlRequest {
    fn url(&self) -> &str {
        self.url.as_str()
    }

    fn title(&self) -> &str {
        self.title.as_str()
    }

    fn thumbnail(&self) -> Option<String> {
        self.thumb.clone()
    }

    fn quality(&self) -> Option<String> {
        None
    }

    fn auto_resume_timestamp(&self) -> Option<u64> {
        self.auto_resume_timestamp.clone()
    }

    fn subtitles_enabled(&self) -> bool {
        self.subtitles_enabled
    }
}

impl From<PlaylistItem> for PlayUrlRequest {
    fn from(value: PlaylistItem) -> Self {
        let mut builder = PlayUrlRequestBuilder::builder()
            .url(value.url.expect("expected an url to have been present").as_str())
            .title(value.title.as_str())
            .subtitles_enabled(value.subtitles_enabled);

        if let Some(e) = value.thumb {
            builder = builder.thumb(e.as_str());
        }
        if let Some(e) = value.auto_resume_timestamp {
            builder = builder.auto_resume_timestamp(e);
        }

        builder.build()
    }
}

/// A builder for constructing a `PlayUrlRequest` with optional parameters.
#[derive(Debug, Default)]
pub struct PlayUrlRequestBuilder {
    url: Option<String>,
    title: Option<String>,
    thumb: Option<String>,
    auto_resume_timestamp: Option<u64>,
    subtitles_enabled: bool,
}

impl PlayUrlRequestBuilder {
    /// Creates a new instance of the builder with default values.
    pub fn builder() -> Self {
        Default::default()
    }

    /// Sets the URL for the media to be played.
    pub fn url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }

    /// Sets the title of the media.
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Sets the URL of the thumbnail associated with the media.
    pub fn thumb(mut self, thumb: &str) -> Self {
        self.thumb = Some(thumb.to_string());
        self
    }

    /// Sets the auto-resume timestamp for media playback.
    pub fn auto_resume_timestamp(mut self, auto_resume_timestamp: u64) -> Self {
        self.auto_resume_timestamp = Some(auto_resume_timestamp);
        self
    }

    /// Sets whether subtitles are enabled for the media playback.
    pub fn subtitles_enabled(mut self, subtitles_enabled: bool) -> Self {
        self.subtitles_enabled = subtitles_enabled;
        self
    }

    /// Builds and returns a `PlayUrlRequest` based on the provided parameters.
    ///
    /// # Panics
    ///
    /// Panics if the required field (`url`) is not provided.
    pub fn build(self) -> PlayUrlRequest {
        PlayUrlRequest {
            url: self.url.expect("url has not been set"),
            title: self.title.expect("title has not been set"),
            thumb: self.thumb,
            auto_resume_timestamp: self.auto_resume_timestamp,
            subtitles_enabled: self.subtitles_enabled,
        }
    }
}

/// A struct representing a play request for media with additional metadata.
#[derive(Debug)]
pub struct PlayMediaRequest {
    /// The base play request for URL-based media.
    pub base: PlayUrlRequest,
    /// The parent media identifier (if applicable).
    pub parent_media: Option<Box<dyn MediaIdentifier>>,
    /// The media identifier for the requested media.
    pub media: Box<dyn MediaIdentifier>,
    /// The quality information for the media.
    pub quality: String,
}

impl PlayMediaRequest {
    /// Create a new `PlayMediaRequest` with the specified parameters.
    pub fn new(
        url: String,
        title: String,
        thumb: Option<String>,
        auto_resume_timestamp: Option<u64>,
        subtitles_enabled: bool,
        media: Box<dyn MediaIdentifier>,
        parent_media: Option<Box<dyn MediaIdentifier>>,
        quality: String,
    ) -> Self {
        let base = PlayUrlRequest {
            url,
            title,
            thumb,
            auto_resume_timestamp,
            subtitles_enabled,
        };

        Self {
            base,
            parent_media,
            media,
            quality,
        }
    }
}

impl PartialEq for PlayMediaRequest {
    fn eq(&self, other: &Self) -> bool {
        self.base.eq(&other.base) &&
            self.parent_media.is_some() == other.parent_media.is_some() &&
            self.media.imdb_id() == other.media.imdb_id() &&
            self.quality == other.quality
    }
}

/// Implementing the `PlayRequest` trait for `PlayMediaRequest`.
impl PlayRequest for PlayMediaRequest {
    fn url(&self) -> &str {
        self.base.url()
    }

    fn title(&self) -> &str {
        self.base.title()
    }

    fn thumbnail(&self) -> Option<String> {
        self.base.thumbnail()
    }

    fn quality(&self) -> Option<String> {
        Some(self.quality.clone())
    }

    fn auto_resume_timestamp(&self) -> Option<u64> {
        self.base.auto_resume_timestamp()
    }

    fn subtitles_enabled(&self) -> bool {
        self.base.subtitles_enabled()
    }
}

impl From<PlaylistItem> for PlayMediaRequest {
    fn from(value: PlaylistItem) -> Self {
        let mut builder = PlayMediaRequestBuilder::builder()
            .url(value.url.expect("expected a url to have been present").as_str())
            .title(value.title.as_str())
            .media(value.media.expect("expected a media item to have been present"))
            .subtitles_enabled(value.subtitles_enabled);

        if let Some(e) = value.thumb {
            builder = builder.thumb(e.as_str());
        }
        if let Some(e) = value.auto_resume_timestamp {
            builder = builder.auto_resume_timestamp(e);
        }
        if let Some(e) = value.parent_media {
            builder = builder.parent_media(e);
        }
        if let Some(e) = value.quality {
            builder = builder.quality(e.as_str());
        }

        builder.build()
    }
}

/// A builder for constructing a `PlayMediaRequest` with optional parameters.
#[derive(Debug, Default)]
pub struct PlayMediaRequestBuilder {
    url: Option<String>,
    title: Option<String>,
    thumb: Option<String>,
    auto_resume_timestamp: Option<u64>,
    subtitles_enabled: bool,
    media: Option<Box<dyn MediaIdentifier>>,
    parent_media: Option<Box<dyn MediaIdentifier>>,
    quality: Option<String>,
}

impl PlayMediaRequestBuilder {
    /// Creates a new instance of the builder with default values.
    pub fn builder() -> Self {
        PlayMediaRequestBuilder::default()
    }

    /// Sets the URL for the media to be played.
    pub fn url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }

    /// Sets the title of the media.
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Sets the URL of the thumbnail associated with the media.
    pub fn thumb(mut self, thumb: &str) -> Self {
        self.thumb = Some(thumb.to_string());
        self
    }

    /// Sets the auto-resume timestamp for media playback.
    pub fn auto_resume_timestamp(mut self, auto_resume_timestamp: u64) -> Self {
        self.auto_resume_timestamp = Some(auto_resume_timestamp);
        self
    }

    /// Sets whether subtitles are enabled for the media playback.
    pub fn subtitles_enabled(mut self, subtitles_enabled: bool) -> Self {
        self.subtitles_enabled = subtitles_enabled;
        self
    }

    /// Sets the media identifier for the requested media.
    pub fn media(mut self, media: Box<dyn MediaIdentifier>) -> Self {
        self.media = Some(media);
        self
    }

    /// Sets the parent media identifier (if applicable).
    pub fn parent_media(mut self, parent_media: Box<dyn MediaIdentifier>) -> Self {
        self.parent_media = Some(parent_media);
        self
    }

    /// Sets the quality information for the media.
    pub fn quality(mut self, quality: &str) -> Self {
        self.quality = Some(quality.to_string());
        self
    }

    /// Builds and returns a `PlayMediaRequest` based on the provided parameters.
    ///
    /// # Panics
    ///
    /// Panics if the required fields (`url`, `title`, and `media`) are not provided.
    pub fn build(self) -> PlayMediaRequest {
        if self.url.is_none() || self.title.is_none() || self.media.is_none() {
            panic!("url, title, and media fields must be provided to build PlayMediaRequest");
        }

        let base = PlayUrlRequest {
            url: self.url.unwrap(),
            title: self.title.unwrap(),
            thumb: self.thumb,
            auto_resume_timestamp: self.auto_resume_timestamp,
            subtitles_enabled: self.subtitles_enabled,
        };

        PlayMediaRequest {
            base,
            parent_media: self.parent_media,
            media: self.media.unwrap(),
            quality: self.quality.unwrap_or_else(|| "".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::media::{Episode, ShowOverview};

    use super::*;

    #[test]
    fn test_play_url_request_builder() {
        let url = "https://localhost:8054/my-video.mp4";
        let title = "DolorEsta";
        let thumb = "https://imgur.com/something.jpg";
        let auto_resume = 84000u64;
        let expected_result = PlayUrlRequest {
            url: url.to_string(),
            title: title.to_string(),
            thumb: Some(thumb.to_string()),
            auto_resume_timestamp: Some(auto_resume),
            subtitles_enabled: true,
        };

        let result = PlayUrlRequestBuilder::builder()
            .url(url)
            .title(title)
            .thumb(thumb)
            .auto_resume_timestamp(auto_resume)
            .subtitles_enabled(true)
            .build();

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_player_url_request_from() {
        let url = "http://localhost:8090/my-video.mkv";
        let title = "MyVideoItem";
        let auto_resume = 50000u64;
        let item = PlaylistItem {
            url: Some(url.to_string()),
            title: title.to_string(),
            thumb: None,
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: Some(auto_resume),
            subtitles_enabled: false,
        };
        let expected_result = PlayUrlRequest {
            url: url.to_string(),
            title: title.to_string(),
            thumb: None,
            auto_resume_timestamp: Some(auto_resume),
            subtitles_enabled: false,
        };

        let result = PlayUrlRequest::from(item);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_play_media_request_builder() {
        let url = "https://localhost:8054/my-video.mp4";
        let title = "DolorEsta";
        let thumb = "https://imgur.com/something.jpg";
        let quality = "720p";
        let show = ShowOverview {
            imdb_id: "tt2157488".to_string(),
            tvdb_id: "".to_string(),
            title: "".to_string(),
            year: "".to_string(),
            num_seasons: 0,
            images: Default::default(),
            rating: None,
        };
        let episode = Episode {
            season: 0,
            episode: 0,
            first_aired: 0,
            title: "".to_string(),
            overview: "".to_string(),
            tvdb_id: 0,
            tvdb_id_value: "".to_string(),
            thumb: None,
            torrents: Default::default(),
        };
        let expected_result = PlayMediaRequest {
            base: PlayUrlRequest {
                url: url.to_string(),
                title: title.to_string(),
                thumb: Some(thumb.to_string()),
                auto_resume_timestamp: None,
                subtitles_enabled: false,
            },
            parent_media: Some(Box::new(show.clone())),
            media: Box::new(episode.clone()),
            quality: quality.to_string(),
        };

        let result = PlayMediaRequestBuilder::builder()
            .url(url)
            .title(title)
            .thumb(thumb)
            .quality(quality)
            .parent_media(Box::new(show))
            .media(Box::new(episode))
            .build();

        assert_eq!(expected_result, result)
    }
}