use std::sync::Weak;

use crate::core::media::{MediaIdentifier, TorrentInfo};
use crate::core::playlists::PlaylistItem;
use crate::core::subtitles::model::{Subtitle, SubtitleInfo};
use crate::core::torrents::{Torrent, TorrentFileInfo, TorrentStream};

/// A structure representing loading data for a media item.
///
/// This struct is used to provide loading data for a media item. Either a `url` or an `media` is always present
/// to specify the source of the media item. Additionally, it may contain optional information about the media
/// torrent, torrent stream, or other related data.
#[derive(Debug)]
pub struct LoadingData {
    pub url: Option<String>,
    pub title: Option<String>,
    pub caption: Option<String>,
    pub thumb: Option<String>,
    pub parent_media: Option<Box<dyn MediaIdentifier>>,
    pub media: Option<Box<dyn MediaIdentifier>>,
    pub torrent_info: Option<crate::core::torrents::TorrentInfo>,
    pub torrent_file_info: Option<TorrentFileInfo>,
    pub quality: Option<String>,
    pub auto_resume_timestamp: Option<u64>,
    pub subtitle: SubtitleData,
    pub media_torrent_info: Option<TorrentInfo>,
    pub torrent: Option<Box<dyn Torrent>>,
    pub torrent_stream: Option<Weak<Box<dyn TorrentStream>>>,
}

impl PartialEq for LoadingData {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
            && self.title == other.title
            && self.caption == other.caption
            && self.thumb == other.thumb
            && self.parent_media.is_some() == other.parent_media.is_some()
            && self.media.is_some() == other.media.is_some()
            && self.torrent_info == other.torrent_info
            && self.torrent_file_info == other.torrent_file_info
            && self.quality == other.quality
            && self.auto_resume_timestamp == other.auto_resume_timestamp
            && self.torrent.is_some() == other.torrent.is_some()
            && self.torrent_stream.is_some() == other.torrent_stream.is_some()
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Clone for LoadingData {
    fn clone(&self) -> Self {
        let cloned_parent_media = match &self.parent_media {
            None => None,
            Some(media) => media.clone_identifier(),
        };
        let cloned_media = match &self.media {
            None => None,
            Some(media) => media.clone_identifier(),
        };

        Self {
            url: self.url.clone(),
            title: self.title.clone(),
            caption: self.caption.clone(),
            thumb: self.thumb.clone(),
            parent_media: cloned_parent_media,
            media: cloned_media,
            torrent_info: self.torrent_info.clone(),
            torrent_file_info: self.torrent_file_info.clone(),
            quality: self.quality.clone(),
            auto_resume_timestamp: self.auto_resume_timestamp,
            subtitle: self.subtitle.clone(),
            media_torrent_info: self.media_torrent_info.clone(),
            torrent: None,
            torrent_stream: self.torrent_stream.clone(),
        }
    }
}

impl From<&str> for LoadingData {
    fn from(value: &str) -> Self {
        Self {
            url: Some(value.to_string()),
            title: None,
            caption: None,
            thumb: None,
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitle: SubtitleData::default(),
            media_torrent_info: None,
            torrent: None,
            torrent_stream: None,
        }
    }
}

impl From<PlaylistItem> for LoadingData {
    fn from(value: PlaylistItem) -> Self {
        Self {
            url: value.url,
            title: Some(value.title),
            caption: value.caption,
            thumb: value.thumb,
            parent_media: value.media.parent,
            media: value.media.media,
            torrent_info: value.torrent.info,
            torrent_file_info: value.torrent.file_info,
            quality: value.quality,
            auto_resume_timestamp: value.auto_resume_timestamp,
            subtitle: SubtitleData {
                enabled: Some(value.subtitle.enabled),
                info: value.subtitle.info,
                subtitle: None,
            },
            media_torrent_info: None,
            torrent: None,
            torrent_stream: None,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SubtitleData {
    pub enabled: Option<bool>,
    pub info: Option<SubtitleInfo>,
    pub subtitle: Option<Subtitle>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::media::{Episode, ShowOverview};
    use crate::core::playlists::{PlaylistMedia, PlaylistSubtitle, PlaylistTorrent};

    #[test]
    fn test_from_str() {
        let url = "SomeUrl";

        let result = LoadingData::from(url);

        assert_eq!(Some(url.to_string()), result.url);
    }

    #[test]
    fn test_from_playlist_item() {
        let title = "MyTitle";
        let caption = "MyCaption";
        let thumb = "MyThumb";
        let quality = "480p";
        let show_overview = ShowOverview {
            imdb_id: "tt123456".to_string(),
            tvdb_id: "tt000001".to_string(),
            title: "FooBar".to_string(),
            year: "2019".to_string(),
            num_seasons: 2,
            images: Default::default(),
            rating: None,
        };
        let episode = Episode {
            season: 1,
            episode: 3,
            first_aired: 0,
            title: "LoremIpsum".to_string(),
            overview: "Some random overview".to_string(),
            tvdb_id: 0,
            tvdb_id_value: "".to_string(),
            thumb: None,
            torrents: Default::default(),
        };
        let item = PlaylistItem {
            url: None,
            title: title.to_string(),
            caption: Some(caption.to_string()),
            thumb: Some(thumb.to_string()),
            media: PlaylistMedia {
                parent: Some(Box::new(show_overview.clone())),
                media: Some(Box::new(episode.clone())),
            },
            quality: Some(quality.to_string()),
            auto_resume_timestamp: None,
            subtitle: PlaylistSubtitle::default(),
            torrent: PlaylistTorrent::default(),
        };
        let expected_result = LoadingData {
            url: None,
            title: Some(title.to_string()),
            caption: Some(caption.to_string()),
            thumb: Some(thumb.to_string()),
            parent_media: Some(Box::new(show_overview)),
            media: Some(Box::new(episode)),
            torrent_info: None,
            torrent_file_info: None,
            quality: Some(quality.to_string()),
            auto_resume_timestamp: None,
            subtitle: SubtitleData::default(),
            media_torrent_info: None,
            torrent: None,
            torrent_stream: None,
        };

        let result = LoadingData::from(item);

        assert_eq!(expected_result, result);
    }
}
