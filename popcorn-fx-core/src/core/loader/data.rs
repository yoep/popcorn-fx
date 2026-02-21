use crate::core::media::MediaIdentifier;
use crate::core::playlist::PlaylistItem;
use crate::core::stream::ServerStream;
use crate::core::subtitles::model::{Subtitle, SubtitleInfo};
use crate::core::torrents::Torrent;

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
    pub quality: Option<String>,
    pub auto_resume_timestamp: Option<u64>,
    pub subtitle: SubtitleData,
    /// The torrent information associated with the media item.
    pub torrent: Option<Box<dyn Torrent>>,
    /// The filename of the torrent that needs to be loaded
    pub torrent_file: Option<String>,
    /// The stream of the media item.
    pub stream: Option<ServerStream>,
}

impl PartialEq for LoadingData {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
            && self.title == other.title
            && self.caption == other.caption
            && self.thumb == other.thumb
            && self.parent_media.is_some() == other.parent_media.is_some()
            && self.media.is_some() == other.media.is_some()
            && self.quality == other.quality
            && self.auto_resume_timestamp == other.auto_resume_timestamp
            && self.torrent.is_some() == other.torrent.is_some()
            && self.torrent_file == other.torrent_file
            && self.stream.is_some() == other.stream.is_some()
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
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
            quality: None,
            auto_resume_timestamp: None,
            subtitle: SubtitleData::default(),
            torrent: None,
            torrent_file: None,
            stream: None,
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
            quality: value.quality,
            auto_resume_timestamp: value.auto_resume_timestamp,
            subtitle: SubtitleData {
                enabled: Some(value.subtitle.enabled),
                info: value.subtitle.info,
                subtitle: None,
            },
            torrent: None,
            torrent_file: value.torrent.filename,
            stream: None,
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
    use crate::core::playlist::{PlaylistMedia, PlaylistSubtitle, PlaylistTorrent};

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
            quality: Some(quality.to_string()),
            auto_resume_timestamp: None,
            subtitle: SubtitleData::default(),
            torrent: None,
            torrent_file: None,
            stream: None,
        };

        let result = LoadingData::from(item);

        assert_eq!(expected_result, result);
    }
}
