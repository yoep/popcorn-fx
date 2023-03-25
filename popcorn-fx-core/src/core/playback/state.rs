use derive_more::Display;

/// The playback state of the current media item.
/// This describes the information of the playback state known by the player.
#[repr(i32)]
#[derive(Debug, Clone, Display)]
pub enum PlaybackState {
    /// This is the initial state and indicates that FX has no known state received from the player.
    UNKNOWN = -1,
    READY = 0,
    LOADING = 1,
    BUFFERING = 2,
    PLAYING = 3,
    PAUSED = 4,
    STOPPED = 5,
    ERROR = 6,
}