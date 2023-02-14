use crate::core::events::PlayerStoppedEvent;

/// The events of Popcorn FX which occur during the lifecycle of the application.
#[derive(Debug)]
pub enum Event {
    /// The playback of a video item has been stopped
    PlayerStopped(PlayerStoppedEvent)
}