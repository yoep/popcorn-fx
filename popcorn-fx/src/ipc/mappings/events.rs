use crate::ipc::proto::events;
use crate::ipc::proto::events::event::{EventType, PlaybackStateChanged, TorrentDetailsLoaded};
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
            Event::TorrentDetailsLoaded(details) => {
                event.type_ = EventType::TORRENT_DETAILS_LOADED.into();
                event.torrent_details_loaded = MessageField::some(TorrentDetailsLoaded {
                    torrent_info: MessageField::some(details.into()),
                    special_fields: Default::default(),
                });
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
    use std::path::PathBuf;

    use crate::ipc::proto::torrent::torrent;

    use popcorn_fx_core::core::event::{PlayerStartedEvent, PlayerStoppedEvent};
    use popcorn_fx_core::core::torrents::TorrentInfo;
    use popcorn_fx_torrent::torrent::{TorrentFileInfo, TorrentHandle};

    #[test]
    fn test_event_from_player_started() {
        let event = Event::PlayerStarted(PlayerStartedEvent {
            url: "https://localhost/my-video.mp4".to_string(),
            title: "FooBar".to_string(),
            thumbnail: None,
            background: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        });
        let expected_result = events::Event {
            type_: EventType::PLAYER_STARTED.into(),
            playback_state_changed: Default::default(),
            torrent_details_loaded: Default::default(),
            special_fields: Default::default(),
        };

        let result = events::Event::from(&event);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_event_from_player_stopped() {
        let event = Event::PlayerStopped(PlayerStoppedEvent {
            url: "https://localhost/my-video.mp4".to_string(),
            media: None,
            time: Some(12000),
            duration: Some(64000),
        });
        let expected_result = events::Event {
            type_: EventType::PLAYER_STOPPED.into(),
            playback_state_changed: Default::default(),
            torrent_details_loaded: Default::default(),
            special_fields: Default::default(),
        };

        let result = events::Event::from(&event);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_event_from_torrent_details_loaded() {
        let handle = TorrentHandle::new();
        let event = Event::TorrentDetailsLoaded(TorrentInfo {
            handle,
            info_hash: "InfoHash".to_string(),
            uri: "magnet:?SomeUri".to_string(),
            name: "FooBar".to_string(),
            directory_name: Some("torDir".to_string()),
            total_files: 13,
            files: vec![popcorn_fx_torrent::torrent::File {
                index: 1,
                torrent_path: PathBuf::from("torDir/my-file.mp4"),
                offset: 1000,
                info: TorrentFileInfo {
                    length: 25000,
                    path: None,
                    path_utf8: None,
                    md5sum: Some("md5sum".to_string()),
                    attr: None,
                    symlink_path: None,
                    sha1: None,
                },
                priority: Default::default(),
                pieces: 0..100,
            }],
        });
        let expected_result = events::Event {
            type_: EventType::TORRENT_DETAILS_LOADED.into(),
            playback_state_changed: Default::default(),
            torrent_details_loaded: MessageField::some(TorrentDetailsLoaded {
                torrent_info: MessageField::some(torrent::Info {
                    handle: MessageField::some((&handle).into()),
                    info_hash: "InfoHash".to_string(),
                    uri: "magnet:?SomeUri".to_string(),
                    name: "FooBar".to_string(),
                    directory_name: Some("torDir".to_string()),
                    total_files: 13,
                    files: vec![torrent::info::File {
                        index: 1,
                        filename: "my-file.mp4".to_string(),
                        torrent_path: "torDir/my-file.mp4".to_string(),
                        offset: 1000,
                        length: 25000,
                        md5sum: Some("md5sum".to_string()),
                        sha1: None,
                        special_fields: Default::default(),
                    }],
                    special_fields: Default::default(),
                }),
                special_fields: Default::default(),
            }),
            special_fields: Default::default(),
        };

        let result = events::Event::from(&event);

        assert_eq!(expected_result, result);
    }

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
