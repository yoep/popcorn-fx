use crate::ipc::proto::media::media;
use crate::ipc::proto::playlist;
use crate::ipc::proto::playlist::playlist_event;
use crate::ipc::{Error, Result};
use popcorn_fx_core::core::media::MediaIdentifier;
use popcorn_fx_core::core::playlist::{
    Playlist, PlaylistItem, PlaylistManagerEvent, PlaylistMedia, PlaylistState, PlaylistSubtitle,
    PlaylistTorrent,
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

impl TryFrom<&PlaylistManagerEvent> for playlist::PlaylistEvent {
    type Error = Error;

    fn try_from(value: &PlaylistManagerEvent) -> Result<Self> {
        let mut event = Self::new();

        match value {
            PlaylistManagerEvent::PlaylistChanged => {
                event.event = playlist_event::Event::PLAYLIST_CHANGED.into()
            }
            PlaylistManagerEvent::PlayingNext(playing_next) => {
                event.event = playlist_event::Event::PLAYING_NEXT.into();
                event.playing_next = MessageField::some(playlist_event::PlayingNext {
                    playing_in: playing_next.playing_in.clone(),
                    item: MessageField::some(playlist::playlist::Item::try_from(
                        &playing_next.item,
                    )?),
                    special_fields: Default::default(),
                });
            }
            PlaylistManagerEvent::StateChanged(state) => {
                event.event = playlist_event::Event::STATE_CHANGED.into();
                event.state_changed = MessageField::some(playlist_event::StateChanged {
                    state: playlist::playlist::State::from(state).into(),
                    special_fields: Default::default(),
                });
            }
        }

        Ok(event)
    }
}
