use async_trait::async_trait;
use derive_more::Display;
use downcast_rs::{impl_downcast, DowncastSync};
use fx_callback::Callback;
use fx_handle::Handle;
use log::{debug, trace};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::path::PathBuf;

use popcorn_fx_torrent::torrent;
use popcorn_fx_torrent::torrent::{Metrics, PiecePriority, TorrentFlags};
pub use popcorn_fx_torrent::torrent::{PieceIndex, TorrentEvent, TorrentState};

#[cfg(any(test, feature = "testing"))]
pub use mock::*;

/// A unique handle identifier of a [Torrent].
pub type TorrentHandle = Handle;

/// The torrent describes the meta-info of a shared file that can be queried over the network.
/// It allows for action such as downloading the shared file to the local system.
#[async_trait]
pub trait Torrent: Debug + DowncastSync + Callback<TorrentEvent> + Send + Sync {
    /// Get the unique identifier handle of the torrent.
    fn handle(&self) -> TorrentHandle;

    /// Get the absolute filesystem path to a given file in the torrent.
    async fn absolute_file_path(&self, file: &torrent::File) -> PathBuf;

    /// Get the files of the torrent.
    /// It might return an empty array if the metadata is unknown.
    async fn files(&self) -> Vec<torrent::File>;

    /// Get the torrent file information by its name.
    ///
    /// # Arguments
    ///
    /// * `name` - The torrent filename.
    ///
    /// # Returns
    ///
    /// It returns the torrent file info for the given torrent filename if found, else [None].
    async fn file_by_name(&self, name: &str) -> Option<torrent::File>;

    /// Get the largest file of the torrent.
    /// It returns [None] if the metadata is currently unknown of the torrent.
    async fn largest_file(&self) -> Option<torrent::File>;

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
    async fn has_piece(&self, piece: PieceIndex) -> bool;

    /// Prioritize the given bytes to be downloaded.
    async fn prioritize_bytes(&self, bytes: &std::ops::Range<usize>);

    /// Prioritize the given piece indexes.
    async fn prioritize_pieces(&self, pieces: &[PieceIndex]);

    /// Returns the piece priorities of the torrent.
    async fn piece_priorities(&self) -> BTreeMap<PieceIndex, PiecePriority>;

    /// Get the total number of pieces in the torrent.
    /// It might return [None] when the metadata is still being retrieved.
    async fn total_pieces(&self) -> usize;

    /// Update the download mode of the torrent to sequential.
    async fn sequential_mode(&self);

    /// Get the current state of the torrent.
    /// It returns an owned instance of the state.
    async fn state(&self) -> TorrentState;

    /// Get the torrent metrics statics.
    fn stats(&self) -> &Metrics;
}
impl_downcast!(sync Torrent);

#[async_trait]
impl Torrent for torrent::Torrent {
    fn handle(&self) -> TorrentHandle {
        self.handle()
    }

    async fn absolute_file_path(&self, file: &torrent::File) -> PathBuf {
        self.absolute_file_path(file).await
    }

    async fn files(&self) -> Vec<torrent::File> {
        self.files().await
    }

    async fn file_by_name(&self, name: &str) -> Option<torrent::File> {
        self.files()
            .await
            .into_iter()
            .find(|e| e.info.filename() == name)
    }

    async fn largest_file(&self) -> Option<torrent::File> {
        let mut result: Option<torrent::File> = None;

        for file in self.files().await {
            if let Some(current) = result.as_ref() {
                if current.len() < file.len() {
                    result = Some(file);
                }
            } else {
                result = Some(file);
            }
        }

        result
    }

    async fn has_bytes(&self, bytes: &std::ops::Range<usize>) -> bool {
        self.has_bytes(bytes).await
    }

    async fn has_piece(&self, piece: usize) -> bool {
        self.has_piece(&(piece as PieceIndex)).await
    }

    async fn prioritize_bytes(&self, bytes: &std::ops::Range<usize>) {
        self.prioritize_bytes(bytes, PiecePriority::Now).await
    }

    async fn prioritize_pieces(&self, pieces: &[PieceIndex]) {
        let mut priorities = Vec::new();

        for piece in pieces {
            priorities.push((*piece as PieceIndex, PiecePriority::High));
        }

        self.prioritize_pieces(priorities).await;
    }

    async fn piece_priorities(&self) -> BTreeMap<PieceIndex, PiecePriority> {
        self.piece_priorities().await
    }

    async fn total_pieces(&self) -> usize {
        self.total_pieces().await
    }

    async fn sequential_mode(&self) {
        self.add_options(TorrentFlags::SequentialDownload).await
    }

    async fn state(&self) -> TorrentState {
        self.state().await
    }

    fn stats(&self) -> &Metrics {
        self.metrics()
    }
}

/// The torrent information
#[derive(Debug, Display, Clone, PartialEq)]
#[display(
    fmt = "handle: {}, info_hash: {}, name: {}, directory_name: {:?}, total_files: {}",
    handle,
    info_hash,
    name,
    directory_name,
    total_files
)]
pub struct TorrentInfo {
    /// The handle of the underlying torrent
    pub handle: TorrentHandle,
    /// The info hash of the torrent.
    pub info_hash: String,
    /// The uri of the torrent
    pub uri: String,
    /// The name of the torrent
    pub name: String,
    /// The torrent directory name in which the media files might reside.
    pub directory_name: Option<String>,
    /// The total number of files available in the torrent
    pub total_files: u32,
    /// The available files
    pub files: Vec<torrent::File>,
}

impl TorrentInfo {
    pub fn by_filename(&self, filename: &str) -> Option<torrent::File> {
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

    pub fn largest_file(&self) -> Option<torrent::File> {
        let mut largest_file_index: Option<usize> = None;
        let mut largest_file_size = 0;
        let mut index: usize = 0;

        for file in self.files.iter() {
            if file.len() > largest_file_size {
                largest_file_index = Some(index);
                largest_file_size = file.len();
            }

            index += 1;
        }

        largest_file_index.and_then(|e| self.files.get(e)).cloned()
    }

    fn by_filename_without_directory(&self, filename: &str) -> Option<&torrent::File> {
        debug!(
            "Searching for torrent file {} without torrent directory",
            filename
        );
        self.files.iter().find(|e| {
            let filepath = Self::simplified_filepath(e.torrent_path.to_str().unwrap_or(""));
            let expected_filepath = Self::simplified_filepath(filename);

            trace!(
                "Checking if filepath \"{}\" matches filename \"{}\" without torrent directory",
                filepath,
                expected_filepath
            );
            filepath.eq_ignore_ascii_case(expected_filepath.as_str())
        })
    }

    fn by_filename_with_directory(&self, filename: &str) -> Option<&torrent::File> {
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
            let filepath = Self::simplified_filepath(e.torrent_path.to_str().unwrap_or(""));

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

#[cfg(any(test, feature = "testing"))]
mod mock {
    use super::*;
    use fx_callback::{Subscriber, Subscription};
    use mockall::mock;
    use std::fmt::{Display, Formatter};
    use std::ops::Range;

    mock! {
        #[derive(Debug, Clone)]
        pub Torrent {}

        #[async_trait]
        impl Torrent for Torrent {
            fn handle(&self) -> TorrentHandle;
            async fn absolute_file_path(&self, file: &torrent::File) -> PathBuf;
            async fn files(&self) -> Vec<torrent::File>;
            async fn file_by_name(&self, name: &str) -> Option<torrent::File>;
            async fn largest_file(&self) -> Option<torrent::File>;
            async fn has_bytes(&self, bytes: &Range<usize>) -> bool;
            async fn has_piece(&self, piece: usize) -> bool;
            async fn prioritize_bytes(&self, bytes: &Range<usize>);
            async fn prioritize_pieces(&self, pieces: &[PieceIndex]);
            async fn piece_priorities(&self) -> BTreeMap<PieceIndex, PiecePriority>;
            async fn total_pieces(&self) -> usize;
            async fn sequential_mode(&self);
            async fn state(&self) -> TorrentState;
            fn stats(&self) -> &Metrics;
        }

        impl Callback<TorrentEvent> for Torrent {
            fn subscribe(&self) -> Subscription<TorrentEvent>;
            fn subscribe_with(&self, subscriber: Subscriber<TorrentEvent>);
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
    use super::*;
    use crate::init_logger;
    use popcorn_fx_torrent::torrent::TorrentFileInfo;

    #[test]
    fn test_torrent_info_by_filename_match_without_torrent_directory() {
        init_logger!();
        let filename = "lorem.mp4";
        let expected_result = create_torrent_file("/lorem.mp4", 1500);
        let info = TorrentInfo {
            handle: Default::default(),
            info_hash: String::new(),
            uri: "".to_string(),
            name: "".to_string(),
            directory_name: Some("torrentDirectory".to_string()),
            total_files: 0,
            files: vec![
                expected_result.clone(),
                create_torrent_file("ipsum.mp4", 18000),
            ],
        };

        let result = info.by_filename(filename);

        assert_eq!(Some(expected_result), result);
    }

    #[test]
    fn test_torrent_info_by_filename_match_with_torrent_directory() {
        init_logger!();
        let filename = "ipsum.mp4";
        let expected_result = create_torrent_file("torrentDirectory/ipsum.mp4", 23000);
        let info = TorrentInfo {
            handle: Default::default(),
            info_hash: String::new(),
            uri: "".to_string(),
            name: "".to_string(),
            directory_name: Some("torrentDirectory".to_string()),
            total_files: 0,
            files: vec![
                create_torrent_file("torrentDirectory/lorem.mp4", 18000),
                expected_result.clone(),
            ],
        };

        let result = info.by_filename(filename);

        assert_eq!(Some(expected_result), result);
    }

    #[test]
    fn test_torrent_info_largest_file() {
        let largest_file = create_torrent_file("file2", 230);
        let info = TorrentInfo {
            handle: Default::default(),
            info_hash: String::new(),
            uri: "".to_string(),
            name: "".to_string(),
            directory_name: None,
            total_files: 0,
            files: vec![
                create_torrent_file("file1", 150),
                largest_file.clone(),
                create_torrent_file("file3", 220),
            ],
        };

        let result = info.largest_file();

        assert_eq!(Some(largest_file), result);
    }

    fn create_torrent_file(relative_path: &str, length: u64) -> torrent::File {
        torrent::File {
            index: 0,
            torrent_path: PathBuf::from(relative_path),
            torrent_offset: 0,
            info: TorrentFileInfo {
                length,
                path: None,
                path_utf8: None,
                md5sum: None,
                attr: None,
                symlink_path: None,
                sha1: None,
            },
            priority: Default::default(),
            pieces: 0..100,
        }
    }
}
