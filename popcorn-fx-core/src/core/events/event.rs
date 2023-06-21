use derive_more::Display;

use crate::core::events::PlayerStoppedEvent;
use crate::core::playback::PlaybackState;

/// Handles all events within the Popcorn FX library.
///
/// The `Event` enum represents the events that occur during the lifecycle of the Popcorn FX application.
/// It provides a mechanism for handling player and playback media events and controls, such as stopping
/// playback, starting a new playback, and changing the playback state.
///
/// # Examples
///
/// ```no_run
/// use popcorn_fx_core::core::events::{Event, PlayVideoEvent};
///
/// let event = Event::PlayVideo(PlayVideoEvent {
///     url: "https://example.com/video.mp4".to_string(),
///     title: "Example Video".to_string(),
///     subtitle: None,
///     thumb: Some("https://example.com/thumb.jpg".to_string()),
/// });
/// ```
#[derive(Debug, Clone, Display)]
pub enum Event {
    /// Invoked when the player playback has stopped
    #[display(fmt = "Player has been stopped with last known timestamp {:?}", "_0.time()")]
    PlayerStopped(PlayerStoppedEvent),
    /// Invoked when a new playback is being started
    #[display(fmt = "Play new video with url: {}", "_0.url")]
    PlayVideo(PlayVideoEvent),
    /// Invoked when the player/playback state is changed
    #[display(fmt = "Playback state has changed to {}", _0)]
    PlaybackStateChanged(PlaybackState),
    /// Invoked when the watched state of a media items is changed
    #[display(fmt = "Watched state of {} changed to {}", _0, _1)]
    WatchStateChanged(String, bool),
}

/// The event is triggered when a new video playback is started.
///
/// The `PlayVideoEvent` struct represents the event that starts a new playback in the Popcorn FX application.
/// It provides information about the URL, title, and thumbnail of the video that is being played.
///
/// # Examples
///
/// ```no_run
/// use popcorn_fx_core::core::events::PlayVideoEvent;
///
/// let event = PlayVideoEvent {
///     url: "https://example.com/video.mp4".to_string(),
///     title: "Example Video".to_string(),
///     subtitle: None,
///     thumb: Some("https://example.com/thumb.jpg".to_string()),
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct PlayVideoEvent {
    /// The playback url to start
    pub url: String,
    /// The video title
    pub title: String,
    /// The video subtitle/additional info
    pub subtitle: Option<String>,
    /// The url to the video thumbnail
    pub thumb: Option<String>,
}