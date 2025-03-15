use crate::core::players::PlayRequest;
use async_trait::async_trait;
use derive_more::Display;
use downcast_rs::{impl_downcast, DowncastSync};
use fx_callback::Callback;
use std::fmt::{Debug, Display};
use std::sync::Weak;

/// A trait representing a Popcorn FX supported media player for media playback.
#[async_trait]
pub trait Player: Debug + Display + DowncastSync + Callback<PlayerEvent> {
    /// Get the unique identifier of the player.
    ///
    /// # Returns
    ///
    /// The unique identifier of the player as a string.
    fn id(&self) -> &str;

    /// Get the display friendly name of the player.
    ///
    /// # Returns
    ///
    /// The name of the player as a string.
    fn name(&self) -> &str;

    /// Get the description of the player.
    ///
    /// # Returns
    ///
    /// The description of the player as a string.
    fn description(&self) -> &str;

    /// Get the graphic resource associated with the player.
    /// This can be used to retrieve graphical assets related to the player.
    ///
    /// # Returns
    ///
    /// The graphic resource as a vector of bytes.
    fn graphic_resource(&self) -> Vec<u8>;

    /// Get the current state of the player.
    ///
    /// # Returns
    ///
    /// The current state of the player.
    async fn state(&self) -> PlayerState;

    /// Get the current playback request, if any.
    ///
    /// # Returns
    ///
    /// An optional weak reference to the current playback request.
    async fn request(&self) -> Option<Weak<Box<dyn PlayRequest>>>;

    /// Start playback with the given request.
    ///
    /// # Arguments
    ///
    /// * `request` - The playback request to start.
    async fn play(&self, request: Box<dyn PlayRequest>);

    /// Pause the current playback of the player.
    fn pause(&self);

    /// Resume the current playback of the player.
    /// If no playback is active, this invocation won't have any effect on the player.
    fn resume(&self);

    /// Seeks to the specified time position in the media.
    ///
    /// # Arguments
    ///
    /// * `time` - The time position to seek to, in milliseconds.
    fn seek(&self, time: u64);

    /// Stop playback.
    fn stop(&self);
}
impl_downcast!(sync Player);

impl PartialEq for dyn Player {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

/// An enumeration representing the possible states of a player.
#[repr(i32)]
#[derive(Debug, Display, Copy, Clone, PartialEq)]
pub enum PlayerState {
    Unknown = -1,
    Ready = 0,
    Loading = 1,
    Buffering = 2,
    Playing = 3,
    Paused = 4,
    Stopped = 5,
    Error = 6,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::Unknown
    }
}

/// An enumeration representing events related to a player.
#[repr(i32)]
#[derive(Debug, Display, Clone, PartialEq)]
pub enum PlayerEvent {
    /// The duration of the media content has changed.
    #[display(fmt = "Player duration changed to {}", _0)]
    DurationChanged(u64),
    /// The playback time position has changed.
    #[display(fmt = "Player time changed to {}", _0)]
    TimeChanged(u64),
    /// The player's state has changed.
    #[display(fmt = "Player state changed to {}", _0)]
    StateChanged(PlayerState),
    /// The volume of the player has changed.
    #[display(fmt = "Player volume changed to {}", _0)]
    VolumeChanged(u32),
}

#[cfg(test)]
mod tests {
    use crate::testing::MockPlayer;

    use super::*;

    #[test]
    fn test_player_identifier_eq() {
        let player_id = "ID123456";
        let mut m_player1 = MockPlayer::new();
        m_player1.expect_id().return_const(player_id.to_string());
        let mut m_player2 = MockPlayer::new();
        m_player2.expect_id().return_const(player_id.to_string());
        let player = Box::new(m_player1) as Box<dyn Player>;
        let other_player = Box::new(m_player2) as Box<dyn Player>;

        assert_eq!(&player, &other_player)
    }
}
