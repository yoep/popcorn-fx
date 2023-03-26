/// The media notification playback that media is being played.
#[derive(Debug, Clone)]
pub enum MediaNotificationEvent {
    /// Invoked when a new playback has started
    PlaybackStarted(MediaInfo),
    StatePaused,
    StatePlaying,
    StateStopped
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