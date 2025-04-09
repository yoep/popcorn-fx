use crate::ipc::proto::player;
use popcorn_fx_core::core::players::{Player, PlayerState};

impl From<&Box<dyn Player>> for player::Player {
    fn from(value: &Box<dyn Player>) -> Self {
        Self {
            id: value.id().to_string(),
            name: value.name().to_string(),
            description: value.description().to_string(),
            graphic_resource: value.graphic_resource(),
            state: Default::default(),
            special_fields: Default::default(),
        }
    }
}

impl From<&PlayerState> for player::player::State {
    fn from(value: &PlayerState) -> Self {
        match value {
            PlayerState::Unknown => player::player::State::UNKNOWN,
            PlayerState::Ready => player::player::State::READY,
            PlayerState::Loading => player::player::State::LOADING,
            PlayerState::Buffering => player::player::State::BUFFERING,
            PlayerState::Playing => player::player::State::PLAYING,
            PlayerState::Paused => player::player::State::PAUSED,
            PlayerState::Stopped => player::player::State::STOPPED,
            PlayerState::Error => player::player::State::ERROR,
        }
    }
}
