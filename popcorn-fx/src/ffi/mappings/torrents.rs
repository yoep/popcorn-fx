use std::os::raw::c_char;
use std::ptr;

use log::trace;

use popcorn_fx_core::core::torrents::{
    DownloadStatus, Error, TorrentFileInfo, TorrentInfo, TorrentState, TorrentStreamEvent,
    TorrentStreamState,
};
use popcorn_fx_core::{from_c_string, into_c_string};

use crate::ffi::CArray;

/// Type alias for a callback that verifies if the given byte is available.
pub type HasByteCallbackC = extern "C" fn(i32, *mut u64) -> bool;

/// Type alias for a callback that verifies if the given piece is available.
pub type HasPieceCallbackC = extern "C" fn(u32) -> bool;

/// Type alias for a callback that retrieves the total pieces of the torrent.
pub type TotalPiecesCallbackC = extern "C" fn() -> i32;

/// Type alias for a callback that prioritizes bytes.
pub type PrioritizeBytesCallbackC = extern "C" fn(i32, *mut u64);

/// Type alias for a callback that prioritizes pieces.
pub type PrioritizePiecesCallbackC = extern "C" fn(i32, *mut u32);

/// Type alias for a callback that updates the torrent mode to sequential.
pub type SequentialModeCallbackC = extern "C" fn();

/// Type alias for a callback that retrieves the torrent state.
pub type TorrentStateCallbackC = extern "C" fn() -> TorrentState;

/// Type alias for a callback that resolves torrent information and starts a download.
pub type ResolveTorrentCallback = extern "C" fn(
    file_info: TorrentFileInfoC,
    torrent_directory: *mut c_char,
    auto_start_download: bool,
) -> TorrentC;

/// Type alias for a callback that cancels a torrent download.
pub type CancelTorrentCallback = extern "C" fn(*mut c_char);

/// Type alias for a callback that handles torrent stream events.
pub type TorrentStreamEventCallback = extern "C" fn(TorrentStreamEventC);

/// A C-compatible enum representing various errors related to torrents.
#[repr(C)]
#[derive(Debug, Clone)]
pub enum TorrentErrorC {
    /// Represents an error indicating an invalid URL.
    InvalidUrl(*mut c_char),
    /// Represents an error indicating a file not found.
    FileNotFound(*mut c_char),
    /// Represents a generic file-related error.
    FileError(*mut c_char),
    /// Represents an error indicating an invalid stream state.
    InvalidStreamState(TorrentStreamState),
    /// Represents an error indicating an invalid handle.
    InvalidHandle(*mut c_char),
    /// Represents an error indicating failure during torrent resolving.
    TorrentResolvingFailed(*mut c_char),
    /// Represents an error indicating failure during torrent collection loading.
    TorrentCollectionLoadingFailed(*mut c_char),
    /// Represent a general torrent error failure
    Torrent(*mut c_char),
}

impl From<Error> for TorrentErrorC {
    fn from(value: Error) -> Self {
        trace!("Converting TorrentErrorC from TorrentError {:?}", value);
        match value {
            Error::InvalidUrl(url) => TorrentErrorC::InvalidUrl(into_c_string(url)),
            Error::FileNotFound(file) => TorrentErrorC::FileNotFound(into_c_string(file)),
            Error::FileError(error) => TorrentErrorC::FileError(into_c_string(error)),
            Error::InvalidStreamState(state) => TorrentErrorC::InvalidStreamState(state),
            Error::InvalidHandle(handle) => TorrentErrorC::InvalidHandle(into_c_string(handle)),
            Error::TorrentResolvingFailed(error) => {
                TorrentErrorC::TorrentResolvingFailed(into_c_string(error))
            }
            Error::TorrentCollectionLoadingFailed(error) => {
                TorrentErrorC::TorrentCollectionLoadingFailed(into_c_string(error))
            }
            Error::TorrentError(error) => TorrentErrorC::Torrent(into_c_string(error)),
        }
    }
}

impl From<TorrentErrorC> for Error {
    fn from(value: TorrentErrorC) -> Self {
        trace!("Converting TorrentError from TorrentErrorC {:?}", value);
        match value {
            TorrentErrorC::InvalidUrl(url) => Error::InvalidUrl(from_c_string(url)),
            TorrentErrorC::FileNotFound(file) => Error::FileNotFound(from_c_string(file)),
            TorrentErrorC::FileError(error) => Error::FileError(from_c_string(error)),
            TorrentErrorC::InvalidStreamState(state) => Error::InvalidStreamState(state),
            TorrentErrorC::InvalidHandle(handle) => Error::InvalidHandle(from_c_string(handle)),
            TorrentErrorC::TorrentResolvingFailed(error) => {
                Error::TorrentResolvingFailed(from_c_string(error))
            }
            TorrentErrorC::TorrentCollectionLoadingFailed(error) => {
                Error::TorrentCollectionLoadingFailed(from_c_string(error))
            }
            TorrentErrorC::Torrent(error) => Error::TorrentError(from_c_string(error)),
        }
    }
}

/// The C compatible abi struct for a [Torrent].
/// This currently uses callbacks as it's a wrapper around a torrent implementation provided through C.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct TorrentC {
    pub handle: *mut c_char,
    /// The filepath to the torrent file
    pub filepath: *mut c_char,
    pub has_byte_callback: HasByteCallbackC,
    pub has_piece_callback: HasPieceCallbackC,
    pub total_pieces: TotalPiecesCallbackC,
    pub prioritize_bytes: PrioritizeBytesCallbackC,
    pub prioritize_pieces: PrioritizePiecesCallbackC,
    pub sequential_mode: SequentialModeCallbackC,
    pub torrent_state: TorrentStateCallbackC,
}

/// A C-compatible struct representing torrent information.
#[repr(C)]
#[derive(Debug)]
pub struct TorrentInfoC {
    pub info_hash: *mut c_char,
    pub uri: *mut c_char,
    /// A pointer to a null-terminated C string representing the name of the torrent.
    pub name: *mut c_char,
    /// A pointer to a null-terminated C string representing the directory name of the torrent.
    pub directory_name: *mut c_char,
    /// The total number of files in the torrent.
    pub total_files: u32,
    /// A set of `TorrentFileInfoC` structs representing individual files within the torrent.
    pub files: CArray<TorrentFileInfoC>,
}

impl From<TorrentInfo> for TorrentInfoC {
    fn from(value: TorrentInfo) -> Self {
        trace!("Converting TorrentInfo to TorrentInfoC for {:?}", value);
        let directory_name = if let Some(e) = value.directory_name {
            into_c_string(e)
        } else {
            ptr::null_mut()
        };
        let torrent_info_files: Vec<TorrentFileInfoC> = value
            .files
            .into_iter()
            .map(|e| TorrentFileInfoC::from(e))
            .collect();

        Self {
            info_hash: into_c_string(value.info_hash),
            uri: into_c_string(value.uri),
            name: into_c_string(value.name),
            directory_name,
            total_files: value.total_files as u32,
            files: CArray::from(torrent_info_files),
        }
    }
}

impl From<TorrentInfoC> for TorrentInfo {
    fn from(value: TorrentInfoC) -> Self {
        trace!("Converting TorrentInfoC to TorrentInfo for {:?}", value);
        let files = Vec::<TorrentFileInfoC>::from(value.files)
            .into_iter()
            .map(|e| TorrentFileInfo::from(e))
            .collect();
        let directory_name = if !value.directory_name.is_null() {
            Some(from_c_string(value.directory_name))
        } else {
            None
        };

        Self {
            info_hash: from_c_string(value.info_hash),
            uri: from_c_string(value.uri),
            name: from_c_string(value.name),
            directory_name,
            total_files: value.total_files,
            files,
        }
    }
}

/// A C-compatible struct representing torrent file information.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct TorrentFileInfoC {
    /// A pointer to a null-terminated C string representing the filename.
    pub filename: *mut c_char,
    /// A pointer to a null-terminated C string representing the file path.
    pub file_path: *mut c_char,
    /// The size of the file in bytes.
    pub file_size: u64,
    /// The index of the file.
    pub file_index: u32,
}

impl From<TorrentFileInfoC> for TorrentFileInfo {
    fn from(value: TorrentFileInfoC) -> Self {
        trace!("Converting TorrentFileInfoC to TorrentFileInfo");
        Self {
            filename: from_c_string(value.filename),
            file_path: from_c_string(value.file_path),
            file_size: value.file_size,
            file_index: value.file_index as usize,
        }
    }
}

impl From<TorrentFileInfo> for TorrentFileInfoC {
    fn from(value: TorrentFileInfo) -> Self {
        trace!("Converting TorrentFileInfo to TorrentFileInfoC");
        Self {
            filename: into_c_string(value.filename),
            file_path: into_c_string(value.file_path),
            file_size: value.file_size,
            file_index: value.file_index as u32,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct DownloadStatusC {
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

impl From<DownloadStatusC> for DownloadStatus {
    fn from(value: DownloadStatusC) -> Self {
        Self {
            progress: value.progress,
            seeds: value.seeds as usize,
            peers: value.peers as usize,
            download_speed: value.download_speed as u64,
            upload_speed: value.upload_speed as u64,
            downloaded: value.downloaded,
            total_size: value.total_size as usize,
        }
    }
}

impl From<DownloadStatus> for DownloadStatusC {
    fn from(value: DownloadStatus) -> Self {
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

/// Represents a torrent stream event in C-compatible form.
#[repr(C)]
#[derive(Debug)]
pub enum TorrentStreamEventC {
    /// Indicates a change in the state of the torrent stream.
    StateChanged(TorrentStreamState),
    /// Indicates a change in the download status of the torrent stream.
    DownloadStatus(DownloadStatusC),
}

impl From<TorrentStreamEvent> for TorrentStreamEventC {
    fn from(value: TorrentStreamEvent) -> Self {
        match value {
            TorrentStreamEvent::StateChanged(e) => TorrentStreamEventC::StateChanged(e),
            TorrentStreamEvent::DownloadStatus(e) => {
                TorrentStreamEventC::DownloadStatus(DownloadStatusC::from(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use popcorn_fx_core::into_c_string;
    use popcorn_fx_core::testing::init_logger;

    use super::*;

    #[test]
    fn test_from_torrent_info_c() {
        let handle = "MyHandle";
        let uri = "magnet:?FooBarUri";
        let name = "FooBar54";
        let total_files = 15;
        let info = TorrentInfoC {
            info_hash: into_c_string(handle.to_string()),
            uri: into_c_string(uri.to_string()),
            name: into_c_string(name.to_string()),
            directory_name: ptr::null_mut(),
            total_files,
            files: CArray::from(Vec::<TorrentFileInfoC>::new()),
        };
        let expected_result = TorrentInfo {
            info_hash: handle.to_string(),
            uri: uri.to_string(),
            name: name.to_string(),
            directory_name: None,
            total_files,
            files: vec![],
        };

        let result = TorrentInfo::from(info);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_from_torrent_file_info_c() {
        let filename = "MyTFile";
        let file_path = "/tmp/path/file";
        let file_size = 87500;
        let file_index = 1usize;
        let file_info = TorrentFileInfoC {
            filename: into_c_string(filename.to_string()),
            file_path: into_c_string(file_path.to_string()),
            file_size,
            file_index: file_index as u32,
        };
        let expected_result = TorrentFileInfo {
            filename: filename.to_string(),
            file_path: file_path.to_string(),
            file_size,
            file_index,
        };

        let result = TorrentFileInfo::from(file_info);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_torrent_file_info_c_from() {
        let filename = "MyFilename";
        let file_path = "TorDir";
        let file_size = 452;
        let file_index = 0;
        let info = TorrentFileInfo {
            filename: filename.to_string(),
            file_path: file_path.to_string(),
            file_size,
            file_index,
        };

        let result = TorrentFileInfoC::from(info);

        assert_eq!(filename.to_string(), from_c_string(result.filename));
        assert_eq!(file_path.to_string(), from_c_string(result.file_path));
        assert_eq!(file_size, result.file_size);
        assert_eq!(file_index as u32, result.file_index);
    }

    #[test]
    fn test_download_status_c_from() {
        let status = DownloadStatus {
            progress: 0.6,
            seeds: 10,
            peers: 12,
            download_speed: 20,
            upload_speed: 16,
            downloaded: 230,
            total_size: 158965,
        };
        let expected_result = DownloadStatusC {
            progress: 0.6,
            seeds: 10,
            peers: 12,
            download_speed: 20,
            upload_speed: 16,
            downloaded: 230,
            total_size: 158965,
        };

        let result = DownloadStatusC::from(status);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_download_status_from() {
        let status_c = DownloadStatusC {
            progress: 0.6,
            seeds: 10,
            peers: 12,
            download_speed: 20,
            upload_speed: 16,
            downloaded: 230,
            total_size: 158965,
        };
        let expected_result = DownloadStatus {
            progress: 0.6,
            seeds: 10,
            peers: 12,
            download_speed: 20,
            upload_speed: 16,
            downloaded: 230,
            total_size: 158965,
        };

        let result = DownloadStatus::from(status_c);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_torrent_stream_event_c_from() {
        let state = TorrentStreamState::Streaming;
        let event = TorrentStreamEvent::StateChanged(state.clone());

        let result = TorrentStreamEventC::from(event);

        if let TorrentStreamEventC::StateChanged(result) = result {
            assert_eq!(state, result)
        } else {
            assert!(
                false,
                "expected TorrentStreamEventC::StateChanged, but got {:?} instead",
                result
            )
        }

        let status = DownloadStatus {
            progress: 0.35,
            seeds: 2,
            peers: 5,
            download_speed: 13,
            upload_speed: 16,
            downloaded: 8200,
            total_size: 20000,
        };
        let expected_result = DownloadStatusC {
            progress: 0.35,
            seeds: 2,
            peers: 5,
            download_speed: 13,
            upload_speed: 16,
            downloaded: 8200,
            total_size: 20000,
        };
        let event = TorrentStreamEvent::DownloadStatus(status);

        let result = TorrentStreamEventC::from(event);

        if let TorrentStreamEventC::DownloadStatus(result) = result {
            assert_eq!(expected_result, result)
        } else {
            assert!(
                false,
                "expected TorrentStreamEventC::DownloadStatus, but got {:?} instead",
                result
            )
        }
    }

    #[test]
    fn test_torrent_error_c_from() {
        init_logger();
        let filename = "my-filename";
        let error = Error::FileNotFound(filename.to_string());

        let error_c = TorrentErrorC::from(error);

        if let TorrentErrorC::FileNotFound(result) = error_c {
            assert_eq!(filename.to_string(), from_c_string(result))
        } else {
            assert!(
                false,
                "expected TorrentErrorC::FileNotFound, but got {:?} instead",
                error_c
            )
        }
    }

    #[test]
    fn test_torrent_error_from() {
        init_logger();
        let filename = "my-filename";
        let resolve_failed_message = "failed to resolve torrent X";

        let error_c = TorrentErrorC::FileNotFound(into_c_string(filename));
        let error = Error::from(error_c);
        assert_eq!(Error::FileNotFound(filename.to_string()), error);

        let error_c = TorrentErrorC::TorrentResolvingFailed(into_c_string(resolve_failed_message));
        let error = Error::from(error_c);
        assert_eq!(
            Error::TorrentResolvingFailed(resolve_failed_message.to_string()),
            error
        );
    }
}
