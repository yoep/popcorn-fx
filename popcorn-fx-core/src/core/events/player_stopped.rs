use crate::core::media::MediaIdentifier;

/// The player stopped event which indicates a video playback has been stopped.
/// It contains the last known information of the video playback right before it was stopped.
#[derive(Debug)]
pub struct PlayerStoppedEvent {
    /// The playback url that was being played
    url: String,
    /// The media item that was being played
    media: Option<Box<dyn MediaIdentifier>>,
    /// The last known video time of the player in millis
    time: Option<u64>,
    /// The duration of the video playback in millis
    duration: Option<u64>,
}

impl PlayerStoppedEvent {
    pub fn new(url: String, media: Option<Box<dyn MediaIdentifier>>, time: Option<u64>, duration: Option<u64>) -> Self {
        Self {
            url,
            media,
            time,
            duration,
        }
    }

    /// The video playback url that was being played.
    pub fn url(&self) -> &str {
        self.url.as_str()
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
}