use derive_more::Display;

/// The playback state of the current media item.
///
/// This enum describes the different states of the playback known by the player.
#[repr(i32)]
#[derive(Debug, Clone, Display, PartialEq)]
pub enum PlaybackState {
    /// This is the initial state and indicates that the playback state is unknown or hasn't been received from the player.
    ///
    /// This state usually occurs when the player is starting up or there is no active media item.
    UNKNOWN = -1,
    /// The media player is ready to start playback.
    READY = 0,
    /// The media player is currently loading the media item.
    LOADING = 1,
    /// The media player is currently buffering the media data.
    BUFFERING = 2,
    /// The media player is currently playing the media item.
    PLAYING = 3,
    /// The media player has paused the playback.
    PAUSED = 4,
    /// The media player has stopped the playback.
    STOPPED = 5,
    /// An error has occurred during playback.
    ERROR = 6,
}
