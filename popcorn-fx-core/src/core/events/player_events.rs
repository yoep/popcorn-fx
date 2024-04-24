use itertools::Itertools;
use log::{error, trace, warn};
use url::Url;

use crate::core::media::MediaIdentifier;
use crate::core::players::PlayRequest;

/// Represents an event indicating that a multimedia player has started playback with specific media details.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PlayerStartedEvent {
    /// The URL of the media resource that was started.
    pub url: String,
    /// The title of the media.
    pub title: String,
    /// An optional URL for the media thumbnail or cover art.
    pub thumbnail: Option<String>,
    /// An optional URL for the media background or cover art.
    pub background: Option<String>,
    /// An optional string indicating the quality of the media (e.g., "HD" or "4K").
    pub quality: Option<String>,
    /// An optional timestamp indicating where to auto-resume playback, if supported.
    pub auto_resume_timestamp: Option<u64>,
    /// A flag indicating whether subtitles are enabled for the media.
    pub subtitles_enabled: bool,
}

impl From<&Box<dyn PlayRequest>> for PlayerStartedEvent {
    fn from(value: &Box<dyn PlayRequest>) -> Self {
        Self {
            url: value.url().to_string(),
            title: value.title().to_string(),
            thumbnail: value.thumbnail().clone(),
            background: value.background().clone(),
            quality: value.quality().clone(),
            auto_resume_timestamp: value.auto_resume_timestamp().clone(),
            subtitles_enabled: value.subtitles_enabled(),
        }
    }
}

/// The player stopped event which indicates a video playback has been stopped.
/// It contains the last known information of the video playback right before it was stopped.
#[derive(Debug)]
pub struct PlayerStoppedEvent {
    /// The playback url that was being played
    pub url: String,
    /// The media item that was being played
    pub media: Option<Box<dyn MediaIdentifier>>,
    /// The last known video time of the player in millis
    pub time: Option<u64>,
    /// The duration of the video playback in millis
    pub duration: Option<u64>,
}

impl PlayerStoppedEvent {
    /// The video playback url that was being played.
    pub fn url(&self) -> &str {
        self.url.as_str()
    }

    pub fn uri(&self) -> Option<Url> {
        match Url::parse(self.url.as_str()) {
            Ok(e) => Some(e),
            Err(e) => {
                error!("Playback url is invalid, {}", e);
                None
            }
        }
    }

    pub fn filename(&self) -> Option<String> {
        match self.uri() {
            Some(uri) => {
                trace!("Extracting path from uri {:?}", uri);
                if let Some(path) = uri.path_segments() {
                    trace!("Extracting filename from path {:?}", path);
                    if let Some(filename) = path.last() {
                        trace!("Extracted filename {} from {}", filename, uri);
                        return Some(Self::url_decode(filename));
                    }
                } else {
                    warn!(
                        "Unable to retrieve filename, uri has no path for {:?}",
                        self
                    );
                }

                None
            }
            None => {
                warn!(
                    "Unable to retrieve filename, no valid uri found for {:?}",
                    self
                );
                None
            }
        }
    }

    /// The media item that was being played.
    pub fn media(&self) -> Option<&Box<dyn MediaIdentifier>> {
        self.media.as_ref()
    }

    /// The last known time of the video playback.
    ///
    /// It returns [None] when the playback didn't start and there is no
    /// known timestamp for the video.
    pub fn time(&self) -> Option<&u64> {
        self.time.as_ref()
    }

    /// The known duration of the video playback.
    ///
    /// It returns [None] when the playback didn't start or the duration of the
    /// video couldn't be determined.
    pub fn duration(&self) -> Option<&u64> {
        self.duration.as_ref()
    }

    fn url_decode(filename: &str) -> String {
        url::form_urlencoded::parse(filename.as_bytes())
            .map(|(key, value)| key.to_string() + value.as_ref())
            .join("")
    }
}

impl Clone for PlayerStoppedEvent {
    fn clone(&self) -> Self {
        let cloned_media = match &self.media {
            None => None,
            Some(media) => media.clone_identifier(),
        };

        PlayerStoppedEvent {
            url: self.url.clone(),
            media: cloned_media,
            time: self.time,
            duration: self.duration,
        }
    }
}

impl PartialEq for PlayerStoppedEvent {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url && self.time == other.time && self.duration == other.duration
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::core::media::{Episode, Images, Rating, ShowOverview};
    use crate::core::players::PlayUrlRequestBuilder;
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn player_started_event_from() {
        let url = "https://dummy";
        let title = "MyTitle";
        let thumb = "MyThumb";
        let auto_resume = 50000;
        let background = "MyBackground.jpg";
        let request = PlayUrlRequestBuilder::builder()
            .url(url)
            .title(title)
            .thumb(thumb)
            .background(background)
            .auto_resume_timestamp(auto_resume)
            .subtitles_enabled(true)
            .build();
        let expected_result = PlayerStartedEvent {
            url: url.to_string(),
            title: title.to_string(),
            thumbnail: Some(thumb.to_string()),
            background: Some(background.to_string()),
            quality: None,
            auto_resume_timestamp: Some(auto_resume),
            subtitles_enabled: true,
        };

        let result = PlayerStartedEvent::from(&(Box::new(request) as Box<dyn PlayRequest>));

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_player_stopped_event_clone_with_episode() {
        init_logger();
        let media = Episode {
            season: 1,
            episode: 1,
            first_aired: 1234567890,
            title: String::from("Episode 1"),
            overview: String::from("The first episode"),
            tvdb_id: 123,
            tvdb_id_value: String::from("123"),
            thumb: Some(String::from("https://example.com/thumb.jpg")),
            torrents: HashMap::new(),
        };
        let boxed_media = Box::new(media.clone());
        let event = PlayerStoppedEvent {
            url: String::from("https://example.com/video.mp4"),
            media: Some(boxed_media),
            time: Some(100),
            duration: Some(500),
        };

        let cloned_event = event.clone();

        let cloned_media = cloned_event.media.unwrap();
        let cloned_episode = cloned_media.as_ref().downcast_ref::<Episode>().unwrap();

        assert_eq!(cloned_episode, &media);
    }

    #[test]
    fn test_player_stopped_event_clone_with_show_overview() {
        let media_with_rating = ShowOverview {
            imdb_id: String::from("tt1234567"),
            tvdb_id: String::from("12345"),
            title: String::from("The Test Show"),
            year: String::from("2021"),
            num_seasons: 3,
            images: Images {
                poster: String::from("https://example.com/poster.jpg"),
                fanart: String::from("https://example.com/fanart.jpg"),
                banner: String::from("https://example.com/banner.jpg"),
            },
            rating: Some(Rating {
                percentage: 85,
                watching: 100,
                votes: 200,
                loved: 150,
                hated: 50,
            }),
        };
        let boxed_media_with_rating =
            Box::new(media_with_rating.clone()) as Box<dyn MediaIdentifier>;

        let event_with_rating = PlayerStoppedEvent {
            url: String::from("https://example.com/video.mp4"),
            media: Some(boxed_media_with_rating),
            time: Some(100),
            duration: Some(500),
        };

        let cloned_event_with_rating = event_with_rating.clone();

        let cloned_media_with_rating = cloned_event_with_rating.media.unwrap();
        let cloned_show_overview_with_rating = cloned_media_with_rating
            .as_ref()
            .downcast_ref::<ShowOverview>()
            .unwrap();

        assert_eq!(cloned_show_overview_with_rating, &media_with_rating);
    }

    #[test]
    fn test_player_stopped_event_equality() {
        let event1 = PlayerStoppedEvent {
            url: String::from("http://example.com/video.mp4"),
            media: None,
            time: Some(5000),
            duration: Some(10000),
        };

        let event2 = PlayerStoppedEvent {
            url: String::from("http://example.com/video.mp4"),
            media: None,
            time: Some(5000),
            duration: Some(10000),
        };

        assert_eq!(event1, event2);
    }

    #[test]
    fn test_player_stopped_event_inequality() {
        let event1 = PlayerStoppedEvent {
            url: String::from("http://example.com/video.mp4"),
            media: None,
            time: Some(5000),
            duration: Some(10000),
        };

        let event2 = PlayerStoppedEvent {
            url: String::from("http://example.com/video.mp4"),
            media: None,
            time: Some(8000),
            duration: Some(30000),
        };

        assert_ne!(event1, event2);
    }
}
