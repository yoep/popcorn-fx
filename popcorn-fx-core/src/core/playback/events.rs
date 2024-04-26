use derive_more::Display;

use crate::core::CoreCallback;

/// A callback for playback control events, used to handle events coming from the media system of the OS.
pub type PlaybackControlCallback = CoreCallback<PlaybackControlEvent>;

/// Events related to playback control, triggered by the media system of the OS.
/// These events can be used to modify the player state based on the given media event.
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

/// Events related to media playback notifications.
#[derive(Debug, Clone, PartialEq)]
pub enum MediaNotificationEvent {
    /// Invoked when a new playback is being started
    StateStarting(MediaInfo),
    /// Invoked when the playback state is changed to paused
    StatePaused,
    /// Invoked when the playback state is changed to playing/resuming
    StatePlaying,
    /// Invoked when the playback state is changed to stopped
    /// This state cannot be resumed anymore and requires a new [MediaNotificationEvent::StateStarting]
    StateStopped,
}

/// Information about the media being played.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaInfo {
    /// The title of the media.
    pub title: String,
    /// The name of the show.
    pub subtitle: Option<String>,
    /// The thumbnail of the currently playing media item
    pub thumb: Option<String>,
}
