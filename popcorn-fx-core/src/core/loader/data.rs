use std::sync::Weak;

use crate::core::media::{MediaIdentifier, TorrentInfo};
use crate::core::players::{PlayUrlRequest, PlayUrlRequestBuilder};
use crate::core::playlists::PlaylistItem;
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
    pub subtitles_enabled: Option<bool>,
    pub media_torrent_info: Option<TorrentInfo>,
    pub torrent: Option<Weak<Box<dyn Torrent>>>,
    pub torrent_stream: Option<Weak<Box<dyn TorrentStream>>>,
}

impl PartialEq for LoadingData {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url &&
            self.title == other.title &&
            self.caption == other.caption &&
            self.thumb == other.thumb &&
            self.parent_media.is_some() == other.parent_media.is_some() &&
            self.media.is_some() == other.media.is_some() &&
            self.torrent_info == other.torrent_info &&
            self.torrent_file_info == other.torrent_file_info &&
            self.quality == other.quality &&
            self.auto_resume_timestamp == other.auto_resume_timestamp &&
            self.torrent.is_some() == other.torrent.is_some() &&
            self.torrent_stream.is_some() == other.torrent_stream.is_some()
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Clone for LoadingData {
    fn clone(&self) -> Self {
        let cloned_parent_media = match &self.parent_media {
            None => None,
            Some(media) => {
                media.clone_identifier()
            }
        };
        let cloned_media = match &self.media {
            None => None,
            Some(media) => {
                media.clone_identifier()
            }
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
            subtitles_enabled: self.subtitles_enabled,
            media_torrent_info: self.media_torrent_info.clone(),
            torrent: self.torrent.clone(),
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
            subtitles_enabled: None,
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
            parent_media: value.parent_media,
            media: value.media,
            torrent_info: value.torrent_info,
            torrent_file_info: value.torrent_file_info,
            quality: value.quality,
            auto_resume_timestamp: value.auto_resume_timestamp,
            subtitles_enabled: Some(value.subtitles_enabled),
            media_torrent_info: None,
            torrent: None,
            torrent_stream: None,
        }
    }
}

impl From<LoadingData> for PlayUrlRequest {
    fn from(value: LoadingData) -> Self {
        let mut builder = PlayUrlRequestBuilder::builder()
            .url(value.url.expect("expected an url to have been present").as_str())
            .title(value.title.unwrap_or(String::new()).as_str())
            .subtitles_enabled(value.subtitles_enabled.unwrap_or(false));

        if let Some(e) = value.thumb {
            builder = builder.thumb(e.as_str());
        }
        if let Some(e) = value.auto_resume_timestamp {
            builder = builder.auto_resume_timestamp(e);
        }

        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let item = PlaylistItem {
            url: None,
            title: title.to_string(),
            caption: Some(caption.to_string()),
            thumb: Some(thumb.to_string()),
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: Some(quality.to_string()),
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        };

        let result = LoadingData::from(item);
        
        assert_eq!(Some(title.to_string()), result.title);
        assert_eq!(Some(caption.to_string()), result.caption);
        assert_eq!(Some(thumb.to_string()), result.thumb);
        assert_eq!(Some(quality.to_string()), result.quality);
    }
    
    #[test]
    fn test_from_play_url_request() {
        let url = "http://localhost:8080/movie.mp4";
        let title = "FooBar";
        let thumb = "http://localhost:8080/thumbnail.jpg";
        let data = LoadingData {
            url: Some(url.to_string()),
            title: Some(title.to_string()),
            caption: None,
            thumb: Some(thumb.to_string()),
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: Some(true),
            media_torrent_info: None,
            torrent: None,
            torrent_stream: None,
        };
        let expected = PlayUrlRequest {
            url: url.to_string(),
            title: title.to_string(),
            caption: None,
            thumb: Some(thumb.to_string()),
            background: None,
            auto_resume_timestamp: None,
            subtitles_enabled: true,
        };

        let result = PlayUrlRequest::from(data);
        
        assert_eq!(expected, result);
    }
}