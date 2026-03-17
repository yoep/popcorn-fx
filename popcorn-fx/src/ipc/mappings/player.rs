use crate::ipc::proto::player::player::play_request;
use crate::ipc::proto::player::player_event::Event;
use crate::ipc::proto::subtitle::subtitle;
use crate::ipc::proto::{player, stream};
use crate::ipc::{proto, Error, Result};
use popcorn_fx_core::core::players::{
    PlayRequest, PlaySubtitleRequest, Player, PlayerEvent, PlayerManagerEvent, PlayerState,
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
                        state: player::player::State::from(state).into(),
                        special_fields: Default::default(),
                    });
            }
        }

        event
    }
}

impl TryFrom<player::PlayerEvent> for PlayerEvent {
    type Error = Error;

    fn try_from(value: player::PlayerEvent) -> Result<Self> {
        let event = value
            .event
            .enum_value()
            .map_err(|_| Error::UnsupportedEnum)?;

        match event {
            Event::DURATION_CHANGED => Ok(PlayerEvent::DurationChanged(
                value
                    .duration_changed
                    .into_option()
                    .map(|e| e.duration)
                    .ok_or(Error::MissingField)?,
            )),
            Event::TIME_CHANGED => Ok(PlayerEvent::TimeChanged(
                value
                    .time_changed
                    .into_option()
                    .map(|e| e.time)
                    .ok_or(Error::MissingField)?,
            )),
            Event::STATE_CHANGED => Ok(PlayerEvent::StateChanged(PlayerState::from(
                &value
                    .state_changed
                    .into_option()
                    .map(|e| e.state)
                    .ok_or(Error::MissingField)?
                    .enum_value()
                    .map_err(|_| Error::UnsupportedEnum)?,
            ))),
            Event::VOLUME_CHANGED => Ok(PlayerEvent::VolumeChanged(
                value
                    .volume_changed
                    .into_option()
                    .map(|e| e.volume)
                    .ok_or(Error::MissingField)?,
            )),
        }
    }
}

impl From<PlayerEvent> for player::PlayerEvent {
    fn from(value: PlayerEvent) -> Self {
        match value {
            PlayerEvent::DurationChanged(duration) => Self {
                event: Event::DURATION_CHANGED.into(),
                duration_changed: MessageField::some(player::player_event::DurationChanged {
                    duration,
                    special_fields: Default::default(),
                }),
                time_changed: Default::default(),
                state_changed: Default::default(),
                volume_changed: Default::default(),
                special_fields: Default::default(),
            },
            PlayerEvent::TimeChanged(time) => Self {
                event: Event::TIME_CHANGED.into(),
                duration_changed: Default::default(),
                time_changed: MessageField::some(player::player_event::TimeChanged {
                    time,
                    special_fields: Default::default(),
                }),
                state_changed: Default::default(),
                volume_changed: Default::default(),
                special_fields: Default::default(),
            },
            PlayerEvent::StateChanged(state) => Self {
                event: Event::STATE_CHANGED.into(),
                duration_changed: Default::default(),
                time_changed: Default::default(),
                state_changed: MessageField::some(player::player_event::StateChanged {
                    state: player::player::State::from(&state).into(),
                    special_fields: Default::default(),
                }),
                volume_changed: Default::default(),
                special_fields: Default::default(),
            },
            PlayerEvent::VolumeChanged(volume) => Self {
                event: Event::VOLUME_CHANGED.into(),
                duration_changed: Default::default(),
                time_changed: Default::default(),
                state_changed: Default::default(),
                volume_changed: MessageField::some(player::player_event::VolumeChanged {
                    volume,
                    special_fields: Default::default(),
                }),
                special_fields: Default::default(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod player_state {
        use super::*;

        #[test]
        fn test_proto_from() {
            let result = player::player::State::from(&PlayerState::Unknown);
            assert_eq!(player::player::State::UNKNOWN, result);

            let result = player::player::State::from(&PlayerState::Ready);
            assert_eq!(player::player::State::READY, result);

            let result = player::player::State::from(&PlayerState::Loading);
            assert_eq!(player::player::State::LOADING, result);

            let result = player::player::State::from(&PlayerState::Buffering);
            assert_eq!(player::player::State::BUFFERING, result);

            let result = player::player::State::from(&PlayerState::Playing);
            assert_eq!(player::player::State::PLAYING, result);

            let result = player::player::State::from(&PlayerState::Paused);
            assert_eq!(player::player::State::PAUSED, result);

            let result = player::player::State::from(&PlayerState::Stopped);
            assert_eq!(player::player::State::STOPPED, result);

            let result = player::player::State::from(&PlayerState::Error);
            assert_eq!(player::player::State::ERROR, result);
        }

        #[test]
        fn test_state_from() {
            let result = PlayerState::from(&player::player::State::UNKNOWN);
            assert_eq!(PlayerState::Unknown, result);

            let result = PlayerState::from(&player::player::State::READY);
            assert_eq!(PlayerState::Ready, result);

            let result = PlayerState::from(&player::player::State::LOADING);
            assert_eq!(PlayerState::Loading, result);

            let result = PlayerState::from(&player::player::State::BUFFERING);
            assert_eq!(PlayerState::Buffering, result);

            let result = PlayerState::from(&player::player::State::PLAYING);
            assert_eq!(PlayerState::Playing, result);

            let result = PlayerState::from(&player::player::State::PAUSED);
            assert_eq!(PlayerState::Paused, result);

            let result = PlayerState::from(&player::player::State::STOPPED);
            assert_eq!(PlayerState::Stopped, result);

            let result = PlayerState::from(&player::player::State::ERROR);
            assert_eq!(PlayerState::Error, result);
        }
    }

    mod player_event {
        use super::*;
        use protobuf::EnumOrUnknown;

        #[test]
        fn test_proto_from() {
            let result = player::PlayerEvent::from(PlayerEvent::DurationChanged(24000));
            assert_eq!(
                result,
                player::PlayerEvent {
                    event: Event::DURATION_CHANGED.into(),
                    duration_changed: MessageField::some(player::player_event::DurationChanged {
                        duration: 24000,
                        special_fields: Default::default(),
                    }),
                    time_changed: Default::default(),
                    state_changed: Default::default(),
                    volume_changed: Default::default(),
                    special_fields: Default::default(),
                }
            );

            let result = player::PlayerEvent::from(PlayerEvent::TimeChanged(10000));
            assert_eq!(
                result,
                player::PlayerEvent {
                    event: Event::TIME_CHANGED.into(),
                    duration_changed: Default::default(),
                    time_changed: MessageField::some(player::player_event::TimeChanged {
                        time: 10000,
                        special_fields: Default::default(),
                    }),
                    state_changed: Default::default(),
                    volume_changed: Default::default(),
                    special_fields: Default::default(),
                }
            );

            let result = player::PlayerEvent::from(PlayerEvent::StateChanged(PlayerState::Playing));
            assert_eq!(
                result,
                player::PlayerEvent {
                    event: Event::STATE_CHANGED.into(),
                    duration_changed: Default::default(),
                    time_changed: Default::default(),
                    state_changed: MessageField::some(player::player_event::StateChanged {
                        state: player::player::State::PLAYING.into(),
                        special_fields: Default::default(),
                    }),
                    volume_changed: Default::default(),
                    special_fields: Default::default(),
                }
            );

            let result = player::PlayerEvent::from(PlayerEvent::VolumeChanged(50));
            assert_eq!(
                result,
                player::PlayerEvent {
                    event: Event::VOLUME_CHANGED.into(),
                    duration_changed: Default::default(),
                    time_changed: Default::default(),
                    state_changed: Default::default(),
                    volume_changed: MessageField::some(player::player_event::VolumeChanged {
                        volume: 50,
                        special_fields: Default::default(),
                    }),
                    special_fields: Default::default(),
                }
            );
        }

        #[test]
        fn test_player_event_try_from() {
            let result = PlayerEvent::try_from(player::PlayerEvent {
                event: Event::DURATION_CHANGED.into(),
                duration_changed: MessageField::some(player::player_event::DurationChanged {
                    duration: 30000,
                    special_fields: Default::default(),
                }),
                time_changed: Default::default(),
                state_changed: Default::default(),
                volume_changed: Default::default(),
                special_fields: Default::default(),
            })
            .unwrap();
            assert_eq!(PlayerEvent::DurationChanged(30000), result);

            let result = PlayerEvent::try_from(player::PlayerEvent {
                event: Event::TIME_CHANGED.into(),
                duration_changed: Default::default(),
                time_changed: MessageField::some(player::player_event::TimeChanged {
                    time: 10000,
                    special_fields: Default::default(),
                }),
                state_changed: Default::default(),
                volume_changed: Default::default(),
                special_fields: Default::default(),
            })
            .unwrap();
            assert_eq!(PlayerEvent::TimeChanged(10000), result);
        }

        #[test]
        fn test_player_event_try_from_invalid_event() {
            let event = player::PlayerEvent {
                event: EnumOrUnknown::from_i32(99),
                duration_changed: Default::default(),
                time_changed: Default::default(),
                state_changed: Default::default(),
                volume_changed: Default::default(),
                special_fields: Default::default(),
            };

            let result = PlayerEvent::try_from(event);

            assert_eq!(Err(Error::UnsupportedEnum), result);
        }
    }
}
