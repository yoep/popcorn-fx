use crate::ffi::CArray;
use log::trace;
use popcorn_fx_core::core::torrents::{Error, TorrentInfo, TorrentState, TorrentStreamState};
use popcorn_fx_core::into_c_string;
use popcorn_fx_torrent::torrent;
use popcorn_fx_torrent::torrent::{
    TorrentError, TorrentEvent, TorrentHealth, TorrentHealthState, TorrentStats,
};
use std::os::raw::c_char;
use std::ptr;

/// Type alias for a callback that handles torrent stream events.
pub type TorrentEventCallback = extern "C" fn(TorrentEventC);

/// A C-compatible enum representing various errors related to torrents.
#[repr(C)]
#[derive(Debug, Clone)]
pub enum TorrentErrorC {
    /// Represents an error indicating an invalid URL.
    InvalidUrl(*mut c_char),
    /// Represents an error indicating a file not found.
    FileNotFound(*mut c_char),
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
    /// Represent an io error
    Io(*mut c_char),
}

impl From<Error> for TorrentErrorC {
    fn from(value: Error) -> Self {
        trace!("Converting TorrentErrorC from TorrentError {:?}", value);
        match value {
            Error::InvalidUrl(url) => TorrentErrorC::InvalidUrl(into_c_string(url)),
            Error::FileNotFound(file) => TorrentErrorC::FileNotFound(into_c_string(file)),
            Error::InvalidStreamState(state) => TorrentErrorC::InvalidStreamState(state),
            Error::InvalidHandle(handle) => TorrentErrorC::InvalidHandle(into_c_string(handle)),
            Error::TorrentResolvingFailed(error) => {
                TorrentErrorC::TorrentResolvingFailed(into_c_string(error))
            }
            Error::TorrentCollectionLoadingFailed(error) => {
                TorrentErrorC::TorrentCollectionLoadingFailed(into_c_string(error))
            }
            Error::TorrentError(error) => TorrentErrorC::Torrent(into_c_string(error)),
            Error::Io(e) => TorrentErrorC::Io(into_c_string(e)),
        }
    }
}

/// A C-compatible struct representing torrent information.
#[repr(C)]
#[derive(Debug)]
pub struct TorrentInfoC {
    /// The underlying torrent handle
    pub handle: i64,
    pub info_hash: *mut c_char,
    /// A pointer to a null-terminated C string representing the URI of the torrent.
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
            handle: value.handle.value(),
            info_hash: into_c_string(value.info_hash),
            uri: into_c_string(value.uri),
            name: into_c_string(value.name),
            directory_name,
            total_files: value.total_files,
            files: CArray::from(torrent_info_files),
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

impl From<torrent::File> for TorrentFileInfoC {
    fn from(value: torrent::File) -> Self {
        trace!("Converting torrent::File to TorrentFileInfoC");
        Self {
            filename: into_c_string(value.filename()),
            file_path: into_c_string(value.io_path.to_str().unwrap_or("")),
            file_size: value.length() as u64,
            file_index: value.index as u32,
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

impl From<TorrentStats> for DownloadStatusC {
    fn from(value: TorrentStats) -> Self {
        Self {
            progress: value.progress(),
            seeds: value.total_peers as u32,
            peers: value.total_peers as u32,
            download_speed: value.download_useful_rate as u32,
            upload_speed: value.upload_useful_rate as u32,
            downloaded: value.total_completed_size as u64,
            total_size: value.total_size as u64,
        }
    }
}

/// Represents a torrent event in C-compatible form.
#[repr(C)]
#[derive(Debug)]
pub enum TorrentEventC {
    /// Indicates a change in the state of the torrent.
    StateChanged(TorrentState),
    /// Indicates a change in the metric statics of the torrent.
    DownloadStatus(DownloadStatusC),
}

impl TryFrom<&TorrentEvent> for TorrentEventC {
    type Error = TorrentError;

    fn try_from(value: &TorrentEvent) -> Result<Self, Self::Error> {
        match value {
            TorrentEvent::StateChanged(e) => Ok(TorrentEventC::StateChanged(*e)),
            TorrentEvent::Stats(e) => Ok(TorrentEventC::DownloadStatus(DownloadStatusC::from(
                e.clone(),
            ))),
            _ => Err(TorrentError::TorrentParse(format!(
                "torrent event {:?} is not support",
                value
            ))),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct TorrentHealthC {
    /// The health state of the torrent.
    pub state: TorrentHealthState,
    /// The ratio of uploaded data to downloaded data for the torrent.
    pub ratio: f32,
    /// The number of seeders (peers with a complete copy of the torrent).
    pub seeds: u32,
    /// The number of leechers currently downloading the torrent.
    pub leechers: u32,
}

impl From<TorrentHealth> for TorrentHealthC {
    fn from(value: TorrentHealth) -> Self {
        Self {
            state: value.state,
            ratio: value.ratio,
            seeds: value.seeds,
            leechers: value.leechers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use popcorn_fx_core::{from_c_string, init_logger};
    use popcorn_fx_torrent::torrent::{TorrentFileInfo, TorrentHandle};
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_torrent_info_c_from() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let handle = TorrentHandle::new();
        let filename = "debian-12.4.0-amd64-DVD-1.iso";
        let info_hash = "EADAF0EFEA39406914414D359E0EA16416409BD7";
        let magnet_uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let total_files = 1;
        let info = TorrentInfo {
            handle,
            info_hash: info_hash.to_string(),
            uri: magnet_uri.to_string(),
            name: "debian-12.4.0-amd64-DVD-1.iso".to_string(),
            directory_name: None,
            total_files,
            files: vec![torrent::File {
                index: 0,
                torrent_path: PathBuf::from(filename),
                io_path: temp_dir.path().join(filename),
                offset: 0,
                info: TorrentFileInfo {
                    length: 2365000,
                    path: None,
                    path_utf8: None,
                    md5sum: None,
                    attr: None,
                    symlink_path: None,
                    sha1: None,
                },
                priority: Default::default(),
            }],
        };

        let result = TorrentInfoC::from(info);

        assert_eq!(
            handle.value(),
            result.handle,
            "expected the handle to match"
        );
        assert_eq!(
            info_hash.to_string(),
            from_c_string(result.info_hash),
            "expected the info hash to match"
        );
        assert_eq!(
            magnet_uri.to_string(),
            from_c_string(result.uri),
            "expected the uri to match"
        );
        assert_eq!(filename, from_c_string(result.name));
        assert_eq!(
            ptr::null_mut(),
            result.directory_name,
            "expected the directory to be null"
        );
        assert_eq!(total_files, result.total_files, "expect");
    }

    #[test]
    fn test_torrent_file_info_c_from() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let filename = "MyTor/MyFilename";
        let absolute_path = temp_dir.path().join(filename);
        let absolute_path_value = absolute_path.to_str().unwrap();
        let file_size = 452;
        let file_index = 1;
        let info = torrent::File {
            index: file_index,
            torrent_path: PathBuf::from(filename),
            io_path: absolute_path.clone(),
            offset: 0,
            info: TorrentFileInfo {
                length: file_size,
                path: None,
                path_utf8: None,
                md5sum: None,
                attr: None,
                symlink_path: None,
                sha1: None,
            },
            priority: Default::default(),
        };

        let result = TorrentFileInfoC::from(info);

        assert_eq!(
            "MyFilename".to_string(),
            from_c_string(result.filename),
            "expected the filename to match"
        );
        assert_eq!(
            absolute_path_value.to_string(),
            from_c_string(result.file_path),
            "expected the filepath to match"
        );
        assert_eq!(
            file_size, result.file_size,
            "expected the file length/size to match"
        );
        assert_eq!(file_index as u32, result.file_index);
    }

    #[test]
    fn test_torrent_error_c_from() {
        init_logger!();
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
}
