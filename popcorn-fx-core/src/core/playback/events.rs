use derive_more::Display;

use crate::core::CoreCallback;

/// The callback for playback control events.
pub type PlaybackControlCallback = CoreCallback<PlaybackControlEvent>;

/// The events of the playback controller.
#[repr(i32)]
#[derive(Debug, Clone, Display)]
pub enum PlaybackControlEvent {
    #[display(fmt = "Toggle the playback state")]
    TogglePlaybackState = 0,
    #[display(fmt = "Forward media")]
    Forward = 1,
    #[display(fmt = "Rewind media")]
    Rewind = 2,
}

/// The media notification playback that media is being played.
#[derive(Debug, Clone, PartialEq)]
pub enum MediaNotificationEvent {
    /// Invoked when a new playback has started
    PlaybackStarted(MediaInfo),
    StatePaused,
    StatePlaying,
    StateStopped,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MediaInfo {
    /// The media title that is currently being played
    pub title: String,
    /// The name of the show
    pub show_name: Option<String>,
    /// The thumbnail of the currently playing media item
    pub thumb: Option<String>,
}