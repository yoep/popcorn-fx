use crate::ipc::proto;
use crate::ipc::proto::player::player::play_request;
use crate::ipc::proto::subtitle::subtitle;
use crate::ipc::proto::{player, stream};
use popcorn_fx_core::core::players::{
    PlayRequest, PlaySubtitleRequest, Player, PlayerManagerEvent, PlayerState,
};
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

impl From<&PlayRequest> for player::player::PlayRequest {
    fn from(value: &PlayRequest) -> Self {
        Self {
            url: value.url().to_string(),
            title: value.title().to_string(),
            caption: value.caption().clone(),
            thumbnail: value.thumbnail().clone(),
            background: value.background().clone(),
            quality: value.quality().clone(),
            auto_resume_timestamp: value.auto_resume_timestamp().clone(),
            subtitle: MessageField::some(play_request::PlaySubtitleRequest::from(value.subtitle())),
            stream: MessageField::from_option(
                value
                    .stream()
                    .map(|e| e.clone())
                    .map(|e| Into::<stream::ServerStream>::into(e)),
            ),
            special_fields: Default::default(),
        }
    }
}

impl From<&PlaySubtitleRequest> for play_request::PlaySubtitleRequest {
    fn from(value: &PlaySubtitleRequest) -> Self {
        Self {
            enabled: value.enabled,
            info: value.info.as_ref().map(subtitle::Info::from).into(),
            subtitle: value
                .subtitle
                .as_ref()
                .map(proto::subtitle::Subtitle::from)
                .into(),
            special_fields: Default::default(),
        }
    }
}

impl From<&PlayerManagerEvent> for player::PlayerManagerEvent {
    fn from(value: &PlayerManagerEvent) -> Self {
        let mut event = Self::new();

        match value {
            PlayerManagerEvent::ActivePlayerChanged(change) => {
                event.event = player::player_manager_event::Event::ACTIVE_PLAYER_CHANGED.into();
                event.active_player_changed =
                    MessageField::some(player::player_manager_event::ActivePlayerChanged {
                        old_player_id: change.old_player_id.clone(),
                        new_player_id: change.new_player_id.clone(),
                        new_player_name: change.new_player_name.clone(),
                        special_fields: Default::default(),
                    });
            }
            PlayerManagerEvent::PlayersChanged => {
                event.event = player::player_manager_event::Event::PLAYERS_CHANGED.into();
            }
            PlayerManagerEvent::PlayerPlaybackChanged(request) => {
                event.event = player::player_manager_event::Event::PLAYER_PLAYBACK_CHANGED.into();
                event.player_playback_changed =
                    MessageField::some(player::player_manager_event::PlayerPlaybackChanged {
                        request: MessageField::some(proto::player::player::PlayRequest::from(
                            request,
                        )),
                        special_fields: Default::default(),
                    });
            }
            PlayerManagerEvent::PlayerDurationChanged(duration) => {
                event.event = player::player_manager_event::Event::PLAYER_DURATION_CHANGED.into();
                event.player_duration_changed =
                    MessageField::some(player::player_manager_event::PlayerDurationChanged {
                        duration: *duration,
                        special_fields: Default::default(),
                    });
            }
            PlayerManagerEvent::PlayerTimeChanged(time) => {
                event.event = player::player_manager_event::Event::PLAYER_TIMED_CHANGED.into();
                event.player_time_changed =
                    MessageField::some(player::player_manager_event::PlayerTimeChanged {
                        time: *time,
                        special_fields: Default::default(),
                    });
            }
            PlayerManagerEvent::PlayerStateChanged(state) => {
                event.event = player::player_manager_event::Event::PLAYER_STATE_CHANGED.into();
                event.player_state_changed =
                    MessageField::some(player::player_manager_event::PlayerStateChanged {
                        state: proto::player::player::State::from(state).into(),
                        special_fields: Default::default(),
                    });
            }
        }

        event
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_state_proto_from() {
        assert_eq!(
            player::player::State::UNKNOWN,
            player::player::State::from(&PlayerState::Unknown)
        );
        assert_eq!(
            player::player::State::READY,
            player::player::State::from(&PlayerState::Ready)
        );
        assert_eq!(
            player::player::State::LOADING,
            player::player::State::from(&PlayerState::Loading)
        );
        assert_eq!(
            player::player::State::BUFFERING,
            player::player::State::from(&PlayerState::Buffering)
        );
        assert_eq!(
            player::player::State::PLAYING,
            player::player::State::from(&PlayerState::Playing)
        );
        assert_eq!(
            player::player::State::PAUSED,
            player::player::State::from(&PlayerState::Paused)
        );
        assert_eq!(
            player::player::State::STOPPED,
            player::player::State::from(&PlayerState::Stopped)
        );
        assert_eq!(
            player::player::State::ERROR,
            player::player::State::from(&PlayerState::Error)
        );
    }

    #[test]
    fn test_player_state_from() {
        assert_eq!(
            PlayerState::Unknown,
            PlayerState::from(&player::player::State::UNKNOWN)
        );
        assert_eq!(
            PlayerState::Ready,
            PlayerState::from(&player::player::State::READY)
        );
        assert_eq!(
            PlayerState::Loading,
            PlayerState::from(&player::player::State::LOADING)
        );
        assert_eq!(
            PlayerState::Buffering,
            PlayerState::from(&player::player::State::BUFFERING)
        );
        assert_eq!(
            PlayerState::Playing,
            PlayerState::from(&player::player::State::PLAYING)
        );
        assert_eq!(
            PlayerState::Paused,
            PlayerState::from(&player::player::State::PAUSED)
        );
        assert_eq!(
            PlayerState::Stopped,
            PlayerState::from(&player::player::State::STOPPED)
        );
        assert_eq!(
            PlayerState::Error,
            PlayerState::from(&player::player::State::ERROR)
        );
    }
}
