use crate::ipc::proto::media::media;
use crate::ipc::proto::playlist;
use crate::ipc::proto::playlist::playlist_event;
use crate::ipc::{Error, Result};
use popcorn_fx_core::core::media::MediaIdentifier;
use popcorn_fx_core::core::playlist::{
    PlayNext, Playlist, PlaylistItem, PlaylistManagerEvent, PlaylistMedia, PlaylistState,
    PlaylistSubtitle, PlaylistTorrent,
};
use protobuf::MessageField;

impl TryFrom<&playlist::Playlist> for Playlist {
    type Error = Error;

    fn try_from(value: &playlist::Playlist) -> Result<Self> {
        Ok(Self {
            items: value
                .items
                .iter()
                .map(PlaylistItem::try_from)
                .collect::<Result<Vec<PlaylistItem>>>()?
                .into(),
        })
    }
}

impl TryFrom<&Playlist> for playlist::Playlist {
    type Error = Error;

    fn try_from(value: &Playlist) -> Result<Self> {
        Ok(Self {
            items: value
                .items
                .iter()
                .map(playlist::playlist::Item::try_from)
                .collect::<Result<Vec<playlist::playlist::Item>>>()?
                .into(),
            special_fields: Default::default(),
        })
    }
}

impl TryFrom<&playlist::playlist::Item> for PlaylistItem {
    type Error = Error;

    fn try_from(value: &playlist::playlist::Item) -> Result<Self> {
        let parent_media = value
            .parent_media
            .as_ref()
            .map(Box::<dyn MediaIdentifier>::try_from)
            .transpose()?;
        let media = value
            .media
            .as_ref()
            .map(Box::<dyn MediaIdentifier>::try_from)
            .transpose()?;

        Ok(Self {
            url: Some(value.url.clone()),
            title: value.title.clone(),
            caption: value.caption.clone(),
            thumb: value.thumb.clone(),
            media: PlaylistMedia {
                parent: parent_media,
                media,
            },
            quality: value.quality.clone(),
            auto_resume_timestamp: value.auto_resume_timestamp.clone(),
            subtitle: PlaylistSubtitle {
                enabled: value.subtitles_enabled,
                info: None,
            },
            torrent: PlaylistTorrent {
                filename: value.torrent_filename.clone(),
            },
        })
    }
}

impl TryFrom<&PlaylistItem> for playlist::playlist::Item {
    type Error = Error;

    fn try_from(value: &PlaylistItem) -> Result<Self> {
        let parent_media = value
            .media
            .parent
            .as_ref()
            .map(media::Item::try_from)
            .transpose()?;
        let media = value
            .media
            .media
            .as_ref()
            .map(media::Item::try_from)
            .transpose()?;

        Ok(Self {
            url: value.url.clone().unwrap_or_default(),
            title: value.title.clone(),
            caption: value.caption.clone(),
            thumb: value.thumb.clone(),
            quality: value.quality.clone(),
            parent_media: MessageField::from_option(parent_media),
            media: MessageField::from_option(media),
            auto_resume_timestamp: value.auto_resume_timestamp.clone(),
            subtitles_enabled: value.subtitle.enabled,
            torrent_filename: value.torrent.filename.clone(),
            special_fields: Default::default(),
        })
    }
}

impl From<&PlaylistState> for playlist::playlist::State {
    fn from(value: &PlaylistState) -> Self {
        match value {
            PlaylistState::Idle => Self::IDLE,
            PlaylistState::Playing => Self::PLAYING,
            PlaylistState::Stopped => Self::STOPPED,
            PlaylistState::Completed => Self::COMPLETED,
            PlaylistState::Error => Self::ERROR,
        }
    }
}

impl TryFrom<&PlayNext> for playlist::PlayNext {
    type Error = Error;

    fn try_from(value: &PlayNext) -> Result<Self> {
        match value {
            PlayNext::Next(item) => Ok(playlist::PlayNext {
                type_: playlist::play_next::Type::NEXT.into(),
                next: MessageField::some(playlist::play_next::Next {
                    item: MessageField::some(playlist::playlist::Item::try_from(item)?),
                    special_fields: Default::default(),
                }),
                special_fields: Default::default(),
            }),
            PlayNext::End => Ok(playlist::PlayNext {
                type_: playlist::play_next::Type::END.into(),
                next: Default::default(),
                special_fields: Default::default(),
            }),
        }
    }
}

impl TryFrom<&PlaylistManagerEvent> for playlist::PlaylistEvent {
    type Error = Error;

    fn try_from(value: &PlaylistManagerEvent) -> Result<Self> {
        match value {
            PlaylistManagerEvent::PlaylistChanged => Ok(Self {
                event: playlist_event::Event::PLAYLIST_CHANGED.into(),
                play_next_changed: Default::default(),
                playing_next_in: Default::default(),
                state_changed: Default::default(),
                special_fields: Default::default(),
            }),
            PlaylistManagerEvent::PlayNextChanged(next) => Ok(Self {
                event: playlist_event::Event::PLAY_NEXT_CHANGED.into(),
                play_next_changed: MessageField::some(playlist_event::PlayNextChanged {
                    next: MessageField::some(playlist::PlayNext::try_from(next)?),
                    special_fields: Default::default(),
                }),
                playing_next_in: Default::default(),
                state_changed: Default::default(),
                special_fields: Default::default(),
            }),
            PlaylistManagerEvent::PlayingNextIn(playing_in_seconds) => Ok(Self {
                event: playlist_event::Event::PLAYING_NEXT_IN.into(),
                play_next_changed: Default::default(),
                playing_next_in: MessageField::some(playlist_event::PlayingNextIn {
                    playing_in_seconds: *playing_in_seconds,
                    special_fields: Default::default(),
                }),
                state_changed: Default::default(),
                special_fields: Default::default(),
            }),
            PlaylistManagerEvent::PlayingNextInAborted => Ok(Self {
                event: playlist_event::Event::PLAYING_NEXT_IN_ABORTED.into(),
                play_next_changed: Default::default(),
                playing_next_in: Default::default(),
                state_changed: Default::default(),
                special_fields: Default::default(),
            }),
            PlaylistManagerEvent::StateChanged(state) => Ok(Self {
                event: playlist_event::Event::STATE_CHANGED.into(),
                play_next_changed: Default::default(),
                playing_next_in: Default::default(),
                state_changed: MessageField::some(playlist_event::StateChanged {
                    state: playlist::playlist::State::from(state).into(),
                    special_fields: Default::default(),
                }),
                special_fields: Default::default(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod play_next {
        use super::*;

        #[test]
        fn test_proto_try_from() {
            let result = playlist::PlayNext::try_from(&PlayNext::Next(PlaylistItem {
                url: Some("http://localhost:8080/my-video.mp4".to_string()),
                title: "FooBar".to_string(),
                caption: Some("MyCaption".to_string()),
                thumb: None,
                media: Default::default(),
                quality: None,
                auto_resume_timestamp: None,
                subtitle: Default::default(),
                torrent: Default::default(),
            }))
            .unwrap();
            assert_eq!(
                result,
                playlist::PlayNext {
                    type_: playlist::play_next::Type::NEXT.into(),
                    next: MessageField::some(playlist::play_next::Next {
                        item: MessageField::some(playlist::playlist::Item {
                            url: "http://localhost:8080/my-video.mp4".to_string(),
                            title: "FooBar".to_string(),
                            caption: Some("MyCaption".to_string()),
                            thumb: None,
                            quality: None,
                            parent_media: Default::default(),
                            media: Default::default(),
                            auto_resume_timestamp: None,
                            subtitles_enabled: false,
                            torrent_filename: None,
                            special_fields: Default::default(),
                        }),
                        special_fields: Default::default(),
                    }),
                    special_fields: Default::default(),
                }
            );
        }
    }

    mod playlist_event {
        use super::*;

        #[test]
        fn test_proto_try_from() {
            let result =
                playlist::PlaylistEvent::try_from(&PlaylistManagerEvent::PlaylistChanged).unwrap();
            assert_eq!(
                result,
                create_playlist_event(playlist::playlist_event::Event::PLAYLIST_CHANGED)
            );

            let result =
                playlist::PlaylistEvent::try_from(&PlaylistManagerEvent::PlayingNextIn(40))
                    .unwrap();
            assert_eq!(
                result,
                playlist::PlaylistEvent {
                    event: playlist::playlist_event::Event::PLAYING_NEXT_IN.into(),
                    play_next_changed: Default::default(),
                    playing_next_in: MessageField::some(playlist::playlist_event::PlayingNextIn {
                        playing_in_seconds: 40,
                        special_fields: Default::default(),
                    }),
                    state_changed: Default::default(),
                    special_fields: Default::default(),
                }
            );

            let result =
                playlist::PlaylistEvent::try_from(&PlaylistManagerEvent::PlayingNextInAborted)
                    .unwrap();
            assert_eq!(
                result,
                create_playlist_event(playlist::playlist_event::Event::PLAYING_NEXT_IN_ABORTED)
            );
        }

        #[test]
        fn test_proto_try_from_play_next_changed() {
            let url = "http://localhost:8080/my-video.mp4";
            let title = "FooBar";
            let caption = "MyCaption";
            let item = PlaylistItem {
                url: Some(url.to_string()),
                title: title.to_string(),
                caption: Some(caption.to_string()),
                thumb: None,
                media: Default::default(),
                quality: None,
                auto_resume_timestamp: None,
                subtitle: Default::default(),
                torrent: Default::default(),
            };
            let expected_item = playlist::playlist::Item {
                url: url.to_string(),
                title: title.to_string(),
                caption: Some(caption.to_string()),
                thumb: None,
                quality: None,
                parent_media: Default::default(),
                media: Default::default(),
                auto_resume_timestamp: None,
                subtitles_enabled: false,
                torrent_filename: None,
                special_fields: Default::default(),
            };

            let result = playlist::PlaylistEvent::try_from(&PlaylistManagerEvent::PlayNextChanged(
                PlayNext::End,
            ))
            .unwrap();

            assert_eq!(
                result,
                playlist::PlaylistEvent {
                    event: playlist::playlist_event::Event::PLAY_NEXT_CHANGED.into(),
                    play_next_changed: MessageField::some(
                        playlist::playlist_event::PlayNextChanged {
                            next: MessageField::some(playlist::PlayNext {
                                type_: playlist::play_next::Type::END.into(),
                                next: Default::default(),
                                special_fields: Default::default(),
                            }),
                            special_fields: Default::default(),
                        }
                    ),
                    playing_next_in: Default::default(),
                    state_changed: Default::default(),
                    special_fields: Default::default(),
                }
            );
        }

        fn create_playlist_event(
            event: playlist::playlist_event::Event,
        ) -> playlist::PlaylistEvent {
            playlist::PlaylistEvent {
                event: event.into(),
                play_next_changed: Default::default(),
                playing_next_in: Default::default(),
                state_changed: Default::default(),
                special_fields: Default::default(),
            }
        }
    }
}
