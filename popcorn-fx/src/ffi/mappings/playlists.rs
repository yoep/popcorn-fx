use std::{mem, ptr};
use std::os::raw::c_char;

use log::trace;

use popcorn_fx_core::{from_c_into_boxed, from_c_string, into_c_owned, into_c_string};
use popcorn_fx_core::core::media::MediaIdentifier;
use popcorn_fx_core::core::playlists::{
    PlayingNextInfo, PlaylistItem, PlaylistManagerEvent, PlaylistState,
};

use crate::ffi::MediaItemC;

/// The callback function type for playlist manager events in C.
///
/// This type represents a C-compatible function pointer that can be used to handle playlist manager events.
/// When invoked, it receives a `PlaylistManagerEventC` as its argument.
pub type PlaylistManagerCallbackC = extern "C" fn(PlaylistManagerEventC);

/// A C-compatible struct representing a playlist item.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct PlaylistItemC {
    /// The URL of the playlist item.
    pub url: *mut c_char,
    /// The title of the playlist item.
    pub title: *mut c_char,
    /// The caption/subtitle of the playlist item.
    pub caption: *mut c_char,
    /// The thumbnail URL of the playlist item.
    pub thumb: *mut c_char,
    /// The quality information of the playlist item.
    pub quality: *mut c_char,
    /// A pointer to the parent media item, if applicable.
    pub parent_media: *mut MediaItemC,
    /// A pointer to the media item associated with the playlist item.
    pub media: *mut MediaItemC,
    /// A pointer to the timestamp for auto-resume, if applicable.
    pub auto_resume_timestamp: *const u64,
    /// A boolean flag indicating whether subtitles are enabled for the playlist item.
    pub subtitles_enabled: bool,
}

impl PlaylistItemC {
    /// Convert an optional boxed `MediaIdentifier` into a C-compatible pointer.
    ///
    /// # Arguments
    ///
    /// * `value` - An optional boxed `MediaIdentifier` to convert.
    ///
    /// # Returns
    ///
    /// A C-compatible pointer to a `MediaItemC` or a null pointer if `value` is `None`.
    fn media_ptr(value: Option<Box<dyn MediaIdentifier>>) -> *mut MediaItemC {
        if let Some(value) = value {
            into_c_owned(MediaItemC::from(value))
        } else {
            ptr::null_mut()
        }
    }

    /// Convert a C-compatible pointer to a `MediaItemC` into an optional boxed `MediaIdentifier`.
    ///
    /// # Arguments
    ///
    /// * `value` - A C-compatible pointer to a `MediaItemC` to convert.
    ///
    /// # Returns
    ///
    /// An optional boxed `MediaIdentifier` or `None` if `value` is a null pointer.
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
        let caption = if !value.caption.is_null() {
            Some(from_c_string(value.caption))
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
            Some(value.auto_resume_timestamp as u64)
        } else {
            None
        };

        PlaylistItem {
            url,
            title: from_c_string(value.title),
            caption,
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
            ptr::null_mut()
        };
        let caption = if let Some(e) = value.caption {
            into_c_string(e)
        } else {
            ptr::null_mut()
        };
        let thumb = if let Some(e) = value.thumb {
            into_c_string(e)
        } else {
            ptr::null_mut()
        };
        let quality = if let Some(e) = value.quality {
            into_c_string(e)
        } else {
            ptr::null_mut()
        };
        let auto_resume_timestamp = if let Some(e) = value.auto_resume_timestamp {
            e as *const u64
        } else {
            ptr::null_mut()
        };

        Self {
            url,
            title: into_c_string(value.title),
            caption,
            thumb,
            quality,
            parent_media: PlaylistItemC::media_ptr(value.parent_media),
            media: PlaylistItemC::media_ptr(value.media),
            auto_resume_timestamp,
            subtitles_enabled: value.subtitles_enabled,
        }
    }
}

/// A C-compatible enum representing different playlist manager events.
#[repr(C)]
#[derive(Debug)]
pub enum PlaylistManagerEventC {
    /// Represents a playlist change event.
    PlaylistChanged,
    /// Represents an event indicating the next item to be played.
    PlayingNext(PlayingNextInfoC),
    /// Represents a state change event in the playlist manager.
    StateChanged(PlaylistState),
}

impl From<PlaylistManagerEvent> for PlaylistManagerEventC {
    fn from(value: PlaylistManagerEvent) -> Self {
        trace!(
            "Converting playlist manager event {:?} to PlaylistManagerEventC",
            value
        );
        match value {
            PlaylistManagerEvent::PlaylistChanged => PlaylistManagerEventC::PlaylistChanged,
            PlaylistManagerEvent::PlayingNext(e) => {
                PlaylistManagerEventC::PlayingNext(PlayingNextInfoC::from(e))
            }
            PlaylistManagerEvent::StateChanged(e) => PlaylistManagerEventC::StateChanged(e),
        }
    }
}

/// A C-compatible struct representing information about the next item to be played.
#[repr(C)]
#[derive(Debug)]
pub struct PlayingNextInfoC {
    /// A pointer to the timestamp indicating when the next item will be played.
    pub playing_in: *const u64,
    /// A pointer to the next playlist item.
    pub next_item: *mut PlaylistItemC,
}

impl From<PlayingNextInfo> for PlayingNextInfoC {
    fn from(value: PlayingNextInfo) -> Self {
        trace!(
            "Converting playing next info {:?} to PlayingNextInfoC",
            value
        );
        let playing_in = if let Some(e) = value.playing_in {
            e as *const u64
        } else {
            ptr::null()
        };

        Self {
            playing_in,
            next_item: into_c_owned(PlaylistItemC::from(value.item)),
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
            caption: ptr::null_mut(),
            thumb: ptr::null_mut(),
            parent_media: ptr::null_mut(),
            media: into_c_owned(MediaItemC::from(media.clone())),
            quality: into_c_string(quality.to_string()),
            auto_resume_timestamp: into_c_owned(8000u64),
            subtitles_enabled: true,
        };
        let expected_result = PlaylistItem {
            url: Some(url.to_string()),
            title: title.to_string(),
            caption: None,
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
        let caption = "myCaption";
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
            caption: Some(caption.to_string()),
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
        assert_eq!(caption.to_string(), from_c_string(result.caption));
        assert_eq!(quality.to_string(), from_c_string(result.quality));
    }

    #[test]
    fn test_player_item_from() {
        let url = "MyUrl";
        let title = "MyTitle";
        let caption = "MyCaption";
        let thumb = "https://imgur.com";
        let quality = "720p";
        let auto_resume_timestamp = 50u64;
        let item = PlaylistItemC {
            url: into_c_string(url.to_string()),
            title: into_c_string(title.to_string()),
            caption: into_c_string(caption.to_string()),
            thumb: into_c_string(thumb.to_string()),
            quality: into_c_string(quality.to_string()),
            parent_media: ptr::null_mut(),
            media: ptr::null_mut(),
            auto_resume_timestamp: into_c_owned(auto_resume_timestamp.clone()),
            subtitles_enabled: true,
        };
        let expected_result = PlaylistItem {
            url: Some(url.to_string()),
            title: title.to_string(),
            caption: Some(caption.to_string()),
            thumb: Some(thumb.to_string()),
            parent_media: None,
            media: None,
            torrent_info: None,
            torrent_file_info: None,
            quality: Some(quality.to_string()),
            auto_resume_timestamp: Some(auto_resume_timestamp),
            subtitles_enabled: true,
        };

        let result = PlaylistItem::from(item);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_playlist_manager_c_from() {
        let event = PlaylistManagerEvent::PlaylistChanged;

        let result = PlaylistManagerEventC::from(event);

        if let PlaylistManagerEventC::PlaylistChanged = result {
        } else {
            assert!(
                false,
                "expected PlaylistManagerEventC::PlaylistChanged, but got {:?} instead",
                result
            )
        }
    }
}
