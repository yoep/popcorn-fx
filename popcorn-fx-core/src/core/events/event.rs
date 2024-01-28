use derive_more::Display;

use crate::core::events::{PlayerStartedEvent, PlayerStoppedEvent};
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
/// use popcorn_fx_core::core::events::{Event, PlayerChangedEvent};
///
/// let event = Event::PlayerChanged(PlayerChangedEvent {
///     old_player_id: Some("OldPlayerId".to_string()),
///     new_player_id: "NewPlayerId".to_string(),
///     new_player_name: "NewPlayerName".to_string(),
/// });
/// ```
#[derive(Debug, Clone, Display)]
pub enum Event {
    /// Invoked when the active player is changed
    #[display(fmt = "Active player changed to {} ({})", "_0.new_player_id.as_str()", "_0.new_player_name.as_str()")]
    PlayerChanged(PlayerChangedEvent),
    /// Invoked when the player playback has started for a new media item
    #[display(fmt = "Player has started playback of {}", "_0.title.as_str()")]
    PlayerStarted(PlayerStartedEvent),
    /// Invoked when the player playback has stopped
    #[display(fmt = "Player has been stopped with last known timestamp {:?}", "_0.time()")]
    PlayerStopped(PlayerStoppedEvent),
    /// Invoked when the player/playback state is changed
    #[display(fmt = "Playback state has changed to {}", _0)]
    PlaybackStateChanged(PlaybackState),
    /// Invoked when the watched state of a media items is changed
    #[display(fmt = "Watched state of {} changed to {}", _0, _1)]
    WatchStateChanged(String, bool),
    #[display(fmt = "Loading of a media item has started")]
    LoadingStarted,
}

/// Represents an event indicating a change in the active player within a multimedia application.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerChangedEvent {
    /// The previous player's unique identifier, if any.
    pub old_player_id: Option<String>,
    /// The new active player's unique identifier.
    pub new_player_id: String,
    /// The name of the new active player.
    pub new_player_name: String,
}