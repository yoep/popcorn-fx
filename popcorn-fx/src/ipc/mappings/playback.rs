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
