use std::mem;
use std::os::raw::c_char;

use log::trace;

use popcorn_fx_core::{from_c_into_boxed, from_c_string};
use popcorn_fx_core::core::playlists::PlaylistItem;

use crate::ffi::MediaItemC;

#[repr(C)]
#[derive(Debug)]
pub struct PlaylistItemC {
    pub url: *const c_char,
    pub title: *const c_char,
    pub thumb: *const c_char,
    pub quality: *const c_char,
    pub media: *mut MediaItemC,
}

impl PlaylistItemC {
    pub fn to_struct(&self) -> PlaylistItem {
        let url = if !self.url.is_null() {
            Some(from_c_string(self.url))
        } else {
            None
        };
        let thumb = if !self.thumb.is_null() {
            Some(from_c_string(self.thumb))
        } else {
            None
        };
        let media = if !self.media.is_null() {
            trace!("Converting MediaItem from C for {:?}", self.media);
            let media = from_c_into_boxed(self.media);
            let identifier = media.as_identifier();
            mem::forget(media);
            identifier
        } else {
            None
        };
        let quality = if !self.quality.is_null() {
            Some(from_c_string(self.quality))
        } else {
            None
        };

        PlaylistItem {
            url,
            title: from_c_string(self.title),
            thumb,
            media,
            quality,
            auto_resume_timestamp: None,
            subtitles_enabled: false,
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
    fn test_playlist_item_to_struct() {
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
            media: into_c_owned(MediaItemC::from(media.clone())),
            quality: into_c_string(quality.to_string()),
        };
        let expected_result = PlaylistItem {
            url: Some(url.to_string()),
            title: title.to_string(),
            thumb: None,
            media: Some(Box::new(media)),
            quality: Some(quality.to_string()),
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        };

        let result = item.to_struct();

        assert_eq!(expected_result, result)
    }
}