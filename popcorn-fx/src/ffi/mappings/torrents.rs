use std::os::raw::c_char;

use log::trace;

use popcorn_fx_core::{from_c_string, into_c_string};
use popcorn_fx_core::core::torrents::{TorrentFileInfo, TorrentInfo};
use popcorn_fx_torrent_stream::TorrentC;

use crate::ffi::CSet;

/// Type definition for a callback that resolves torrent information.
pub type ResolveTorrentInfoCallback = extern "C" fn(url: *const c_char) -> TorrentInfoC;

/// Type definition for a callback that resolves torrent information and starts a download.
pub type ResolveTorrentCallback = extern "C" fn(file_info: TorrentFileInfoC, torrent_directory: *const c_char, auto_start_download: bool) -> TorrentC;

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
    pub files: CSet<TorrentFileInfoC>,
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
            files: CSet::from(Vec::<TorrentFileInfoC>::new()),
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
}