use crate::ipc::proto::events;
use crate::ipc::proto::events::event::{EventType, PlaybackStateChanged};
use crate::ipc::proto::player::player;
use crate::ipc::{Error, Result};
use log::warn;
use popcorn_fx_core::core::event::Event;
use popcorn_fx_core::core::playback::PlaybackState;
use protobuf::MessageField;

impl From<&Event> for events::Event {
    fn from(value: &Event) -> Self {
        let mut event = events::Event::new();

        match value {
            Event::PlayerStarted(_) => {
                event.type_ = EventType::PLAYER_STARTED.into();
            }
            Event::PlayerStopped(_) => {
                event.type_ = EventType::PLAYER_STOPPED.into();
            }
            Event::PlaybackStateChanged(state) => {
                event.type_ = EventType::PLAYBACK_STATE_CHANGED.into();
                event.playback_state_changed = MessageField::some(PlaybackStateChanged {
                    new_state: player::State::from(state).into(),
                    special_fields: Default::default(),
                });
            }
            Event::LoadingStarted => {
                event.type_ = EventType::LOADING_STARTED.into();
            }
            Event::LoadingCompleted => {
                event.type_ = EventType::LOADING_COMPLETED.into();
            }
            Event::TorrentDetailsLoaded(_) => {
                event.type_ = EventType::TORRENT_DETAILS_LOADED.into();
            }
            Event::ClosePlayer => {
                event.type_ = EventType::CLOSE_PLAYER.into();
            }
        }

        event
    }
}

impl TryFrom<&events::Event> for Event {
    type Error = Error;

    fn try_from(value: &events::Event) -> Result<Self> {
        let type_ = value
            .type_
            .enum_value()
            .map_err(|_| Error::UnsupportedEnum)?;

        match type_ {
            EventType::PLAYBACK_STATE_CHANGED => {
                let state = value
                    .playback_state_changed
                    .as_ref()
                    .map(|e| {
                        e.new_state
                            .enum_value()
                            .as_ref()
                            .map(PlaybackState::from)
                            .map_err(|enum_value| {
                                warn!("Playback state value {} is not supported", enum_value);
                                Error::UnsupportedEnum
                            })
                    })
                    .transpose()?
                    .ok_or(Error::MissingField)?;

                Ok(Event::PlaybackStateChanged(state))
            }
            _ => Err(Error::UnsupportedMessage(format!("{:?}", type_))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from() {
        let event = events::Event {
            type_: EventType::PLAYBACK_STATE_CHANGED.into(),
            playback_state_changed: MessageField::some(PlaybackStateChanged {
                new_state: player::State::BUFFERING.into(),
                special_fields: Default::default(),
            }),
            torrent_details_loaded: Default::default(),
            special_fields: Default::default(),
        };
        let expected_result = Event::PlaybackStateChanged(PlaybackState::BUFFERING);

        let result = Event::try_from(&event).unwrap();

        assert_eq!(result, expected_result);
    }
}
