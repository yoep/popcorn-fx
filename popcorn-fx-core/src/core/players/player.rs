use std::fmt::{Debug, Display};
#[cfg(test)]
use std::fmt::Formatter;

#[cfg(test)]
use mockall::automock;

/// A trait representing a player for media playback.
///
/// This trait extends `PlayerIdentifier` and includes additional methods related to the player's
/// description, graphic resource, and current state.
#[cfg_attr(test, automock)]
pub trait Player: Debug + Display {
    /// Get the unique identifier of the player.
    fn id(&self) -> &str;

    /// Get the name of the player.
    fn name(&self) -> &str;

    /// Get the description of the player.
    fn description(&self) -> &str;

    /// Get the graphic resource associated with the player.
    ///
    /// This can be used to retrieve graphical assets related to the player.
    fn graphic_resource(&self) -> Vec<u8>;

    /// Get the current state of the player.
    fn state(&self) -> &PlayerState;
}

impl PartialEq for dyn Player {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

/// An enumeration representing the possible states of a player.
#[repr(i32)]
#[derive(Debug, Clone)]
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

#[cfg(test)]
impl Display for MockPlayer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockPlayer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_identifier_eq() {
        let player_id = "ID123456";
        let mut m_player1 = MockPlayer::new();
        m_player1.expect_id()
            .return_const(player_id.to_string());
        let mut m_player2 = MockPlayer::new();
        m_player2.expect_id()
            .return_const(player_id.to_string());
        let player = Box::new(m_player1) as Box<dyn Player>;
        let other_player = Box::new(m_player2) as Box<dyn Player>;

        assert_eq!(&player, &other_player)
    }
}