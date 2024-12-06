use log::trace;
use std::os::raw::c_char;
use std::ptr;

use popcorn_fx_core::core::loader::{
    LoaderEvent, LoadingError, LoadingProgress, LoadingStartedEvent, LoadingState,
};
use popcorn_fx_core::{from_c_string, into_c_string};

/// A C-compatible callback function type for loader events.
pub type LoaderEventCallback = extern "C" fn(LoaderEventC);

/// A C-compatible handle representing a loading process.
///
/// This type is used to represent a loading process and is exposed as a C-compatible handle.
/// It points to the memory location where loading process information is stored in a C context.
pub type LoadingHandleC = *const i64;

/// A C-compatible enum representing loader events.
#[repr(C)]
#[derive(Debug)]
pub enum LoaderEventC {
    LoadingStarted(i64, LoadingStartedEventC),
    StateChanged(i64, LoadingState),
    ProgressChanged(i64, LoadingProgressC),
    LoaderError(i64, LoadingErrorC),
}

impl From<LoaderEvent> for LoaderEventC {
    fn from(value: LoaderEvent) -> Self {
        match value {
            LoaderEvent::LoadingStarted(handle, e) => {
                LoaderEventC::LoadingStarted(handle.value(), LoadingStartedEventC::from(e))
            }
            LoaderEvent::StateChanged(handle, e) => LoaderEventC::StateChanged(handle.value(), e),
            LoaderEvent::LoadingError(handle, e) => {
                LoaderEventC::LoaderError(handle.value(), LoadingErrorC::from(e))
            }
            LoaderEvent::ProgressChanged(handle, e) => {
                LoaderEventC::ProgressChanged(handle.value(), LoadingProgressC::from(e))
            }
        }
    }
}

/// A C-compatible struct representing the event when loading starts.
/// A C-compatible struct representing the event when loading starts.
#[repr(C)]
#[derive(Debug)]
pub struct LoadingStartedEventC {
    /// The URL of the media being loaded.
    pub url: *mut c_char,
    /// The title or name of the media being loaded.
    pub title: *mut c_char,
    /// The URL of a thumbnail image associated with the media, or `ptr::null()` if not available.
    pub thumbnail: *mut c_char,
    /// The URL of a background image associated with the media, or `ptr::null()` if not available.
    pub background: *mut c_char,
    /// The quality or resolution information of the media, or `ptr::null()` if not available.
    pub quality: *mut c_char,
}

/// Convert a `LoadingStartedEvent` into a C-compatible `LoadingStartedEventC`.
impl From<LoadingStartedEvent> for LoadingStartedEventC {
    fn from(value: LoadingStartedEvent) -> Self {
        trace!(
            "Converting `LoadingStartedEvent` into `LoadingStartedEventC` for {:?}",
            value
        );
        let thumbnail = if let Some(e) = value.thumbnail {
            into_c_string(e)
        } else {
            ptr::null_mut()
        };
        let background = if let Some(e) = value.background {
            into_c_string(e)
        } else {
            ptr::null_mut()
        };
        let quality = if let Some(e) = value.quality {
            into_c_string(e)
        } else {
            ptr::null_mut()
        };

        Self {
            url: into_c_string(value.url),
            title: into_c_string(value.title),
            thumbnail,
            background,
            quality,
        }
    }
}

/// Convert a C-compatible `LoadingStartedEventC` into a `LoadingStartedEvent`.
impl From<LoadingStartedEventC> for LoadingStartedEvent {
    fn from(value: LoadingStartedEventC) -> Self {
        trace!(
            "Converting `LoadingStartedEventC` into `LoadingStartedEvent` for {:?}",
            value
        );
        let thumbnail = if !value.thumbnail.is_null() {
            Some(from_c_string(value.thumbnail))
        } else {
            None
        };
        let background = if !value.background.is_null() {
            Some(from_c_string(value.background))
        } else {
            None
        };
        let quality = if !value.quality.is_null() {
            Some(from_c_string(value.quality))
        } else {
            None
        };

        Self {
            url: from_c_string(value.url),
            title: from_c_string(value.title),
            thumbnail,
            background,
            quality,
        }
    }
}

/// A C-compatible enum representing loading errors.
#[repr(C)]
#[derive(Debug)]
pub enum LoadingErrorC {
    /// Error indicating a parsing failure with an associated error message.
    ParseError(*mut c_char),
    /// Error indicating a torrent-related failure with an associated error message.
    TorrentError(*mut c_char),
    /// Error indicating a media-related failure with an associated error message.
    MediaError(*mut c_char),
    /// Error indicating a timeout with an associated error message.
    TimeoutError(*mut c_char),
    InvalidData(*mut c_char),
    Cancelled,
}

/// Convert a `LoadingError` into a C-compatible `LoadingErrorC`.
impl From<LoadingError> for LoadingErrorC {
    fn from(value: LoadingError) -> Self {
        match value {
            LoadingError::ParseError(e) => LoadingErrorC::ParseError(into_c_string(e)),
            LoadingError::TorrentError(e) => {
                LoadingErrorC::TorrentError(into_c_string(e.to_string()))
            }
            LoadingError::MediaError(e) => LoadingErrorC::MediaError(into_c_string(e)),
            LoadingError::TimeoutError(e) => LoadingErrorC::TimeoutError(into_c_string(e)),
            LoadingError::InvalidData(e) => LoadingErrorC::InvalidData(into_c_string(e)),
            LoadingError::Cancelled => LoadingErrorC::Cancelled,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct LoadingProgressC {
    /// Progress indication between 0 and 1 that represents the progress of the download.
    pub progress: f32,
    /// The number of seeds available for the torrent.
    pub seeds: u32,
    /// The number of peers connected to the torrent.
    pub peers: u32,
    /// The total download transfer rate in bytes of payload only, not counting protocol chatter.
    pub download_speed: u32,
    /// The total upload transfer rate in bytes of payload only, not counting protocol chatter.
    pub upload_speed: u32,
    /// The total amount of data downloaded in bytes.
    pub downloaded: u64,
    /// The total size of the torrent in bytes.
    pub total_size: u64,
}

impl From<LoadingProgress> for LoadingProgressC {
    fn from(value: LoadingProgress) -> Self {
        Self {
            progress: value.progress,
            seeds: value.seeds as u32,
            peers: value.peers as u32,
            download_speed: value.download_speed as u32,
            upload_speed: value.upload_speed as u32,
            downloaded: value.downloaded,
            total_size: value.total_size as u64,
        }
    }
}

#[cfg(test)]
mod tests {
    use popcorn_fx_core::core::Handle;

    use super::*;

    #[test]
    fn test_loader_event_c_from() {
        let state = LoadingState::Downloading;
        let event = LoaderEvent::StateChanged(Handle::new(), state.clone());

        let result = LoaderEventC::from(event);

        if let LoaderEventC::StateChanged(_, result) = result {
            assert_eq!(state, result);
        } else {
            assert!(
                false,
                "expected LoaderEventC::StateChanged, but got {:?} instead",
                result
            )
        }
    }

    #[test]
    fn test_loading_started_event_c_from() {
        let url = "MyUrl";
        let title = "MyTitle";
        let thumb = "MyThumb";
        let event = LoadingStartedEvent {
            url: url.to_string(),
            title: title.to_string(),
            thumbnail: Some(thumb.to_string()),
            background: None,
            quality: None,
        };

        let result = LoadingStartedEventC::from(event);

        assert_eq!(url.to_string(), from_c_string(result.url));
        assert_eq!(title.to_string(), from_c_string(result.title));
        assert_eq!(thumb.to_string(), from_c_string(result.thumbnail));
    }

    #[test]
    fn test_loading_started_event_from() {
        let url = "MyUrl";
        let title = "MyTitle";
        let thumb = "MyThumb";
        let background = "MyBackground";
        let event = LoadingStartedEventC {
            url: into_c_string(url.to_string()),
            title: into_c_string(title.to_string()),
            thumbnail: into_c_string(thumb.to_string()),
            background: into_c_string(background.to_string()),
            quality: ptr::null_mut(),
        };
        let expected_result = LoadingStartedEvent {
            url: url.to_string(),
            title: title.to_string(),
            thumbnail: Some(thumb.to_string()),
            background: Some(background.to_string()),
            quality: None,
        };

        let result = LoadingStartedEvent::from(event);

        assert_eq!(expected_result, result)
    }
}
