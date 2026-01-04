use derive_more::Display;

use crate::core::event::{PlayerStartedEvent, PlayerStoppedEvent};
use crate::core::playback::PlaybackState;
use crate::core::torrents::TorrentInfo;

/// Handles all events within the Popcorn FX library.
///
/// The `Event` enum represents the events that occur during the lifecycle of the Popcorn FX application.
/// It provides a mechanism for handling player and playback media events and controls, such as stopping
/// playback, starting a new playback, and changing the playback state.
#[derive(Debug, Clone, Display, PartialEq)]
pub enum Event {
    /// Invoked when the player playback has started for a new media item
    #[display("Player has started playback of {}", "_0.title.as_str()")]
    PlayerStarted(PlayerStartedEvent),
    /// Invoked when the player playback has stopped
    #[display("Player has been stopped with last known timestamp {:?}", "_0.time()")]
    PlayerStopped(PlayerStoppedEvent),
    /// Invoked when the player/playback state is changed
    #[display("Playback state has changed to {}", "_0")]
    PlaybackStateChanged(PlaybackState),
    /// Invoked when the loading of a media item has started
    #[display("Loading of a media item has started")]
    LoadingStarted,
    /// Invoked when the loading of a media item has completed
    #[display("Loading of a media item has completed")]
    LoadingCompleted,
    /// Invoked when the torrent details have been loaded of a magnet uri
    #[display("Torrent details have been loaded of {}", _0)]
    TorrentDetailsLoaded(TorrentInfo),
    /// Invoked when the player should be closed
    #[display("Closing player")]
    ClosePlayer,
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
