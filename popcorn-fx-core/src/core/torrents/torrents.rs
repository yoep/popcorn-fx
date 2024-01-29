use std::fmt::{Debug, Display};
#[cfg(any(test, feature = "testing"))]
use std::fmt::Formatter;
use std::path::PathBuf;

use derive_more::Display;
use downcast_rs::{DowncastSync, impl_downcast};
use log::{debug, trace};
#[cfg(any(test, feature = "testing"))]
use mockall::automock;

use crate::core::CoreCallback;

const TORRENT_STATES: [TorrentState; 7] = [
    TorrentState::Creating,
    TorrentState::Ready,
    TorrentState::Starting,
    TorrentState::Downloading,
    TorrentState::Paused,
    TorrentState::Completed,
    TorrentState::Error,
];

/// The callback type for all torrent events.
pub type TorrentCallback = CoreCallback<TorrentEvent>;

/// The torrent event which occurred for the torrent.
#[derive(Debug, Clone, Display)]
pub enum TorrentEvent {
    /// The new state of the torrent
    #[display(fmt = "Torrent state changed to {}", _0)]
    StateChanged(TorrentState),
    /// The piece that has finished downloading
    #[display(fmt = "Torrent piece {} finished downloading", _0)]
    PieceFinished(u32),
}

/// The state of a [Torrent] which is represented as a [i32].
/// This state is abi compatible to be used over [std::ffi].
#[repr(i32)]
#[derive(Debug, Clone, Display, PartialEq)]
pub enum TorrentState {
    /// The initial phase of the torrent in which it's still being created.
    /// This is the state where the metadata of the torrent is retrieved.
    Creating = 0,
    /// The torrent is ready to be downloaded (metadata is available).
    Ready = 1,
    /// The download of the torrent is starting.
    Starting = 2,
    /// The torrent is being downloaded.
    Downloading = 3,
    /// The torrent download has been paused.
    Paused = 4,
    /// The torrent download has completed.
    Completed = 5,
    /// The torrent encountered an error and cannot be downloaded.
    Error = -1,
}

impl From<i32> for TorrentState {
    fn from(value: i32) -> Self {
        for state in TORRENT_STATES {
            let ordinal = state.clone() as i32;

            if ordinal == value {
                return state;
            }
        }

        panic!("Ordinal {} is out of range for TorrentState", value)
    }
}

/// The torrent describes the meta-info of a shared file that can be queried over the network.
/// It allows for action such as downloading the shared file to the local system.
#[cfg_attr(any(test, feature = "testing"), automock)]
pub trait Torrent: Display + Debug + DowncastSync {
    /// The unique handle of this [Torrent].
    fn handle(&self) -> &str;

    /// The absolute path to this torrent file.
    fn file(&self) -> PathBuf;

    /// Verify if the given bytes are available for this [Torrent].
    ///
    /// It returns true when the bytes are available, else false.
    fn has_bytes(&self, bytes: &[u64]) -> bool;

    /// Verify if the given piece is available.
    ///
    /// It returns true when the piece is present, else false.
    fn has_piece(&self, piece: u32) -> bool;

    /// Prioritize the given bytes to be downloaded.
    fn prioritize_bytes(&self, bytes: &[u64]);

    /// Prioritize the given piece indexes.
    fn prioritize_pieces(&self, pieces: &[u32]);

    /// The total number of pieces that are available for download.
    fn total_pieces(&self) -> i32;

    /// Update the download mode of the torrent to sequential.
    fn sequential_mode(&self);

    /// Retrieve the current state of the torrent.
    /// It returns an owned instance of the state.
    fn state(&self) -> TorrentState;

    /// Register a new callback for the [TorrentEvent]'s.
    /// The callback will be triggered when a new event occurs within the torrent.
    fn subscribe(&self, callback: TorrentCallback) -> i64;
}
impl_downcast!(sync Torrent);

#[cfg(any(test, feature = "testing"))]
impl Display for MockTorrent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockTorrent")
    }
}

/// The torrent information
#[derive(Debug, Clone, PartialEq)]
pub struct TorrentInfo {
    /// The name of the torrent
    pub name: String,
    /// The torrent directory name in which the media files might reside.
    pub directory_name: Option<String>,
    /// The total number of files available in the torrent
    pub total_files: i32,
    /// The available files
    pub files: Vec<TorrentFileInfo>,
}

impl TorrentInfo {
    pub fn by_filename(&self, filename: &str) -> Option<TorrentFileInfo> {
        self.files.iter()
            .find(|e| {
                let filepath = Self::simplified_filepath(e.file_path.as_str());
                let expected_filepath = Self::simplified_filepath(filename);

                trace!("Checking if filepath {} matches filename {} without torrent directory", filepath, expected_filepath);
                filepath.eq_ignore_ascii_case(expected_filepath.as_str())
            })
            .or_else(|| {
                debug!("Torrent file couldn't be found for {} without torrent directory, searching with torrent directory {:?}", filename, self.directory_name);
                let torrent_directory_name = self.directory_name.clone();
                let filepath = PathBuf::from(torrent_directory_name.unwrap_or("".to_string()))
                    .join(filename)
                    .to_str()
                    .map(Self::simplified_filepath)
                    .expect("expected a valid filepath to have been created");

                self.files.iter()
                    .find(|e| Self::simplified_filepath(e.file_path.as_str()).eq_ignore_ascii_case(filepath.as_str()))
            })
            .cloned()
    }

    pub fn largest_file(&self) -> Option<TorrentFileInfo> {
        let mut largest_file_index: Option<usize> = None;
        let mut largest_file_size = 0i64;
        let mut index: usize = 0;

        for file in self.files.iter() {
            if file.file_size > largest_file_size {
                largest_file_index = Some(index);
                largest_file_size = file.file_size;
            }

            index += 1;
        }

        largest_file_index
            .and_then(|e| self.files.get(e))
            .cloned()
    }

    fn simplified_filepath(file_path: &str) -> String {
        file_path
            .replace("/", "")
            .replace("\\", "")
            .trim()
            .to_string()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TorrentFileInfo {
    pub filename: String,
    pub file_path: String,
    pub file_size: i64,
    pub file_index: i32,
}

#[cfg(test)]
mod test {
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_torrent_state_from() {
        let error = TorrentState::from(-1);
        let creating = TorrentState::from(0);
        let ready = TorrentState::from(1);
        let starting = TorrentState::from(2);
        let downloading = TorrentState::from(3);
        let paused = TorrentState::from(4);

        assert_eq!(TorrentState::Error, error);
        assert_eq!(TorrentState::Creating, creating);
        assert_eq!(TorrentState::Ready, ready);
        assert_eq!(TorrentState::Starting, starting);
        assert_eq!(TorrentState::Downloading, downloading);
        assert_eq!(TorrentState::Paused, paused);
    }

    #[test]
    fn test_torrent_info_by_filename_match_without_torrent_directory() {
        init_logger();
        let filename = "lorem.mp4";
        let expected_result = TorrentFileInfo {
            filename: "".to_string(),
            file_path: "/lorem.mp4".to_string(),
            file_size: 1500,
            file_index: 0,
        };
        let info = TorrentInfo {
            name: "".to_string(),
            directory_name: Some("torrentDirectory".to_string()),
            total_files: 0,
            files: vec![
                expected_result.clone(),
                TorrentFileInfo {
                    filename: "".to_string(),
                    file_path: "/ipsum.mp4".to_string(),
                    file_size: 18000,
                    file_index: 1,
                },
            ],
        };

        let result = info.by_filename(filename);

        assert_eq!(Some(expected_result), result);
    }

    #[test]
    fn test_torrent_info_by_filename_match_with_torrent_directory() {
        init_logger();
        let filename = "ipsum.mp4";
        let expected_result = TorrentFileInfo {
            filename: "".to_string(),
            file_path: "torrentDirectory/ipsum.mp4".to_string(),
            file_size: 23000,
            file_index: 0,
        };
        let info = TorrentInfo {
            name: "".to_string(),
            directory_name: Some("torrentDirectory".to_string()),
            total_files: 0,
            files: vec![
                TorrentFileInfo {
                    filename: "".to_string(),
                    file_path: "torrentDirectory/lorem.mp4".to_string(),
                    file_size: 18000,
                    file_index: 1,
                },
                expected_result.clone(),
            ],
        };

        let result = info.by_filename(filename);

        assert_eq!(Some(expected_result), result);
    }

    #[test]
    fn test_torrent_info_largest_file() {
        let largest_file = TorrentFileInfo {
            filename: "file2".to_string(),
            file_path: "".to_string(),
            file_size: 230,
            file_index: 0,
        };
        let info = TorrentInfo {
            name: "".to_string(),
            directory_name: None,
            total_files: 0,
            files: vec![
                TorrentFileInfo {
                    filename: "file1".to_string(),
                    file_path: "".to_string(),
                    file_size: 150,
                    file_index: 0,
                },
                largest_file.clone(),
                TorrentFileInfo {
                    filename: "file3".to_string(),
                    file_path: "".to_string(),
                    file_size: 220,
                    file_index: 0,
                },
            ],
        };

        let result = info.largest_file();

        assert_eq!(Some(largest_file), result);
    }
}