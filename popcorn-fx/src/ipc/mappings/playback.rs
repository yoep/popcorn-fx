use crate::ipc::proto::player::player;
use popcorn_fx_core::core::playback::PlaybackState;

impl From<&PlaybackState> for player::State {
    fn from(value: &PlaybackState) -> Self {
        match value {
            PlaybackState::UNKNOWN => player::State::UNKNOWN,
            PlaybackState::READY => player::State::READY,
            PlaybackState::LOADING => player::State::LOADING,
            PlaybackState::BUFFERING => player::State::BUFFERING,
            PlaybackState::PLAYING => player::State::PLAYING,
            PlaybackState::PAUSED => player::State::PAUSED,
            PlaybackState::STOPPED => player::State::STOPPED,
            PlaybackState::ERROR => player::State::ERROR,
        }
    }
}

impl From<&player::State> for PlaybackState {
    fn from(value: &player::State) -> Self {
        match value {
            player::State::READY => PlaybackState::READY,
            player::State::LOADING => PlaybackState::LOADING,
            player::State::BUFFERING => PlaybackState::BUFFERING,
            player::State::PLAYING => PlaybackState::PLAYING,
            player::State::PAUSED => PlaybackState::PAUSED,
            player::State::STOPPED => PlaybackState::STOPPED,
            player::State::ERROR => PlaybackState::ERROR,
            player::State::UNKNOWN => PlaybackState::UNKNOWN,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playback_state_from() {
        assert_eq!(
            PlaybackState::UNKNOWN,
            PlaybackState::from(&player::State::UNKNOWN)
        );
        assert_eq!(
            PlaybackState::READY,
            PlaybackState::from(&player::State::READY)
        );
        assert_eq!(
            PlaybackState::LOADING,
            PlaybackState::from(&player::State::LOADING)
        );
        assert_eq!(
            PlaybackState::BUFFERING,
            PlaybackState::from(&player::State::BUFFERING)
        );
        assert_eq!(
            PlaybackState::PLAYING,
            PlaybackState::from(&player::State::PLAYING)
        );
        assert_eq!(
            PlaybackState::PAUSED,
            PlaybackState::from(&player::State::PAUSED)
        );
        assert_eq!(
            PlaybackState::STOPPED,
            PlaybackState::from(&player::State::STOPPED)
        );
        assert_eq!(
            PlaybackState::ERROR,
            PlaybackState::from(&player::State::ERROR)
        );
    }

    #[test]
    fn test_player_state_proto_from() {
        assert_eq!(
            player::State::UNKNOWN,
            player::State::from(&PlaybackState::UNKNOWN)
        );
        assert_eq!(
            player::State::READY,
            player::State::from(&PlaybackState::READY)
        );
        assert_eq!(
            player::State::LOADING,
            player::State::from(&PlaybackState::LOADING)
        );
        assert_eq!(
            player::State::BUFFERING,
            player::State::from(&PlaybackState::BUFFERING)
        );
        assert_eq!(
            player::State::PLAYING,
            player::State::from(&PlaybackState::PLAYING)
        );
        assert_eq!(
            player::State::PAUSED,
            player::State::from(&PlaybackState::PAUSED)
        );
        assert_eq!(
            player::State::STOPPED,
            player::State::from(&PlaybackState::STOPPED)
        );
        assert_eq!(
            player::State::ERROR,
            player::State::from(&PlaybackState::ERROR)
        );
    }
}
