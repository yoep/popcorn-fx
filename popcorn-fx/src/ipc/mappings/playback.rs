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
