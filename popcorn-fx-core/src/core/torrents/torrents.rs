use crate::core::{Callbacks, CoreCallback, Handle};
use async_trait::async_trait;
use derive_more::Display;
use log::{debug, trace};

#[cfg(any(test, feature = "testing"))]
pub use mock::*;
use std::fmt::{Debug, Display};
use std::path::PathBuf;

/// A unique handle identifier of a [Torrent].
pub type TorrentHandle = Handle;

/// The torrent event specific callbacks.
pub type TorrentEventCallback = CoreCallback<TorrentEvent>;

const TORRENT_STATES: [TorrentState; 7] = [
    TorrentState::Initializing,
    TorrentState::Ready,
    TorrentState::Starting,
    TorrentState::Downloading,
    TorrentState::Paused,
    TorrentState::Completed,
    TorrentState::Error,
];

/// The callback type for all torrent events.
pub type TorrentCallback = CoreCallback<TorrentEvent>;

/// Represents events that can occur for a torrent.
#[derive(Debug, Clone, Display)]
pub enum TorrentEvent {
    /// Indicates a change in the state of the torrent.
    #[display(fmt = "Torrent state changed to {}", _0)]
    StateChanged(TorrentState),
    /// Indicates that a piece of the torrent has finished downloading.
    #[display(fmt = "Torrent piece {} finished downloading", _0)]
    PieceFinished(u32),
    /// Indicates a change in the download status of the torrent.
    #[display(fmt = "Torrent download status changed, {}", _0)]
    DownloadStatus(DownloadStatus),
}

/// The state of a [Torrent] which is represented as a [i32].
/// This state is abi compatible to be used over [std::ffi].
#[repr(u8)]
#[derive(Debug, Clone, Display, PartialEq)]
pub enum TorrentState {
    /// The initial phase of the torrent in which it's still being created.
    /// This is the state where the metadata of the torrent is retrieved.
    Initializing = 0,
    /// The torrent is validating existing files
    CheckingFiles = 1,
    /// The torrent is retrieving metadata from peers
    RetrievingMetadata = 2,
    /// The torrent is ready to be downloaded (metadata is available).
    Ready = 3,
    /// The download of the torrent is starting.
    Starting = 4,
    /// The torrent is being downloaded.
    Downloading = 5,
    /// The torrent download has been paused.
    Paused = 6,
    /// The torrent download has completed.
    Completed = 7,
    /// The torrent encountered an error and cannot be downloaded.
    Error = 8,
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

/// Represents the download status of a torrent.
#[derive(Debug, Display, Clone, PartialOrd, PartialEq)]
#[display(
    fmt = "progress: {}, seeds: {}, peers: {}, download_speed: {}",
    progress,
    seeds,
    peers,
    download_speed
)]
pub struct DownloadStatus {
    /// Progress indication between 0 and 1 that represents the progress of the download.
    pub progress: f32,
    /// The number of seeds available for the torrent.
    pub seeds: usize,
    /// The number of peers connected to the torrent.
    pub peers: usize,
    /// The total download transfer rate in bytes of payload only, not counting protocol chatter.
    pub download_speed: u64,
    /// The total upload transfer rate in bytes of payload only, not counting protocol chatter.
    pub upload_speed: u64,
    /// The total amount of data downloaded in bytes.
    pub downloaded: u64,
    /// The total size of the torrent in bytes.
    pub total_size: usize,
}

/// The torrent describes the meta-info of a shared file that can be queried over the network.
/// It allows for action such as downloading the shared file to the local system.
#[async_trait]
pub trait Torrent: Debug + Display + Callbacks<TorrentEvent> + Send + Sync {
    /// Get the unique identifier handle of the torrent.
    fn handle(&self) -> TorrentHandle;

    /// The absolute path to this torrent file.
    async fn file(&self) -> PathBuf;

    /// Check if the given bytes are available within the torrent.
    /// This will check if the underlying pieces that contain the given byte range are downloaded, validated and written to storage.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The byte range to check.
    ///
    /// # Returns
    ///
    /// Returns true when all bytes are downloaded, validated and written to storage, else false.
    async fn has_bytes(&self, bytes: &std::ops::Range<usize>) -> bool;

    /// Check if the given piece is downloaded, validated and written to storage.
    ///
    /// It returns true when the piece is present, else false.
    async fn has_piece(&self, piece: usize) -> bool;

    /// Prioritize the given bytes to be downloaded.
    async fn prioritize_bytes(&self, bytes: &std::ops::Range<usize>);

    /// Prioritize the given piece indexes.
    async fn prioritize_pieces(&self, pieces: &[u32]);

    /// Get the total number of pieces in the torrent.
    /// It might return [None] when the metadata is still being retrieved.
    async fn total_pieces(&self) -> usize;

    /// Update the download mode of the torrent to sequential.
    async fn sequential_mode(&self);

    /// Retrieve the current state of the torrent.
    /// It returns an owned instance of the state.
    async fn state(&self) -> TorrentState;
}

/// The torrent information
#[derive(Debug, Display, Clone, PartialEq)]
#[display(
    fmt = "info_hash: {}, uri: {}, name: {}, directory_name: {:?}, total_files: {}",
    info_hash,
    uri,
    name,
    directory_name,
    total_files
)]
pub struct TorrentInfo {
    /// The info hash of the torrent.
    pub info_hash: String,
    /// The magnet uri of the torrent
    pub uri: String,
    /// The name of the torrent
    pub name: String,
    /// The torrent directory name in which the media files might reside.
    pub directory_name: Option<String>,
    /// The total number of files available in the torrent
    pub total_files: u32,
    /// The available files
    pub files: Vec<TorrentFileInfo>,
}

impl TorrentInfo {
    pub fn by_filename(&self, filename: &str) -> Option<TorrentFileInfo> {
        trace!(
            "Searching for torrent file {} within {:?}",
            filename,
            self.files
        );
        self.by_filename_without_directory(filename)
            .or_else(|| {
                debug!("Torrent file couldn't be found for {} without torrent directory, searching with torrent directory {:?}", filename, self.directory_name);
                self.by_filename_with_directory(filename)
            })
            .cloned()
    }

    pub fn largest_file(&self) -> Option<TorrentFileInfo> {
        let mut largest_file_index: Option<usize> = None;
        let mut largest_file_size = 0u64;
        let mut index: usize = 0;

        for file in self.files.iter() {
            if file.file_size > largest_file_size {
                largest_file_index = Some(index);
                largest_file_size = file.file_size;
            }

            index += 1;
        }

        largest_file_index.and_then(|e| self.files.get(e)).cloned()
    }

    fn by_filename_without_directory(&self, filename: &str) -> Option<&TorrentFileInfo> {
        debug!(
            "Searching for torrent file {} without torrent directory",
            filename
        );
        self.files.iter().find(|e| {
            let filepath = Self::simplified_filepath(e.file_path.as_str());
            let expected_filepath = Self::simplified_filepath(filename);

            trace!(
                "Checking if filepath \"{}\" matches filename \"{}\" without torrent directory",
                filepath,
                expected_filepath
            );
            filepath.eq_ignore_ascii_case(expected_filepath.as_str())
        })
    }

    fn by_filename_with_directory(&self, filename: &str) -> Option<&TorrentFileInfo> {
        debug!(
            "Searching for torrent file {} with torrent directory {:?}",
            filename, self.directory_name
        );
        let torrent_directory_name = self.directory_name.clone();
        let expected_filepath = PathBuf::from(torrent_directory_name.unwrap_or("".to_string()))
            .join(filename)
            .to_str()
            .map(Self::simplified_filepath)
            .expect("expected a valid filepath to have been created");

        self.files.iter().find(|e| {
            let filepath = Self::simplified_filepath(e.file_path.as_str());

            trace!(
                "Checking if filepath \"{}\" matches filename \"{}\" with torrent directory",
                filepath,
                expected_filepath
            );
            filepath.eq_ignore_ascii_case(expected_filepath.as_str())
        })
    }

    fn simplified_filepath(file_path: &str) -> String {
        file_path
            .replace("/", "")
            .replace("\\", "")
            .trim()
            .to_string()
    }
}

/// Represents information about a file within a torrent.
#[derive(Debug, Display, Clone, PartialEq)]
#[display(
    fmt = "filename: {}, path: {}, size: {}, index: {}",
    filename,
    file_path,
    file_size,
    file_index
)]
pub struct TorrentFileInfo {
    /// The name of the file.
    pub filename: String,
    /// The full path to the file within the torrent.
    pub file_path: String,
    /// The size of the file in bytes.
    pub file_size: u64,
    /// The index of the file within the torrent.
    pub file_index: usize,
}

impl TorrentFileInfo {
    pub fn filename(&self) -> &str {
        self.filename.as_str()
    }

    pub fn file_path(&self) -> &str {
        self.file_path.as_str()
    }
}

/// Represents the different states of torrent health.
#[repr(u32)]
#[derive(Debug, Default, Display, Clone, PartialEq)]
pub enum TorrentHealthState {
    /// Unknown health state, indicating that the health of the torrent could not be determined.
    #[default]
    #[display(fmt = "unknown")]
    Unknown,
    /// Bad health state, indicating that the torrent is in poor condition.
    #[display(fmt = "bad")]
    Bad,
    /// Medium health state, indicating that the torrent is in a moderate condition.
    #[display(fmt = "medium")]
    Medium,
    /// Good health state, indicating that the torrent is in good condition.
    #[display(fmt = "good")]
    Good,
    /// Excellent health state, indicating that the torrent is in excellent condition.
    #[display(fmt = "excellent")]
    Excellent,
}

/// Represents the health statistics of a torrent.
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct TorrentHealth {
    /// The health state of the torrent.
    pub state: TorrentHealthState,
    /// The ratio of uploaded data to downloaded data for the torrent.
    pub ratio: f32,
    /// The number of seeders (peers with a complete copy of the torrent).
    pub seeds: u32,
    /// The number of leechers currently downloading the torrent.
    pub leechers: u32,
}

impl TorrentHealth {
    pub fn from(seeds: u32, leechers: u32) -> Self {
        // the seeds that have completed the download
        let seeds = seeds as f64;
        // the leechers that have partially downloaded the torrent
        let leechers = leechers as f64;

        let ratio = if leechers > 0.0 {
            seeds / leechers
        } else {
            seeds
        };

        // Precompute constants
        const RATIO_WEIGHT: f64 = 0.6;
        const SEEDS_WEIGHT: f64 = 0.4;

        // Normalize the data
        let normalized_ratio = f64::min(ratio / 5.0 * 100.0, 100.0);
        let normalized_seeds = f64::min(seeds / 30.0 * 100.0, 100.0);

        // Weight the metrics
        let weighted_total = (normalized_ratio * RATIO_WEIGHT) + (normalized_seeds * SEEDS_WEIGHT);
        let scaled_total = (weighted_total * 3.0 / 100.0).round() as u64;

        // Determine the health state
        let health_state = if seeds == 0f64 && leechers == 0f64 {
            TorrentHealthState::Unknown
        } else {
            match scaled_total {
                0 => TorrentHealthState::Bad,
                1 => TorrentHealthState::Medium,
                2 => TorrentHealthState::Good,
                3 => TorrentHealthState::Excellent,
                _ => TorrentHealthState::Unknown,
            }
        };

        Self {
            state: health_state,
            ratio: ratio as f32,
            seeds: seeds as u32,
            leechers: leechers as u32,
        }
    }
}

#[cfg(any(test, feature = "testing"))]
mod mock {
    use super::*;
    use crate::core::CallbackHandle;
    use mockall::mock;
    use std::fmt::{Display, Formatter};
    use std::ops::Range;

    mock! {
        #[derive(Debug, Clone)]
        pub Torrent {}

        #[async_trait]
        impl Torrent for Torrent {
            fn handle(&self) -> TorrentHandle;
            async fn file(&self) -> PathBuf;
            async fn has_bytes(&self, bytes: &Range<usize>) -> bool;
            async fn has_piece(&self, piece: usize) -> bool;
            async fn prioritize_bytes(&self, bytes: &Range<usize>);
            async fn prioritize_pieces(&self, pieces: &[u32]);
            async fn total_pieces(&self) -> usize;
            async fn sequential_mode(&self);
            async fn state(&self) -> TorrentState;
        }

        impl Callbacks<TorrentEvent> for Torrent {
            fn add_callback(&self, callback: CoreCallback<TorrentEvent>) -> CallbackHandle;
            fn remove_callback(&self, handle: CallbackHandle);
        }
    }

    impl Display for MockTorrent {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "MockTorrent")
        }
    }
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
        assert_eq!(TorrentState::Initializing, creating);
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
            info_hash: String::new(),
            uri: String::new(),
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
            info_hash: String::new(),
            uri: String::new(),
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
            info_hash: String::new(),
            uri: String::new(),
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

    #[test]
    fn test_torrent_health_from() {
        let expected_result = TorrentHealth {
            state: Default::default(),
            ratio: 0.0,
            seeds: 0,
            leechers: 0,
        };
        let result = TorrentHealth::from(0, 0);
        assert_eq!(expected_result, result);

        let expected_result = TorrentHealth {
            state: TorrentHealthState::Bad,
            ratio: 0.5,
            seeds: 5,
            leechers: 10,
        };
        let result = TorrentHealth::from(5, 10);
        assert_eq!(expected_result, result);

        let expected_result = TorrentHealth {
            state: TorrentHealthState::Medium,
            ratio: 1.0,
            seeds: 10,
            leechers: 10,
        };
        let result = TorrentHealth::from(10, 10);
        assert_eq!(expected_result, result);

        let expected_result = TorrentHealth {
            state: TorrentHealthState::Good,
            ratio: 3.5,
            seeds: 35,
            leechers: 10,
        };
        let result = TorrentHealth::from(35, 10);
        assert_eq!(expected_result, result);

        let expected_result = TorrentHealth {
            state: TorrentHealthState::Excellent,
            ratio: 5.0,
            seeds: 50,
            leechers: 10,
        };
        let result = TorrentHealth::from(50, 10);
        assert_eq!(expected_result, result);
    }
}
