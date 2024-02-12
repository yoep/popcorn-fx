use std::os::raw::c_char;
use std::ptr;

use log::trace;

use popcorn_fx_core::{from_c_string, into_c_string, to_c_vec};
use popcorn_fx_core::core::torrents::{DownloadStatus, TorrentFileInfo, TorrentInfo, TorrentState, TorrentStreamEvent, TorrentStreamState, TorrentWrapper};

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

/// Type alias for a callback that resolves torrent information.
pub type ResolveTorrentInfoCallback = extern "C" fn(url: *const c_char) -> TorrentInfoC;

/// Type alias for a callback that resolves torrent information and starts a download.
pub type ResolveTorrentCallback = extern "C" fn(file_info: TorrentFileInfoC, torrent_directory: *const c_char, auto_start_download: bool) -> TorrentC;

/// Type alias for a callback that cancels a torrent download.
pub type CancelTorrentCallback = extern "C" fn(*const c_char);

/// Type alias for a callback that handles torrent stream events.
pub type TorrentStreamEventCallback = extern "C" fn(TorrentStreamEventC);

/// The C compatible abi struct for a [Torrent].
/// This currently uses callbacks as it's a wrapper around a torrent implementation provided through C.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct TorrentC {
    pub handle: *const c_char,
    /// The filepath to the torrent file
    pub filepath: *const c_char,
    pub has_byte_callback: HasByteCallbackC,
    pub has_piece_callback: HasPieceCallbackC,
    pub total_pieces: TotalPiecesCallbackC,
    pub prioritize_bytes: PrioritizeBytesCallbackC,
    pub prioritize_pieces: PrioritizePiecesCallbackC,
    pub sequential_mode: SequentialModeCallbackC,
    pub torrent_state: TorrentStateCallbackC,
}

impl From<TorrentC> for TorrentWrapper {
    fn from(value: TorrentC) -> Self {
        trace!("Converting TorrentWrapper from TorrentC {:?}", value);
        Self::new(
            from_c_string(value.handle),
            from_c_string(value.filepath),
            Box::new(move |bytes| -> bool {
                let (bytes, len) = to_c_vec(bytes.to_vec());
                (value.has_byte_callback)(len, bytes)
            }),
            Box::new(move |piece| {
                (value.has_piece_callback)(piece)
            }),
            Box::new(move || (value.total_pieces)()),
            Box::new(move |bytes| {
                let (bytes, len) = to_c_vec(bytes.to_vec());
                (value.prioritize_bytes)(len, bytes)
            }),
            Box::new(move |pieces| {
                let (pieces, len) = to_c_vec(pieces.to_vec());
                (value.prioritize_pieces)(len, pieces)
            }),
            Box::new(move || (value.sequential_mode)()),
            Box::new(move || (value.torrent_state)()),
        )
    }
}

/// A C-compatible struct representing torrent information.
#[repr(C)]
#[derive(Debug)]
pub struct TorrentInfoC {
    /// A pointer to a null-terminated C string representing the name of the torrent.
    pub name: *const c_char,
    /// A pointer to a null-terminated C string representing the directory name of the torrent.
    pub directory_name: *const c_char,
    /// The total number of files in the torrent.
    pub total_files: i32,
    /// A set of `TorrentFileInfoC` structs representing individual files within the torrent.
    pub files: CArray<TorrentFileInfoC>,
}

impl From<TorrentInfo> for TorrentInfoC {
    fn from(value: TorrentInfo) -> Self {
        let directory_name = if let Some(e) = value.directory_name {
            into_c_string(e)
        } else {
            ptr::null()
        };
        let torrent_info_files: Vec<TorrentFileInfoC> = value.files.into_iter()
            .map(|e| TorrentFileInfoC::from(e))
            .collect();

        Self {
            name: into_c_string(value.name),
            directory_name,
            total_files: value.total_files,
            files: CArray::from(torrent_info_files),
        }
    }
}

impl From<TorrentInfoC> for TorrentInfo {
    fn from(value: TorrentInfoC) -> Self {
        trace!("Converting TorrentInfoC to TorrentInfo");
        let files = Vec::<TorrentFileInfoC>::from(value.files).into_iter()
            .map(|e| TorrentFileInfo::from(e))
            .collect();
        let directory_name = if !value.directory_name.is_null() {
            Some(from_c_string(value.directory_name))
        } else {
            None
        };

        Self {
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
    pub filename: *const c_char,
    /// A pointer to a null-terminated C string representing the file path.
    pub file_path: *const c_char,
    /// The size of the file in bytes.
    pub file_size: i64,
    /// The index of the file.
    pub file_index: i32,
}

impl From<TorrentFileInfoC> for TorrentFileInfo {
    fn from(value: TorrentFileInfoC) -> Self {
        trace!("Converting TorrentFileInfoC to TorrentFileInfo");
        Self {
            filename: from_c_string(value.filename),
            file_path: from_c_string(value.file_path),
            file_size: value.file_size,
            file_index: value.file_index,
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
            file_index: value.file_index,
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
            seeds: value.seeds,
            peers: value.peers,
            download_speed: value.download_speed,
            upload_speed: value.upload_speed,
            downloaded: value.downloaded,
            total_size: value.total_size,
        }
    }
}

impl From<DownloadStatus> for DownloadStatusC {
    fn from(value: DownloadStatus) -> Self {
        Self {
            progress: value.progress,
            seeds: value.seeds,
            peers: value.peers,
            download_speed: value.download_speed,
            upload_speed: value.upload_speed,
            downloaded: value.downloaded,
            total_size: value.total_size,
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
            TorrentStreamEvent::DownloadStatus(e) => TorrentStreamEventC::DownloadStatus(DownloadStatusC::from(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use popcorn_fx_core::into_c_string;

    use super::*;

    #[test]
    fn test_from_torrent_info_c() {
        let name = "FooBar54";
        let total_files = 15;
        let info = TorrentInfoC {
            name: into_c_string(name.to_string()),
            directory_name: ptr::null(),
            total_files,
            files: CArray::from(Vec::<TorrentFileInfoC>::new()),
        };
        let expected_result = TorrentInfo {
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
        let file_index = 1;
        let file_info = TorrentFileInfoC {
            filename: into_c_string(filename.to_string()),
            file_path: into_c_string(file_path.to_string()),
            file_size,
            file_index,
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
        assert_eq!(file_index, result.file_index);
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
            assert!(false, "expected TorrentStreamEventC::StateChanged, but got {:?} instead", result)
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
            assert!(false, "expected TorrentStreamEventC::DownloadStatus, but got {:?} instead", result)
        }
    }
}