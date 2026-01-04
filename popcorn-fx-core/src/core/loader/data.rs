use crate::core::media::MediaIdentifier;
use crate::core::playlist::PlaylistItem;
use crate::core::subtitles::model::{Subtitle, SubtitleInfo};
use crate::core::torrents::{Torrent, TorrentHandle, TorrentStream};
use async_trait::async_trait;
use fx_callback::{Callback, Subscriber, Subscription};
use fx_torrent;
use fx_torrent::{Metrics, PieceIndex, PiecePriority, TorrentEvent, TorrentState};
use std::collections::BTreeMap;
use std::ops::Range;
use std::path::PathBuf;

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
    pub torrent: Option<TorrentData>,
    /// The filename of the torrent that needs to be loaded
    pub torrent_file: Option<String>,
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
        }
    }
}

/// The torrent loading data identifying a torrent or torrent stream.
#[derive(Debug)]
pub enum TorrentData {
    Torrent(Box<dyn Torrent>),
    Stream(Box<dyn TorrentStream>),
}

impl TorrentData {
    /// Check if the torrent data is a torrent stream.
    /// Returns true if the torrent data is a torrent stream.
    pub fn is_stream(&self) -> bool {
        matches!(self, TorrentData::Stream(_))
    }
}

impl Callback<TorrentEvent> for TorrentData {
    fn subscribe(&self) -> Subscription<TorrentEvent> {
        match self {
            TorrentData::Torrent(e) => e.subscribe(),
            TorrentData::Stream(e) => e.subscribe(),
        }
    }

    fn subscribe_with(&self, subscriber: Subscriber<TorrentEvent>) {
        match self {
            TorrentData::Torrent(e) => e.subscribe_with(subscriber),
            TorrentData::Stream(e) => e.subscribe_with(subscriber),
        }
    }
}

#[async_trait]
impl Torrent for TorrentData {
    fn handle(&self) -> TorrentHandle {
        match self {
            TorrentData::Torrent(e) => e.handle(),
            TorrentData::Stream(e) => e.handle(),
        }
    }

    async fn absolute_file_path(&self, file: &fx_torrent::File) -> PathBuf {
        match self {
            TorrentData::Torrent(e) => e.absolute_file_path(file).await,
            TorrentData::Stream(e) => e.absolute_file_path(file).await,
        }
    }

    async fn files(&self) -> Vec<fx_torrent::File> {
        match self {
            TorrentData::Torrent(e) => e.files().await,
            TorrentData::Stream(e) => e.files().await,
        }
    }

    async fn file_by_name(&self, name: &str) -> Option<fx_torrent::File> {
        match self {
            TorrentData::Torrent(e) => e.file_by_name(name).await,
            TorrentData::Stream(e) => e.file_by_name(name).await,
        }
    }

    async fn largest_file(&self) -> Option<fx_torrent::File> {
        match self {
            TorrentData::Torrent(e) => e.largest_file().await,
            TorrentData::Stream(e) => e.largest_file().await,
        }
    }

    async fn has_bytes(&self, bytes: &Range<usize>) -> bool {
        match self {
            TorrentData::Torrent(e) => e.has_bytes(bytes).await,
            TorrentData::Stream(e) => e.has_bytes(bytes).await,
        }
    }

    async fn has_piece(&self, piece: usize) -> bool {
        match self {
            TorrentData::Torrent(e) => e.has_piece(piece).await,
            TorrentData::Stream(e) => e.has_piece(piece).await,
        }
    }

    async fn prioritize_bytes(&self, bytes: &Range<usize>) {
        match self {
            TorrentData::Torrent(e) => e.prioritize_bytes(bytes).await,
            TorrentData::Stream(e) => e.prioritize_bytes(bytes).await,
        }
    }

    async fn prioritize_pieces(&self, pieces: &[PieceIndex]) {
        match self {
            TorrentData::Torrent(e) => e.prioritize_pieces(pieces).await,
            TorrentData::Stream(e) => e.prioritize_pieces(pieces).await,
        }
    }

    async fn piece_priorities(&self) -> BTreeMap<PieceIndex, PiecePriority> {
        match self {
            TorrentData::Torrent(e) => e.piece_priorities().await,
            TorrentData::Stream(e) => e.piece_priorities().await,
        }
    }

    async fn total_pieces(&self) -> usize {
        match self {
            TorrentData::Torrent(e) => e.total_pieces().await,
            TorrentData::Stream(e) => e.total_pieces().await,
        }
    }

    async fn sequential_mode(&self) {
        match self {
            TorrentData::Torrent(e) => e.sequential_mode().await,
            TorrentData::Stream(e) => e.sequential_mode().await,
        }
    }

    async fn state(&self) -> TorrentState {
        match self {
            TorrentData::Torrent(e) => e.state().await,
            TorrentData::Stream(e) => e.state().await,
        }
    }

    fn stats(&self) -> &Metrics {
        match self {
            TorrentData::Torrent(e) => e.stats(),
            TorrentData::Stream(e) => e.stats(),
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
        };

        let result = LoadingData::from(item);

        assert_eq!(expected_result, result);
    }
}
