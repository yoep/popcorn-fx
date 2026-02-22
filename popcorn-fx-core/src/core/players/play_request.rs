use crate::core::loader::LoadingData;
use crate::core::media::MediaIdentifier;
use crate::core::players::RequestError;
use crate::core::stream::ServerStream;
use crate::core::subtitles::model::{Subtitle, SubtitleInfo};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

/// A struct representing a play request for media with additional metadata.
#[derive(Debug, Clone)]
pub struct PlayRequest {
    inner: Arc<InnerPlayRequest>,
}

impl PlayRequest {
    /// Create a new builder instance for creating a play request.
    pub fn builder() -> PlayRequestBuilder {
        PlayRequestBuilder::default()
    }

    /// Get the URL of the media to be played.
    pub fn url(&self) -> &str {
        self.inner.url.as_str()
    }

    /// Get the title of the media (if available).
    ///
    /// Returns a `String` containing the title.
    pub fn title(&self) -> &str {
        self.inner.title.as_str()
    }

    /// Get the optional caption of this request.
    ///
    /// Returns an optional `String` containing the caption of the request, or `None` if not available.
    pub fn caption(&self) -> Option<String> {
        self.inner.caption.clone()
    }

    /// Get the URL of the thumbnail associated with the media (if available).
    ///
    /// Returns an optional `String` containing the thumbnail URL, or `None` if not available.
    pub fn thumbnail(&self) -> Option<String> {
        self.inner.thumb.clone()
    }

    /// Get the URL of the background image associated with the media (if available).
    /// This image can be shown during the loading of the media stream.
    ///
    /// Returns an optional `String` containing the background URL, or `None` if not available.
    pub fn background(&self) -> Option<String> {
        self.inner.background.clone()
    }

    /// Get the quality information of the media (if available).
    ///
    /// Returns an optional `String` containing quality information, or `None` if not available.
    pub fn quality(&self) -> Option<String> {
        self.inner.quality.clone()
    }

    /// Get the auto-resume timestamp for the media playback (if available).
    ///
    /// Returns an optional `u64` representing the auto-resume timestamp in seconds, or `None` if not available.
    pub fn auto_resume_timestamp(&self) -> Option<u64> {
        self.inner.auto_resume_timestamp.clone()
    }

    /// Get the subtitle playback information for the media.
    ///
    /// Returns a reference to the `PlaySubtitleRequest` information.
    pub fn subtitle(&self) -> &PlaySubtitleRequest {
        &self.inner.subtitle
    }

    /// Get the media item of the play request.
    pub fn media(&self) -> Option<Box<dyn MediaIdentifier>> {
        self.inner.media.as_ref().and_then(|e| e.clone_identifier())
    }

    /// Get the parent media item of the play request.
    pub fn parent_media(&self) -> Option<Box<dyn MediaIdentifier>> {
        self.inner
            .parent_media
            .as_ref()
            .and_then(|e| e.clone_identifier())
    }

    /// Returns the stream of the play request.
    pub fn stream(&self) -> Option<&ServerStream> {
        self.inner.stream.as_ref()
    }

    /// Get the additional metadata if the play request.
    pub fn metadata(&self) -> PlayRequestMetadata {
        self.inner.metadata.clone()
    }
}

impl PartialEq for PlayRequest {
    fn eq(&self, other: &Self) -> bool {
        *self.inner == *other.inner
    }
}

impl<S> From<S> for PlayRequest
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        PlayRequest::builder()
            .url(value.into().as_str())
            .title("")
            .build()
    }
}

impl TryFrom<&mut LoadingData> for PlayRequest {
    type Error = RequestError;

    fn try_from(value: &mut LoadingData) -> Result<Self, Self::Error> {
        let url = value.url.take().ok_or(RequestError::UrlMissing)?;
        let title = value.title.take().ok_or(RequestError::TitleMissing)?;
        let subtitles_enabled = value.subtitle.enabled.unwrap_or(false);
        let media = value.media.take();
        let mut builder = Self::builder();
        builder
            .url(url)
            .title(title)
            .subtitles_enabled(subtitles_enabled);

        if let Some(e) = value.caption.take() {
            builder.caption(e);
        }
        if let Some(e) = value.thumb.take() {
            builder.thumb(e);
        }
        if let Some(e) = value.auto_resume_timestamp {
            builder.auto_resume_timestamp(e);
        }
        match value.parent_media.take() {
            Some(parent) => {
                if let Some(overview) = parent.into_overview() {
                    builder.background(overview.images().fanart());
                }
                builder.parent_media(parent);
            }
            None => {
                if let Some(media) = media.as_ref().and_then(|e| e.into_overview()) {
                    builder.background(media.images().fanart());
                }
            }
        }
        if let Some(media) = media {
            builder.media(media);
        }
        if let Some(e) = value.quality.take() {
            builder.quality(e.as_str());
        }
        if let Some(stream) = value.stream.take() {
            builder.stream(stream);
        }
        if subtitles_enabled {
            if let Some(e) = value.subtitle.subtitle.take() {
                builder.subtitle(e);
            }
        }

        Ok(builder.build())
    }
}

/// A builder for constructing a `PlayMediaRequest` with optional parameters.
#[derive(Debug, Default)]
pub struct PlayRequestBuilder {
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
    stream: Option<ServerStream>,
    metadata: Option<PlayRequestMetadata>,
}

impl PlayRequestBuilder {
    /// Creates a new instance of the builder with default values.
    pub fn builder() -> Self {
        Self::default()
    }

    /// Sets the URL for the media to be played.
    pub fn url<S>(&mut self, url: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.url = Some(url.into());
        self
    }

    /// Sets the title of the media.
    pub fn title<S>(&mut self, title: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.title = Some(title.into());
        self
    }

    /// Sets the caption of the media.
    pub fn caption<S>(&mut self, caption: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.caption = Some(caption.into());
        self
    }

    /// Sets the URL of the thumbnail associated with the media.
    pub fn thumb<S>(&mut self, thumb: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.thumb = Some(thumb.into());
        self
    }

    /// Sets the URL of the background associated with the media.
    pub fn background<S>(&mut self, background: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.background = Some(background.into());
        self
    }

    /// Sets the auto-resume timestamp for media playback.
    pub fn auto_resume_timestamp(&mut self, auto_resume_timestamp: u64) -> &mut Self {
        self.auto_resume_timestamp = Some(auto_resume_timestamp);
        self
    }

    /// Sets whether subtitles are enabled for the media playback.
    pub fn subtitles_enabled(&mut self, subtitles_enabled: bool) -> &mut Self {
        self.subtitles_enabled = subtitles_enabled;
        self
    }

    /// Sets the subtitle information for the media playback.
    pub fn subtitle_info(&mut self, subtitle_info: SubtitleInfo) -> &mut Self {
        self.subtitle_info = Some(subtitle_info);
        self
    }

    /// Sets the selected subtitle for the media playback.
    pub fn subtitle(&mut self, subtitle: Subtitle) -> &mut Self {
        self.subtitle = Some(subtitle);
        self
    }

    /// Sets the media identifier for the requested media.
    pub fn media(&mut self, media: Box<dyn MediaIdentifier>) -> &mut Self {
        self.media = Some(media);
        self
    }

    /// Sets the parent media identifier (if applicable).
    pub fn parent_media(&mut self, parent_media: Box<dyn MediaIdentifier>) -> &mut Self {
        self.parent_media = Some(parent_media);
        self
    }

    /// Sets the quality information for the media.
    pub fn quality<S>(&mut self, quality: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.quality = Some(quality.into());
        self
    }

    /// Sets the stream for the media item.
    pub fn stream(&mut self, stream: ServerStream) -> &mut Self {
        self.stream = Some(stream);
        self
    }

    /// Add a string metadata value to the play request.
    pub fn metadata_str<S: AsRef<str>>(&mut self, key: S, value: S) -> &mut Self {
        self.metadata
            .get_or_insert(PlayRequestMetadata::default())
            .insert(key, MetadataValue::from(value.as_ref()));
        self
    }

    /// Add a boolean metadata value to the play request.
    pub fn metadata_bool<S: AsRef<str>>(&mut self, key: S, value: bool) -> &mut Self {
        self.metadata
            .get_or_insert(PlayRequestMetadata::default())
            .insert(key, MetadataValue::from(value));
        self
    }

    /// Builds and returns a `PlayMediaRequest` based on the provided parameters.
    ///
    /// # Panics
    ///
    /// Panics if the required fields (`url`, `title`, and `media`) are not provided.
    pub fn build(&mut self) -> PlayRequest {
        if self.url.is_none() || self.title.is_none() {
            panic!("url and title fields must be provided to build PlayMediaRequest");
        }

        PlayRequest {
            inner: Arc::new(InnerPlayRequest {
                url: self.url.take().unwrap(),
                title: self.title.take().unwrap(),
                caption: self.caption.take(),
                thumb: self.thumb.take(),
                background: self.background.take(),
                auto_resume_timestamp: self.auto_resume_timestamp.take(),
                subtitle: PlaySubtitleRequest {
                    enabled: self.subtitles_enabled,
                    info: self.subtitle_info.take(),
                    subtitle: self.subtitle.take(),
                },
                parent_media: self.parent_media.take(),
                media: self.media.take(),
                quality: self.quality.take(),
                stream: self.stream.take(),
                metadata: self.metadata.take().unwrap_or_default(),
            }),
        }
    }
}

impl From<&PlayRequest> for PlayRequestBuilder {
    fn from(request: &PlayRequest) -> Self {
        Self {
            url: Some(request.url().to_string()),
            title: Some(request.title().to_string()),
            caption: request.caption().clone(),
            thumb: request.thumbnail().clone(),
            background: request.background().clone(),
            auto_resume_timestamp: request.auto_resume_timestamp(),
            subtitles_enabled: request.subtitle().enabled,
            subtitle_info: request.subtitle().info.clone(),
            subtitle: request.subtitle().subtitle.clone(),
            media: request.media(),
            parent_media: request.parent_media(),
            quality: request.quality(),
            stream: None, // TODO: make stream cloneable
            metadata: Some(request.metadata()),
        }
    }
}

/// The subtitle information of a play request.
#[derive(Debug, Clone, PartialEq)]
pub struct PlaySubtitleRequest {
    pub enabled: bool,
    pub info: Option<SubtitleInfo>,
    pub subtitle: Option<Subtitle>,
}

/// The additional metadata of a play request.
#[derive(Debug, Clone, Default)]
pub struct PlayRequestMetadata {
    data: HashMap<String, MetadataValue>,
}

impl PlayRequestMetadata {
    /// Get a value from the play request metadata.
    pub fn get(&self, key: &str) -> Option<&MetadataValue> {
        self.data.get(key)
    }

    /// Insert a new value into the metadata.
    pub fn insert<S: AsRef<str>>(&mut self, key: S, value: MetadataValue) {
        self.data.insert(key.as_ref().to_string(), value);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetadataValue {
    String(String),
    Bool(bool),
    Integer(i32),
    Long(i64),
}

impl From<String> for MetadataValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for MetadataValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<bool> for MetadataValue {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

#[derive(Debug)]
struct InnerPlayRequest {
    /// The URL of the media to be played.
    url: String,
    /// The title of the media.
    title: String,
    /// The caption of the media request (if available).
    caption: Option<String>,
    /// The URL of the thumbnail associated with the media (if available).
    thumb: Option<String>,
    /// The URL of the background image associated with the media (if available).
    background: Option<String>,
    /// The auto-resume timestamp for media playback (if available).
    auto_resume_timestamp: Option<u64>,
    /// The subtitle playback information for this request.
    subtitle: PlaySubtitleRequest,
    /// The parent media identifier (if applicable).
    parent_media: Option<Box<dyn MediaIdentifier>>,
    /// The media identifier for the requested media.
    media: Option<Box<dyn MediaIdentifier>>,
    /// The quality information for the media.
    quality: Option<String>,
    /// The info that is being used to stream the media item
    stream: Option<ServerStream>,
    /// The metadata of the play request.
    metadata: PlayRequestMetadata,
}

impl PartialEq for InnerPlayRequest {
    fn eq(&self, other: &Self) -> bool {
        let imdb_id = self.media.as_ref().map(|e| e.imdb_id()).unwrap_or_default();
        let other_imdb_id = other
            .media
            .as_ref()
            .map(|e| e.imdb_id())
            .unwrap_or_default();

        self.url == other.url
            && self.title == other.title
            && self.caption == other.caption
            && self.parent_media.is_some() == other.parent_media.is_some()
            && self.media.is_some() == other.media.is_some()
            && imdb_id == other_imdb_id
            && self.quality == other.quality
    }
}

#[cfg(test)]
mod tests {
    use crate::core::loader::SubtitleData;
    use crate::core::media::{Episode, Images, MovieOverview, ShowOverview};
    use crate::core::playlist::{PlaylistItem, PlaylistMedia, PlaylistSubtitle};
    use url::Url;

    use super::*;

    #[test]
    fn test_play_request_builder() {
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
        let expected_result = PlayRequest {
            inner: Arc::new(InnerPlayRequest {
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
                parent_media: Some(Box::new(show.clone())),
                media: Some(Box::new(episode.clone())),
                quality: Some(quality.to_string()),
                stream: Some(ServerStream {
                    url: Url::parse("http://localhost:8054/my-video.mp4").unwrap(),
                    filename: "".to_string(),
                }),
                metadata: Default::default(),
            }),
        };

        let result = PlayRequestBuilder::builder()
            .url(url)
            .title(title)
            .thumb(thumb)
            .quality(quality)
            .parent_media(Box::new(show))
            .media(Box::new(episode))
            .stream(ServerStream {
                url: Url::parse("http://localhost:8090/my-video.mp4").unwrap(),
                filename: "my-video.mp4".to_string(),
            })
            .build();

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_player_request_from_movie() {
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
        let mut data = LoadingData::from(item);
        data.stream = Some(ServerStream {
            url: Url::parse("http://localhost:8745/my-episode.mkv").unwrap(),
            filename: "my-episode.mkv".to_string(),
        });
        let expected_result = PlayRequest {
            inner: Arc::new(InnerPlayRequest {
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
                parent_media: None,
                media: Some(Box::new(media)),
                quality: Some(quality.to_string()),
                stream: Some(ServerStream {
                    url: Url::parse("http://localhost:8054/my-episode.mkv").unwrap(),
                    filename: "my-episode.mkv".to_string(),
                }),
                metadata: Default::default(),
            }),
        };

        let result = PlayRequest::try_from(&mut data);

        assert_eq!(Ok(expected_result), result)
    }

    #[test]
    fn test_player_request_from_episode() {
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
        let mut data = LoadingData::from(item);
        data.stream = Some(ServerStream {
            url: Url::parse("http://localhost:8745/my-episode.mkv").unwrap(),
            filename: "my-episode.mkv".to_string(),
        });
        let expected_result = PlayRequest {
            inner: Arc::new(InnerPlayRequest {
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
                parent_media: Some(Box::new(media)),
                media: Some(Box::new(episode)),
                quality: Some(quality.to_string()),
                stream: Some(ServerStream {
                    url: Url::parse("http://localhost:8400/my-episode.mkv").unwrap(),
                    filename: "my-episode.mkv".to_string(),
                }),
                metadata: Default::default(),
            }),
        };

        let result = PlayRequest::try_from(&mut data);

        assert_eq!(Ok(expected_result), result)
    }

    #[test]
    fn test_from_play_request_builder() {
        let request = PlayRequest {
            inner: Arc::new(InnerPlayRequest {
                url: "http://localhost/my-video.mp4".to_string(),
                title: "Bar".to_string(),
                caption: Some("Lorem ipsum dolor".to_string()),
                thumb: Some("SomeThumb".to_string()),
                background: Some("SomeBackground".to_string()),
                auto_resume_timestamp: Some(12365999),
                subtitle: PlaySubtitleRequest {
                    enabled: true,
                    info: None,
                    subtitle: None,
                },
                parent_media: None,
                media: None,
                quality: Some("1080p".to_string()),
                stream: None,
                metadata: Default::default(),
            }),
        };

        let result = PlayRequestBuilder::from(&request).build();

        assert_eq!(request, result);
    }

    #[test]
    fn test_try_from_url_loading_data_play_request() {
        let url = "http://localhost:8080/movie.mp4";
        let title = "FooBar";
        let thumb = "http://localhost:8080/thumbnail.jpg";
        let mut data = LoadingData {
            url: Some(url.to_string()),
            title: Some(title.to_string()),
            caption: None,
            thumb: Some(thumb.to_string()),
            parent_media: None,
            media: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitle: SubtitleData {
                enabled: Some(true),
                info: None,
                subtitle: None,
            },
            torrent: None,
            filename: None,
            stream: None,
        };
        let expected = PlayRequest {
            inner: Arc::new(InnerPlayRequest {
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
                parent_media: None,
                media: None,
                quality: None,
                stream: None,
                metadata: Default::default(),
            }),
        };

        let result = PlayRequest::try_from(&mut data);

        assert_eq!(Ok(expected), result);
    }
}
