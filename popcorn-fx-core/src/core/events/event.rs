use derive_more::Display;

use crate::core::events::PlayerStoppedEvent;

/// The events of Popcorn FX application which occur during the lifecycle of the application.
#[derive(Debug, Clone, Display)]
pub enum Event {
    /// The playback of a video item has been stopped
    #[display(fmt = "Player has been stopped with last known timestamp {:?}", "_0.time()")]
    PlayerStopped(PlayerStoppedEvent),
    #[display(fmt = "Play new video with url: {}", "_0.url")]
    PlayVideo(PlayVideoEvent),
}

/// The play video event which starts a new playback.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayVideoEvent {
    /// The playback url to start
    pub url: String,
    /// The video title
    pub title: String,
    /// The url to the video thumbnail
    pub thumb: Option<String>,
}