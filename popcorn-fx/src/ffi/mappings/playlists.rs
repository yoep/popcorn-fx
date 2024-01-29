use std::{mem, ptr};
use std::os::raw::c_char;

use log::trace;

use popcorn_fx_core::{from_c_into_boxed, from_c_owned, from_c_string, into_c_owned, into_c_string};
use popcorn_fx_core::core::media::MediaIdentifier;
use popcorn_fx_core::core::playlists::PlaylistItem;

use crate::ffi::MediaItemC;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct PlaylistItemC {
    pub url: *const c_char,
    pub title: *const c_char,
    pub thumb: *const c_char,
    pub quality: *const c_char,
    pub parent_media: *mut MediaItemC,
    pub media: *mut MediaItemC,
    pub auto_resume_timestamp: *mut u64,
    pub subtitles_enabled: bool,
}

impl PlaylistItemC {
    fn media_ptr(value: Option<Box<dyn MediaIdentifier>>) -> *mut MediaItemC {
        if let Some(value) = value {
            into_c_owned(MediaItemC::from(value))
        } else {
            ptr::null_mut()
        }
    }

    fn media_value(value: *mut MediaItemC) -> Option<Box<dyn MediaIdentifier>> {
        if !value.is_null() {
            trace!("Converting MediaItem from C for {:?}", value);
            let media = from_c_into_boxed(value);
            let identifier = media.as_identifier();
            mem::forget(media);
            identifier
        } else {
            None
        }
    }
}

impl From<PlaylistItemC> for PlaylistItem {
    fn from(value: PlaylistItemC) -> Self {
        trace!("Mapping PlaylistItemC to PlaylistItem for {:?}", value);
        let url = if !value.url.is_null() {
            Some(from_c_string(value.url))
        } else {
            None
        };
        let thumb = if !value.thumb.is_null() {
            Some(from_c_string(value.thumb))
        } else {
            None
        };
        let parent_media = PlaylistItemC::media_value(value.parent_media);
        let media = PlaylistItemC::media_value(value.media);
        let quality = if !value.quality.is_null() {
            Some(from_c_string(value.quality))
        } else {
            None
        };
        let auto_resume_timestamp = if !value.auto_resume_timestamp.is_null() {
            Some(from_c_owned(value.auto_resume_timestamp))
        } else {
            None
        };

        PlaylistItem {
            url,
            title: from_c_string(value.title),
            thumb,
            parent_media,
            media,
            torrent_info: None,
            torrent_file_info: None,
            quality,
            auto_resume_timestamp,
            subtitles_enabled: value.subtitles_enabled,
        }
    }
}

impl From<PlaylistItem> for PlaylistItemC {
    fn from(value: PlaylistItem) -> Self {
        let url = if let Some(e) = value.url {
            into_c_string(e)
        } else {
            ptr::null()
        };
        let thumb = if let Some(e) = value.thumb {
            into_c_string(e)
        } else {
            ptr::null()
        };
        let quality = if let Some(e) = value.quality {
            into_c_string(e)
        } else {
            ptr::null()
        };
        let auto_resume_timestamp = if let Some(e) = value.auto_resume_timestamp {
            into_c_owned(e)
        } else {
            ptr::null_mut()
        };

        Self {
            url,
            title: into_c_string(value.title),
            thumb,
            quality,
            parent_media: PlaylistItemC::media_ptr(value.parent_media),
            media: PlaylistItemC::media_ptr(value.media),
            auto_resume_timestamp,
            subtitles_enabled: value.subtitles_enabled,
        }
    }
}

#[cfg(test)]
mod test {
    use std::ptr;

    use popcorn_fx_core::{into_c_owned, into_c_string};
    use popcorn_fx_core::core::media::ShowOverview;

    use super::*;

    #[test]
    fn test_playlist_item_from() {
        let url = "MyUrl";
        let title = "FooBar";
        let quality = "720p";
        let media = ShowOverview {
            imdb_id: "tt0000123".to_string(),
            tvdb_id: "tt0000123".to_string(),
            title: "FooBar".to_string(),
            year: "".to_string(),
            num_seasons: 0,
            images: Default::default(),
            rating: None,
        };
        let item = PlaylistItemC {
            url: into_c_string(url.to_string()),
            title: into_c_string(title.to_string()),
            thumb: ptr::null(),
            parent_media: ptr::null_mut(),
            media: into_c_owned(MediaItemC::from(media.clone())),
            quality: into_c_string(quality.to_string()),
            auto_resume_timestamp: into_c_owned(8000u64),
            subtitles_enabled: true,
        };
        let expected_result = PlaylistItem {
            url: Some(url.to_string()),
            title: title.to_string(),
            thumb: None,
            parent_media: None,
            media: Some(Box::new(media)),
            torrent_info: None,
            torrent_file_info: None,
            quality: Some(quality.to_string()),
            auto_resume_timestamp: None,
            subtitles_enabled: true,
        };

        let result = PlaylistItem::from(item);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_playlist_item_c_from() {
        let url = "https://youtube.com/v/qwe654874a";
        let title = "FooBar";
        let quality = "720p";
        let media = ShowOverview {
            imdb_id: "tt0000666".to_string(),
            tvdb_id: "tt0000845".to_string(),
            title: "FooBar".to_string(),
            year: "".to_string(),
            num_seasons: 0,
            images: Default::default(),
            rating: None,
        };
        let item = PlaylistItem {
            url: Some(url.to_string()),
            title: title.to_string(),
            thumb: None,
            parent_media: None,
            media: Some(Box::new(media.clone())),
            torrent_info: None,
            torrent_file_info: None,
            quality: Some(quality.to_string()),
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        };

        let result = PlaylistItemC::from(item);

        assert_eq!(url.to_string(), from_c_string(result.url));
        assert_eq!(title.to_string(), from_c_string(result.title));
        assert_eq!(quality.to_string(), from_c_string(result.quality));
    }
}