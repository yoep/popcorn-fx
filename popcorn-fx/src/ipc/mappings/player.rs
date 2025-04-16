use crate::ipc::proto::player;
use crate::ipc::proto::subtitle::subtitle;
use popcorn_fx_core::core::players::{PlayRequest, PlaySubtitleRequest, Player, PlayerState};
use protobuf::MessageField;

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
            PlayerState::Unknown => Self::UNKNOWN,
            PlayerState::Ready => Self::READY,
            PlayerState::Loading => Self::LOADING,
            PlayerState::Buffering => Self::BUFFERING,
            PlayerState::Playing => Self::PLAYING,
            PlayerState::Paused => Self::PAUSED,
            PlayerState::Stopped => Self::STOPPED,
            PlayerState::Error => Self::ERROR,
        }
    }
}

impl From<&player::player::State> for PlayerState {
    fn from(value: &player::player::State) -> Self {
        match value {
            player::player::State::UNKNOWN => Self::Unknown,
            player::player::State::READY => Self::Ready,
            player::player::State::LOADING => Self::Loading,
            player::player::State::BUFFERING => Self::Buffering,
            player::player::State::PLAYING => Self::Playing,
            player::player::State::PAUSED => Self::Paused,
            player::player::State::STOPPED => Self::Stopped,
            player::player::State::ERROR => Self::Error,
        }
    }
}

impl From<&Box<dyn PlayRequest>> for player::player::PlayRequest {
    fn from(value: &Box<dyn PlayRequest>) -> Self {
        Self {
            url: value.url().to_string(),
            title: value.title().to_string(),
            caption: value.caption().clone(),
            thumbnail: value.thumbnail().clone(),
            background: value.background().clone(),
            quality: value.quality().clone(),
            auto_resume_timestamp: value.auto_resume_timestamp().clone(),
            subtitle: Default::default(),
            stream_handle: Default::default(),
            special_fields: Default::default(),
        }
    }
}

impl From<&PlaySubtitleRequest> for player::player::play_request::PlaySubtitleRequest {
    fn from(value: &PlaySubtitleRequest) -> Self {
        Self {
            enabled: value.enabled,
            info: MessageField::from(value.info.as_ref().map(subtitle::Info::from)),
            subtitle: Default::default(),
            special_fields: Default::default(),
        }
    }
}
