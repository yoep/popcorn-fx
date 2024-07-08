use std::fmt::Formatter;
use std::fmt::{Debug, Display};
use std::sync::Weak;

use derive_more::Display;
use downcast_rs::{impl_downcast, DowncastSync};
#[cfg(any(test, feature = "testing"))]
use mockall::automock;

use crate::core::loader::LoadingData;
use crate::core::media::MediaIdentifier;
use crate::core::subtitles::model::{Subtitle, SubtitleInfo};
use crate::core::torrents::TorrentStream;

#[derive(Debug, Clone, PartialEq)]
pub struct PlaySubtitleRequest {
    pub enabled: bool,
    pub info: Option<SubtitleInfo>,
    pub subtitle: Option<Subtitle>,
}

/// A trait representing a play request for media playback.
#[cfg_attr(any(test, feature = "testing"), automock)]
pub trait PlayRequest: Debug + Display + DowncastSync {
    /// Get the URL of the media to be played.
    fn url(&self) -> &str;

    /// Get the title of the media (if available).
    ///
    /// Returns a `String` containing the title.
    fn title(&self) -> &str;

    /// Get the optional caption of this request.
    ///
    /// Returns an optional `String` containing the caption of the request, or `None` if not available.
    fn caption(&self) -> Option<String>;

    /// Get the URL of the thumbnail associated with the media (if available).
    ///
    /// Returns an optional `String` containing the thumbnail URL, or `None` if not available.
    fn thumbnail(&self) -> Option<String>;

    /// Get the URL of the background image associated with the media (if available).
    /// This image can be shown during the loading of the media stream.
    ///
    /// Returns an optional `String` containing the background URL, or `None` if not available.
    fn background(&self) -> Option<String>;

    /// Get the quality information of the media (if available).
    ///
    /// Returns an optional `String` containing quality information, or `None` if not available.
    fn quality(&self) -> Option<String>;

    /// Get the auto-resume timestamp for the media playback (if available).
    ///
    /// Returns an optional `u64` representing the auto-resume timestamp in seconds, or `None` if not available.
    fn auto_resume_timestamp(&self) -> Option<u64>;

    /// Get the subtitle playback information for the media.
    ///
    /// Returns a reference to the `PlaySubtitleRequest` information.
    fn subtitle(&self) -> &PlaySubtitleRequest;
}
impl_downcast!(sync PlayRequest);

#[cfg(any(test, feature = "testing"))]
impl Display for MockPlayRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockPlayRequest")
    }
}

/// A struct representing a play request for a URL-based media.
#[derive(Display, Clone, PartialEq)]
#[display(fmt = "{}", title)]
pub struct PlayUrlRequest {
    /// The URL of the media to be played.
    pub url: String,
    /// The title of the media.
    pub title: String,
    /// The caption of the media request (if available).
    pub caption: Option<String>,
    /// The URL of the thumbnail associated with the media (if available).
    pub thumb: Option<String>,
    /// The URL of the background image associated with the media (if available).
    pub background: Option<String>,
    /// The auto-resume timestamp for media playback (if available).
    pub auto_resume_timestamp: Option<u64>,
    /// The subtitle playback information for this request.
    pub subtitle: PlaySubtitleRequest,
}

impl PlayUrlRequest {
    pub fn builder() -> PlayUrlRequestBuilder {
        PlayUrlRequestBuilder::builder()
    }
}

/// Implementing the `PlayRequest` trait for `PlayUrlRequest`.
impl PlayRequest for PlayUrlRequest {
    fn url(&self) -> &str {
        self.url.as_str()
    }

    fn title(&self) -> &str {
        self.title.as_str()
    }

    fn caption(&self) -> Option<String> {
        self.caption.clone()
    }

    fn thumbnail(&self) -> Option<String> {
        self.thumb.clone()
    }

    fn background(&self) -> Option<String> {
        self.background.clone()
    }

    fn quality(&self) -> Option<String> {
        None
    }

    fn auto_resume_timestamp(&self) -> Option<u64> {
        self.auto_resume_timestamp.clone()
    }

    fn subtitle(&self) -> &PlaySubtitleRequest {
        &self.subtitle
    }
}

impl Debug for PlayUrlRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlayUrlRequest")
            .field("url", &self.url)
            .field("title", &self.title)
            .field("caption", &self.caption)
            .field("thumb", &self.thumb)
            .field("background", &self.background)
            .field("auto_resume_timestamp", &self.auto_resume_timestamp)
            .field("subtitle", &self.subtitle)
            .finish()
    }
}

impl<S> From<S> for PlayUrlRequest
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        PlayUrlRequestBuilder::builder()
            .url(value.into().as_str())
            .title("")
            .build()
    }
}

impl From<LoadingData> for PlayUrlRequest {
    fn from(value: LoadingData) -> Self {
        let subtitles_enabled = value.subtitle.enabled.unwrap_or(false);
        let mut builder = PlayUrlRequestBuilder::builder()
            .url(
                value
                    .url
                    .expect("expected an url to have been present")
                    .as_str(),
            )
            .title(value.title.unwrap_or(String::new()).as_str())
            .subtitles_enabled(subtitles_enabled);

        if let Some(e) = value.thumb {
            builder = builder.thumb(e.as_str());
        }
        if let Some(e) = value.auto_resume_timestamp {
            builder = builder.auto_resume_timestamp(e);
        }
        if subtitles_enabled {
            if let Some(e) = value.subtitle.subtitle {
                builder = builder.subtitle(e);
            }
        }

        builder.build()
    }
}

/// A builder for constructing a `PlayUrlRequest` with optional parameters.
#[derive(Debug, Default, Clone)]
pub struct PlayUrlRequestBuilder {
    url: Option<String>,
    title: Option<String>,
    caption: Option<String>,
    thumb: Option<String>,
    background: Option<String>,
    auto_resume_timestamp: Option<u64>,
    subtitles_enabled: bool,
    subtitle_info: Option<SubtitleInfo>,
    subtitle: Option<Subtitle>,
}

impl PlayUrlRequestBuilder {
    /// Creates a new instance of the builder with default values.
    pub fn builder() -> Self {
        Default::default()
    }

    /// Sets the URL for the media to be played.
    pub fn url<S: Into<String>>(mut self, url: S) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Sets the title of the media.
    pub fn title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the caption of the associated media.
    pub fn caption<S: Into<String>>(mut self, caption: S) -> Self
    where
        S: Into<String>,
    {
        self.caption = Some(caption.into());
        self
    }

    /// Sets the URL of the thumbnail associated with the media.
    pub fn thumb<S: Into<String>>(mut self, thumb: S) -> Self {
        self.thumb = Some(thumb.into());
        self
    }

    /// Sets the URL of the background associated with the media.
    pub fn background<S: Into<String>>(mut self, background: S) -> Self {
        self.background = Some(background.into());
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

    /// Sets the selected subtitle for the media playback.
    pub fn subtitle_info(mut self, subtitle_info: SubtitleInfo) -> Self {
        self.subtitle_info = Some(subtitle_info);
        self
    }

    /// Sets the selected subtitle for the media playback.
    pub fn subtitle(mut self, subtitle: Subtitle) -> Self {
        self.subtitle = Some(subtitle);
        self
    }

    /// Builds and returns a `PlayUrlRequest` based on the provided parameters.
    ///
    /// # Panics
    ///
    /// Panics if the required fields (`url`, `title`) is not provided.
    pub fn build(self) -> PlayUrlRequest {
        PlayUrlRequest {
            url: self.url.expect("url has not been set"),
            title: self.title.expect("title has not been set"),
            caption: self.caption,
            thumb: self.thumb,
            background: self.background,
            auto_resume_timestamp: self.auto_resume_timestamp,
            subtitle: PlaySubtitleRequest {
                enabled: self.subtitles_enabled,
                info: self.subtitle_info,
                subtitle: self.subtitle,
            },
        }
    }
}

/// Represents a request for streaming media.
#[derive(Debug, Display, Clone)]
#[display(fmt = "{}", base)]
pub struct PlayStreamRequest {
    /// The base play request for URL-based media.
    pub base: PlayUrlRequest,
    /// The quality of the media.
    pub quality: Option<String>,
    /// The torrent stream being used to stream the media item.
    pub torrent_stream: Weak<Box<dyn TorrentStream>>,
}

impl PlayStreamRequest {
    /// Creates a new builder for `PlayStreamRequest`.
    pub fn builder() -> PlayStreamRequestBuilder {
        PlayStreamRequestBuilder::builder()
    }
}

impl PlayRequest for PlayStreamRequest {
    fn url(&self) -> &str {
        self.base.url()
    }

    fn title(&self) -> &str {
        self.base.title()
    }

    fn caption(&self) -> Option<String> {
        self.base.caption()
    }

    fn thumbnail(&self) -> Option<String> {
        self.base.thumbnail()
    }

    fn background(&self) -> Option<String> {
        self.base.background()
    }

    fn quality(&self) -> Option<String> {
        self.quality.clone()
    }

    fn auto_resume_timestamp(&self) -> Option<u64> {
        self.base.auto_resume_timestamp()
    }

    fn subtitle(&self) -> &PlaySubtitleRequest {
        self.base.subtitle()
    }
}

impl PartialEq for PlayStreamRequest {
    fn eq(&self, other: &Self) -> bool {
        self.base.eq(&other.base) && self.quality == other.quality
    }
}

impl From<LoadingData> for PlayStreamRequest {
    fn from(value: LoadingData) -> Self {
        let subtitles_enabled = value.subtitle.enabled.unwrap_or(false);
        let mut builder = Self::builder()
            .url(
                value
                    .url
                    .expect("expected a url to have been present")
                    .as_str(),
            )
            .title(
                value
                    .title
                    .expect("expected a title to have been present")
                    .as_str(),
            )
            .subtitles_enabled(subtitles_enabled);

        if let Some(e) = value.caption {
            builder = builder.caption(e);
        }
        if let Some(e) = value.thumb {
            builder = builder.thumb(e);
        }
        if let Some(e) = value.auto_resume_timestamp {
            builder = builder.auto_resume_timestamp(e);
        }
        if let Some(e) = value.quality {
            builder = builder.quality(e.as_str());
        }
        if let Some(e) = value.torrent_stream {
            builder = builder.torrent_stream(e);
        }
        if subtitles_enabled {
            if let Some(e) = value.subtitle.subtitle {
                builder = builder.subtitle(e);
            }
        }

        builder.build()
    }
}

/// A builder for `PlayStreamRequest`.
#[derive(Debug, Default)]
pub struct PlayStreamRequestBuilder {
    url: Option<String>,
    title: Option<String>,
    caption: Option<String>,
    thumb: Option<String>,
    background: Option<String>,
    auto_resume_timestamp: Option<u64>,
    subtitles_enabled: bool,
    subtitle_info: Option<SubtitleInfo>,
    subtitle: Option<Subtitle>,
    quality: Option<String>,
    torrent_stream: Option<Weak<Box<dyn TorrentStream>>>,
}

impl PlayStreamRequestBuilder {
    /// Creates a new instance of the builder with default values.
    pub fn builder() -> Self {
        Self::default()
    }

    /// Sets the URL for the media to be played.
    pub fn url<S>(mut self, url: S) -> Self
    where
        S: Into<String>,
    {
        self.url = Some(url.into());
        self
    }

    /// Sets the title of the media.
    pub fn title<S>(mut self, title: S) -> Self
    where
        S: Into<String>,
    {
        self.title = Some(title.into());
        self
    }

    /// Sets the caption of the media.
    pub fn caption<S>(mut self, caption: S) -> Self
    where
        S: Into<String>,
    {
        self.caption = Some(caption.into());
        self
    }

    /// Sets the URL of the thumbnail associated with the media.
    pub fn thumb<S>(mut self, thumb: S) -> Self
    where
        S: Into<String>,
    {
        self.thumb = Some(thumb.into());
        self
    }

    /// Sets the URL of the background associated with the media.
    pub fn background<S>(mut self, background: S) -> Self
    where
        S: Into<String>,
    {
        self.background = Some(background.into());
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

    /// Sets the subtitle information for the media.
    pub fn subtitle_info(mut self, subtitle_info: SubtitleInfo) -> Self {
        self.subtitle_info = Some(subtitle_info);
        self
    }

    /// Sets the selected subtitle for the media.
    pub fn subtitle(mut self, subtitle: Subtitle) -> Self {
        self.subtitle = Some(subtitle);
        self
    }

    /// Sets the quality information for the media.
    pub fn quality<S>(mut self, quality: S) -> Self
    where
        S: Into<String>,
    {
        self.quality = Some(quality.into());
        self
    }

    /// Sets the torrent stream of the media.
    pub fn torrent_stream(mut self, torrent_stream: Weak<Box<dyn TorrentStream>>) -> Self {
        self.torrent_stream = Some(torrent_stream);
        self
    }

    /// Builds the `PlayStreamRequest`.
    ///
    /// # Panics
    ///
    /// This method will panic if the `url` or `title` fields are not provided.
    pub fn build(self) -> PlayStreamRequest {
        if self.url.is_none() || self.title.is_none() {
            panic!("url and title fields must be provided to build PlayMediaRequest");
        }

        let base = PlayUrlRequest {
            url: self.url.unwrap(),
            title: self.title.unwrap(),
            caption: self.caption,
            thumb: self.thumb,
            background: self.background,
            auto_resume_timestamp: self.auto_resume_timestamp,
            subtitle: PlaySubtitleRequest {
                enabled: self.subtitles_enabled,
                info: self.subtitle_info,
                subtitle: self.subtitle,
            },
        };

        PlayStreamRequest {
            base,
            quality: self.quality,
            torrent_stream: self
                .torrent_stream
                .expect("torrent_stream has not been set"),
        }
    }
}

/// A struct representing a play request for media with additional metadata.
#[derive(Debug, Display)]
#[display(fmt = "{}", base)]
pub struct PlayMediaRequest {
    /// The base play request for URL-based media.
    pub base: PlayUrlRequest,
    /// The parent media identifier (if applicable).
    pub parent_media: Option<Box<dyn MediaIdentifier>>,
    /// The media identifier for the requested media.
    pub media: Box<dyn MediaIdentifier>,
    /// The quality information for the media.
    pub quality: String,
    /// The torrent stream that is being used to stream the media item
    pub torrent_stream: Weak<Box<dyn TorrentStream>>,
}

impl PlayMediaRequest {
    pub fn builder() -> PlayMediaRequestBuilder {
        PlayMediaRequestBuilder::default()
    }
}

impl PartialEq for PlayMediaRequest {
    fn eq(&self, other: &Self) -> bool {
        self.base.eq(&other.base)
            && self.parent_media.is_some() == other.parent_media.is_some()
            && self.media.imdb_id() == other.media.imdb_id()
            && self.quality == other.quality
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

    fn caption(&self) -> Option<String> {
        self.base.caption()
    }

    fn thumbnail(&self) -> Option<String> {
        self.base.thumbnail()
    }

    fn background(&self) -> Option<String> {
        self.base.background()
    }

    fn quality(&self) -> Option<String> {
        Some(self.quality.clone())
    }

    fn auto_resume_timestamp(&self) -> Option<u64> {
        self.base.auto_resume_timestamp()
    }

    fn subtitle(&self) -> &PlaySubtitleRequest {
        self.base.subtitle()
    }
}

impl Clone for PlayMediaRequest {
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            parent_media: self
                .parent_media
                .as_ref()
                .and_then(|e| e.clone_identifier()),
            media: self
                .media
                .clone_identifier()
                .expect("expected the media identifier to have been cloned"),
            quality: self.quality.clone(),
            torrent_stream: self.torrent_stream.clone(),
        }
    }
}

impl From<LoadingData> for PlayMediaRequest {
    fn from(value: LoadingData) -> Self {
        let subtitles_enabled = value.subtitle.enabled.unwrap_or(false);
        let mut builder = Self::builder()
            .url(
                value
                    .url
                    .expect("expected a url to have been present")
                    .as_str(),
            )
            .title(
                value
                    .title
                    .expect("expected a title to have been present")
                    .as_str(),
            )
            .subtitles_enabled(subtitles_enabled);

        if let Some(e) = value.caption {
            builder = builder.caption(e);
        }
        if let Some(e) = value.thumb {
            builder = builder.thumb(e);
        }
        if let Some(media_identifier) = value.media.as_ref() {
            if let Some(media) = media_identifier.into_overview() {
                builder = builder.background(media.images().fanart());
            }
        }
        if let Some(e) = value.auto_resume_timestamp {
            builder = builder.auto_resume_timestamp(e);
        }
        if let Some(media_identifier) = value.parent_media {
            if let Some(media) = media_identifier.into_overview() {
                builder = builder.background(media.images().fanart());
            }

            builder = builder.parent_media(media_identifier);
        }
        if let Some(e) = value.quality {
            builder = builder.quality(e.as_str());
        }
        if let Some(e) = value.torrent_stream {
            builder = builder.torrent_stream(e);
        }
        if subtitles_enabled {
            if let Some(e) = value.subtitle.subtitle {
                builder = builder.subtitle(e);
            }
        }

        builder
            .media(
                value
                    .media
                    .expect("expected a media item to have been present"),
            )
            .build()
    }
}

/// A builder for constructing a `PlayMediaRequest` with optional parameters.
#[derive(Debug, Default)]
pub struct PlayMediaRequestBuilder {
    url: Option<String>,
    title: Option<String>,
    caption: Option<String>,
    thumb: Option<String>,
    background: Option<String>,
    auto_resume_timestamp: Option<u64>,
    subtitles_enabled: bool,
    subtitle_info: Option<SubtitleInfo>,
    subtitle: Option<Subtitle>,
    media: Option<Box<dyn MediaIdentifier>>,
    parent_media: Option<Box<dyn MediaIdentifier>>,
    quality: Option<String>,
    torrent_stream: Option<Weak<Box<dyn TorrentStream>>>,
}

impl PlayMediaRequestBuilder {
    /// Creates a new instance of the builder with default values.
    pub fn builder() -> Self {
        Self::default()
    }

    /// Sets the URL for the media to be played.
    pub fn url<S>(mut self, url: S) -> Self
    where
        S: Into<String>,
    {
        self.url = Some(url.into());
        self
    }

    /// Sets the title of the media.
    pub fn title<S>(mut self, title: S) -> Self
    where
        S: Into<String>,
    {
        self.title = Some(title.into());
        self
    }

    /// Sets the caption of the media.
    pub fn caption<S>(mut self, caption: S) -> Self
    where
        S: Into<String>,
    {
        self.caption = Some(caption.into());
        self
    }

    /// Sets the URL of the thumbnail associated with the media.
    pub fn thumb<S>(mut self, thumb: S) -> Self
    where
        S: Into<String>,
    {
        self.thumb = Some(thumb.into());
        self
    }

    /// Sets the URL of the background associated with the media.
    pub fn background<S>(mut self, background: S) -> Self
    where
        S: Into<String>,
    {
        self.background = Some(background.into());
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

    /// Sets the subtitle information for the media playback.
    pub fn subtitle_info(mut self, subtitle_info: SubtitleInfo) -> Self {
        self.subtitle_info = Some(subtitle_info);
        self
    }

    /// Sets the selected subtitle for the media playback.
    pub fn subtitle(mut self, subtitle: Subtitle) -> Self {
        self.subtitle = Some(subtitle);
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
    pub fn quality<S>(mut self, quality: S) -> Self
    where
        S: Into<String>,
    {
        self.quality = Some(quality.into());
        self
    }

    /// Sets the torrent stream of the media.
    pub fn torrent_stream(mut self, torrent_stream: Weak<Box<dyn TorrentStream>>) -> Self {
        self.torrent_stream = Some(torrent_stream);
        self
    }

    /// Builds and returns a `PlayMediaRequest` based on the provided parameters.
    ///
    /// # Panics
    ///
    /// Panics if the required fields (`url`, `title`, and `media`) are not provided.
    pub fn build(self) -> PlayMediaRequest {
        if self.url.is_none() || self.title.is_none() {
            panic!("url and title fields must be provided to build PlayMediaRequest");
        }

        let base = PlayUrlRequest {
            url: self.url.unwrap(),
            title: self.title.unwrap(),
            caption: self.caption,
            thumb: self.thumb,
            background: self.background,
            auto_resume_timestamp: self.auto_resume_timestamp,
            subtitle: PlaySubtitleRequest {
                enabled: self.subtitles_enabled,
                info: self.subtitle_info,
                subtitle: self.subtitle,
            },
        };

        PlayMediaRequest {
            base,
            parent_media: self.parent_media,
            media: self.media.expect("media has not been set"),
            quality: self.quality.unwrap_or_else(|| "".to_string()),
            torrent_stream: self
                .torrent_stream
                .expect("torrent_stream has not been set"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::loader::SubtitleData;
    use std::sync::Arc;

    use crate::core::media::{Episode, Images, MovieOverview, ShowOverview};
    use crate::core::playlists::{PlaylistItem, PlaylistMedia, PlaylistSubtitle};
    use crate::testing::MockTorrentStream;

    use super::*;

    #[test]
    fn test_play_url_request_builder() {
        let url = "https://localhost:8054/my-video.mp4";
        let title = "DolorEsta";
        let caption = "lorem ipsum dolor esta";
        let thumb = "https://imgur.com/something.jpg";
        let background = "https://imgur.com/background.jpg";
        let auto_resume = 84000u64;
        let expected_result = PlayUrlRequest {
            url: url.to_string(),
            title: title.to_string(),
            caption: Some(caption.to_string()),
            thumb: Some(thumb.to_string()),
            background: Some(background.to_string()),
            auto_resume_timestamp: Some(auto_resume),
            subtitle: PlaySubtitleRequest {
                enabled: true,
                info: None,
                subtitle: None,
            },
        };

        let result = PlayUrlRequestBuilder::builder()
            .url(url)
            .title(title)
            .caption(caption)
            .thumb(thumb)
            .background(background)
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
        let data = LoadingData {
            url: Some(url.to_string()),
            title: Some(title.to_string()),
            caption: None,
            thumb: None,
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: Some(auto_resume.clone()),
            subtitle: SubtitleData::default(),
            media_torrent_info: None,
            torrent: None,
            torrent_stream: None,
        };
        let expected_result = PlayUrlRequest {
            url: url.to_string(),
            title: title.to_string(),
            caption: None,
            thumb: None,
            background: None,
            auto_resume_timestamp: Some(auto_resume),
            subtitle: PlaySubtitleRequest {
                enabled: false,
                info: None,
                subtitle: None,
            },
        };

        let result = PlayUrlRequest::from(data);

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
        let stream = Arc::new(Box::new(MockTorrentStream::new()) as Box<dyn TorrentStream>);
        let expected_result = PlayMediaRequest {
            base: PlayUrlRequest {
                url: url.to_string(),
                title: title.to_string(),
                caption: None,
                thumb: Some(thumb.to_string()),
                background: None,
                auto_resume_timestamp: None,
                subtitle: PlaySubtitleRequest {
                    enabled: false,
                    info: None,
                    subtitle: None,
                },
            },
            parent_media: Some(Box::new(show.clone())),
            media: Box::new(episode.clone()),
            quality: quality.to_string(),
            torrent_stream: Arc::downgrade(&stream),
        };

        let result = PlayMediaRequestBuilder::builder()
            .url(url)
            .title(title)
            .thumb(thumb)
            .quality(quality)
            .parent_media(Box::new(show))
            .media(Box::new(episode))
            .torrent_stream(Arc::downgrade(&stream))
            .build();

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_player_media_request_from_movie() {
        let url = "https://exmaple.com";
        let title = "FooBar";
        let subtitles_enabled = true;
        let quality = "1080p";
        let background = "MyBackgroundUri";
        let media = MovieOverview {
            imdb_id: "tt123456".to_string(),
            title: "MyTitle".to_string(),
            year: "2016".to_string(),
            images: Images::builder()
                .poster("MyPoster.jpg")
                .banner("MyBanner.jpg")
                .fanart(background)
                .build(),
            rating: None,
        };
        let item = PlaylistItem {
            url: Some(url.to_string()),
            title: title.to_string(),
            caption: None,
            thumb: None,
            media: PlaylistMedia {
                parent: None,
                media: Some(Box::new(media.clone())),
            },
            quality: Some(quality.to_string()),
            auto_resume_timestamp: None,
            subtitle: PlaylistSubtitle {
                enabled: subtitles_enabled,
                info: None,
            },
            torrent: Default::default(),
        };
        let stream = Arc::new(Box::new(MockTorrentStream::new()) as Box<dyn TorrentStream>);
        let mut data = LoadingData::from(item);
        data.torrent_stream = Some(Arc::downgrade(&stream));
        let expected_result = PlayMediaRequest {
            base: PlayUrlRequest {
                url: url.to_string(),
                title: title.to_string(),
                caption: None,
                thumb: None,
                background: Some(background.to_string()),
                auto_resume_timestamp: None,
                subtitle: PlaySubtitleRequest {
                    enabled: subtitles_enabled,
                    info: None,
                    subtitle: None,
                },
            },
            parent_media: None,
            media: Box::new(media),
            quality: quality.to_string(),
            torrent_stream: Arc::downgrade(&stream),
        };

        let result = PlayMediaRequest::from(data);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_player_media_request_from_episode() {
        let url = "https://localhost:87445/my-episode.mkv";
        let title = "FooBar";
        let subtitles_enabled = true;
        let quality = "720p";
        let background = "MyShowBackground.png";
        let media = ShowOverview {
            imdb_id: "tt123456".to_string(),
            tvdb_id: "tt200020".to_string(),
            title: "MyTitle".to_string(),
            year: "2016".to_string(),
            num_seasons: 5,
            images: Images::builder().fanart(background).build(),
            rating: None,
        };
        let episode = Episode {
            season: 1,
            episode: 5,
            first_aired: 2013,
            title: "MyEpisodeTitle".to_string(),
            overview: "lorem ipsum dolor".to_string(),
            tvdb_id: 1202220,
            tvdb_id_value: "tt1202220".to_string(),
            thumb: Some("MyEpisodeThumb.jpg".to_string()),
            torrents: Default::default(),
        };
        let item = PlaylistItem {
            url: Some(url.to_string()),
            title: title.to_string(),
            caption: None,
            thumb: None,
            media: PlaylistMedia {
                parent: Some(Box::new(media.clone())),
                media: Some(Box::new(episode.clone())),
            },
            quality: Some(quality.to_string()),
            auto_resume_timestamp: None,
            subtitle: PlaylistSubtitle {
                enabled: subtitles_enabled,
                info: None,
            },
            torrent: Default::default(),
        };
        let stream = Arc::new(Box::new(MockTorrentStream::new()) as Box<dyn TorrentStream>);
        let mut data = LoadingData::from(item);
        data.torrent_stream = Some(Arc::downgrade(&stream));
        let expected_result = PlayMediaRequest {
            base: PlayUrlRequest {
                url: url.to_string(),
                title: title.to_string(),
                caption: None,
                thumb: None,
                background: Some(background.to_string()),
                auto_resume_timestamp: None,
                subtitle: PlaySubtitleRequest {
                    enabled: subtitles_enabled,
                    info: None,
                    subtitle: None,
                },
            },
            parent_media: Some(Box::new(media)),
            media: Box::new(episode),
            quality: quality.to_string(),
            torrent_stream: Arc::downgrade(&stream),
        };

        let result = PlayMediaRequest::from(data);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_from_play_url_request() {
        let url = "http://localhost:8080/movie.mp4";
        let title = "FooBar";
        let thumb = "http://localhost:8080/thumbnail.jpg";
        let data = LoadingData {
            url: Some(url.to_string()),
            title: Some(title.to_string()),
            caption: None,
            thumb: Some(thumb.to_string()),
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitle: SubtitleData {
                enabled: Some(true),
                info: None,
                subtitle: None,
            },
            media_torrent_info: None,
            torrent: None,
            torrent_stream: None,
        };
        let expected = PlayUrlRequest {
            url: url.to_string(),
            title: title.to_string(),
            caption: None,
            thumb: Some(thumb.to_string()),
            background: None,
            auto_resume_timestamp: None,
            subtitle: PlaySubtitleRequest {
                enabled: true,
                info: None,
                subtitle: None,
            },
        };

        let result = PlayUrlRequest::from(data);

        assert_eq!(expected, result);
    }
}
