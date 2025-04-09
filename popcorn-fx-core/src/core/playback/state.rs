use derive_more::Display;

/// The playback state of the current media item.
///
/// This enum describes the different states of the playback known by the player.
#[derive(Debug, Clone, Display, PartialEq)]
pub enum PlaybackState {
    /// This is the initial state and indicates that the playback state is unknown or hasn't been received from the player.
    ///
    /// This state usually occurs when the player is starting up or there is no active media item.
    UNKNOWN,
    /// The media player is ready to start playback.
    READY,
    /// The media player is currently loading the media item.
    LOADING,
    /// The media player is currently buffering the media data.
    BUFFERING,
    /// The media player is currently playing the media item.
    PLAYING,
    /// The media player has paused the playback.
    PAUSED,
    /// The media player has stopped the playback.
    STOPPED,
    /// An error has occurred during playback.
    ERROR,
}
