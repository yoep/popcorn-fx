use crate::ipc::proto::playlist;
use crate::ipc::{Error, Result};
use popcorn_fx_core::core::media::MediaIdentifier;
use popcorn_fx_core::core::playlist::{
    Playlist, PlaylistItem, PlaylistMedia, PlaylistSubtitle, PlaylistTorrent,
};

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
