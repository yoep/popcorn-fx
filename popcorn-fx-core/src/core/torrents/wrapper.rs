use std::fmt::{Debug, Formatter};
use std::path::PathBuf;

use derive_more::Display;
use log::trace;
use tokio::sync::Mutex;

use crate::core::{CallbackHandle, Callbacks, CoreCallbacks};
use crate::core::torrents::{DownloadStatus, Torrent, TorrentCallback, TorrentEvent, TorrentState};

/// The has byte callback.
pub type HasBytesCallback = Box<dyn Fn(&[u64]) -> bool + Send>;

/// The has piece callback.
pub type HasPieceCallback = Box<dyn Fn(u32) -> bool + Send>;

/// The total number of pieces callback.
pub type TotalPiecesCallback = Box<dyn Fn() -> i32 + Send>;

/// The prioritization of bytes callback.
pub type PrioritizeBytesCallback = Box<dyn Fn(&[u64]) + Send>;

/// The prioritization of pieces callback.
pub type PrioritizePiecesCallback = Box<dyn Fn(&[u32]) + Send>;

/// The callback for update the torrent mode to sequential.
pub type SequentialModeCallback = Box<dyn Fn() + Send>;

/// The callback for retrieving the torrent state.
pub type TorrentStateCallback = Box<dyn Fn() -> TorrentState + Send>;

/// The callback for cancelling the torrent.
pub type CancelTorrentCallback = Box<dyn Fn() + Send>;

/// The wrapper containing the callbacks to retrieve the actual torrent information from C.
#[derive(Display)]
#[display(fmt = "filepath: {:?}", filepath)]
pub struct TorrentWrapper {
    /// The handle for identifying the torrent.
    pub handle: String,
    /// The filepath of the torrent.
    pub filepath: PathBuf,
    /// Mutex for the callback to check if a byte exists in the torrent.
    pub has_bytes: Mutex<HasBytesCallback>,
    /// Mutex for the callback to check if a piece exists in the torrent.
    pub has_piece: Mutex<HasPieceCallback>,
    /// Mutex for the callback to retrieve the total number of pieces in the torrent.
    pub total_pieces: Mutex<TotalPiecesCallback>,
    /// Mutex for the callback to prioritize bytes in the torrent.
    pub prioritize_bytes: Mutex<PrioritizeBytesCallback>,
    /// Mutex for the callback to prioritize pieces in the torrent.
    pub prioritize_pieces: Mutex<PrioritizePiecesCallback>,
    /// Mutex for the callback to set sequential mode in the torrent.
    pub sequential_mode: Mutex<SequentialModeCallback>,
    /// Mutex for the callback to handle torrent state changes.
    pub torrent_state: Mutex<TorrentStateCallback>,
    /// Callbacks for handling torrent events.
    pub callbacks: CoreCallbacks<TorrentEvent>,
}

impl TorrentWrapper {
    /// Creates a new `TorrentWrapper` instance.
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle for identifying the torrent.
    /// * `filepath` - The filepath of the torrent.
    /// * `has_byte` - The callback for checking if a byte exists in the torrent.
    /// * `has_piece` - The callback for checking if a piece exists in the torrent.
    /// * `total_pieces` - The callback for retrieving the total number of pieces in the torrent.
    /// * `prioritize_bytes` - The callback for prioritizing bytes in the torrent.
    /// * `prioritize_pieces` - The callback for prioritizing pieces in the torrent.
    /// * `sequential_mode` - The callback for setting sequential mode in the torrent.
    /// * `torrent_state` - The callback for handling torrent state changes.
    ///
    /// # Returns
    ///
    /// A new `TorrentWrapper` instance.
    pub fn new(
        handle: String,
        filepath: String,
        has_byte: HasBytesCallback,
        has_piece: HasPieceCallback,
        total_pieces: TotalPiecesCallback,
        prioritize_bytes: PrioritizeBytesCallback,
        prioritize_pieces: PrioritizePiecesCallback,
        sequential_mode: SequentialModeCallback,
        torrent_state: TorrentStateCallback,
    ) -> Self {
        Self {
            handle,
            filepath: PathBuf::from(filepath),
            has_bytes: Mutex::new(has_byte),
            has_piece: Mutex::new(has_piece),
            total_pieces: Mutex::new(total_pieces),
            prioritize_bytes: Mutex::new(prioritize_bytes),
            prioritize_pieces: Mutex::new(prioritize_pieces),
            sequential_mode: Mutex::new(sequential_mode),
            torrent_state: Mutex::new(torrent_state),
            callbacks: CoreCallbacks::default(),
        }
    }

    /// Notifies the wrapper that the state of the torrent has changed.
    ///
    /// # Arguments
    ///
    /// * `state` - The new state of the torrent.
    pub fn state_changed(&self, state: TorrentState) {
        self.callbacks.invoke(TorrentEvent::StateChanged(state))
    }

    /// Notifies the wrapper that a piece of the torrent has finished downloading.
    ///
    /// # Arguments
    ///
    /// * `piece` - The index of the finished piece.
    pub fn piece_finished(&self, piece: u32) {
        self.callbacks.invoke(TorrentEvent::PieceFinished(piece))
    }

    /// Notifies the wrapper of the torrent's download status.
    ///
    /// # Arguments
    ///
    /// * `download_status` - The download status of the torrent.
    pub fn download_status(&self, download_status: DownloadStatus) {
        self.callbacks
            .invoke(TorrentEvent::DownloadStatus(download_status))
    }
}

impl Debug for TorrentWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TorrentWrapper")
            .field("filepath", &self.filepath)
            .field("callbacks", &self.callbacks)
            .finish()
    }
}

impl Torrent for TorrentWrapper {
    fn handle(&self) -> &str {
        self.handle.as_str()
    }

    fn file(&self) -> PathBuf {
        self.filepath.clone()
    }

    fn has_bytes(&self, bytes: &[u64]) -> bool {
        tokio::task::block_in_place(move || {
            let mutex = self.has_bytes.blocking_lock();
            mutex(bytes)
        })
    }

    fn has_piece(&self, piece: u32) -> bool {
        tokio::task::block_in_place(move || {
            let mutex = self.has_piece.blocking_lock();
            mutex(piece)
        })
    }

    fn prioritize_bytes(&self, bytes: &[u64]) {
        tokio::task::block_in_place(move || {
            let mutex = self.prioritize_bytes.blocking_lock();
            mutex(bytes)
        })
    }

    fn prioritize_pieces(&self, pieces: &[u32]) {
        tokio::task::block_in_place(move || {
            let mutex = self.prioritize_pieces.blocking_lock();
            trace!("Prioritizing a total of {} torrent pieces", pieces.len());
            mutex(pieces)
        })
    }

    fn total_pieces(&self) -> i32 {
        tokio::task::block_in_place(move || {
            let mutex = self.total_pieces.blocking_lock();
            mutex()
        })
    }

    fn sequential_mode(&self) {
        tokio::task::block_in_place(move || (self.sequential_mode.blocking_lock())())
    }

    fn state(&self) -> TorrentState {
        tokio::task::block_in_place(move || (self.torrent_state.blocking_lock())())
    }

    fn subscribe(&self, callback: TorrentCallback) -> CallbackHandle {
        self.callbacks.add(callback)
    }
}

#[cfg(test)]
mod test {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_has_bytes() {
        let (tx, rx) = channel();
        let has_bytes: HasBytesCallback = Box::new(move |byte| {
            tx.send(byte.to_vec()).unwrap();
            true
        });
        let has_piece = Box::new(|_: u32| true);
        let total_pieces = Box::new(|| 0);
        let prioritize_bytes = Box::new(|_: &[u64]| {});
        let prioritize_pieces = Box::new(|_: &[u32]| {});
        let sequential_mode = Box::new(|| {});
        let torrent_state = Box::new(|| TorrentState::Completed);
        let wrapper = TorrentWrapper::new(
            "MyHandle".to_string(),
            "lorem.txt".to_string(),
            has_bytes,
            has_piece,
            total_pieces,
            prioritize_bytes,
            prioritize_pieces,
            sequential_mode,
            torrent_state,
        );
        let bytes = vec![2, 3];

        let result = wrapper.has_bytes(&bytes[..]);
        let result_byte = rx.recv_timeout(Duration::from_secs(3)).unwrap();

        assert!(result, "expected true to have been returned");
        assert_eq!(bytes, result_byte)
    }

    #[test]
    fn test_state() {
        let has_bytes: HasBytesCallback = Box::new(move |_| true);
        let has_piece = Box::new(|_: u32| true);
        let total_pieces = Box::new(|| 0);
        let prioritize_bytes = Box::new(|_: &[u64]| {});
        let prioritize_pieces = Box::new(|_: &[u32]| {});
        let sequential_mode = Box::new(|| {});
        let torrent_state = Box::new(|| TorrentState::Completed);
        let wrapper = TorrentWrapper::new(
            "MyHandle".to_string(),
            "lorem.txt".to_string(),
            has_bytes,
            has_piece,
            total_pieces,
            prioritize_bytes,
            prioritize_pieces,
            sequential_mode,
            torrent_state,
        );

        let result = wrapper.state();

        assert_eq!(TorrentState::Completed, result)
    }
}
