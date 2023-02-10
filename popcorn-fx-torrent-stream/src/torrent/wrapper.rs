use std::fmt::{Debug, Formatter};
use std::path::PathBuf;

use derive_more::Display;
use log::trace;
use tokio::sync::Mutex;

use popcorn_fx_core::core::CoreCallbacks;
use popcorn_fx_core::core::torrent::{Torrent, TorrentCallback, TorrentEvent, TorrentState};

/// The has byte callback.
pub type HasByteCallback = Box<dyn Fn(&[u64]) -> bool + Send>;

/// The total number of pieces callback.
pub type TotalPiecesCallback = Box<dyn Fn() -> i32 + Send>;

/// The prioritization of pieces callback.
pub type PrioritizePiecesCallback = Box<dyn Fn(&[u32]) + Send>;

/// The callback for update the torrent mode to sequential.
pub type SequentialModeCallback = Box<dyn Fn() + Send>;

#[derive(Display)]
#[display(fmt = "filepath: {:?}", filepath)]
pub struct TorrentWrapper {
    filepath: PathBuf,
    has_byte: Mutex<HasByteCallback>,
    total_pieces: Mutex<TotalPiecesCallback>,
    prioritize_pieces: Mutex<PrioritizePiecesCallback>,
    sequential_mode: Mutex<SequentialModeCallback>,
    callbacks: CoreCallbacks<TorrentEvent>,
}

impl TorrentWrapper {
    pub fn new(filepath: String, has_byte: HasByteCallback, total_pieces: TotalPiecesCallback, prioritize_pieces: PrioritizePiecesCallback, sequential_mode: SequentialModeCallback) -> Self {
        Self {
            filepath: PathBuf::from(filepath),
            has_byte: Mutex::new(has_byte),
            total_pieces: Mutex::new(total_pieces),
            prioritize_pieces: Mutex::new(prioritize_pieces),
            sequential_mode: Mutex::new(sequential_mode),
            callbacks: CoreCallbacks::default(),
        }
    }

    pub fn state_changed(&self, state: TorrentState) {
        self.callbacks.invoke(TorrentEvent::StateChanged(state))
    }

    pub fn piece_finished(&self, piece: u32) {
        self.callbacks.invoke(TorrentEvent::PieceFinished(piece))
    }
}

impl Debug for TorrentWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "filepath: {:?}", self.filepath)
    }
}

impl Torrent for TorrentWrapper {
    fn file(&self) -> PathBuf {
        self.filepath.clone()
    }

    fn has_bytes(&self, bytes: &[u64]) -> bool {
        tokio::task::block_in_place(move || {
            let mutex = self.has_byte.blocking_lock();
            mutex(bytes)
        })
    }

    fn prioritize_bytes(&self, bytes: &[u64]) {}

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
        tokio::task::block_in_place(move || {
            (self.sequential_mode.blocking_lock())()
        })
    }

    fn register(&self, callback: TorrentCallback) {
        self.callbacks.add(callback);
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
        let callback: HasByteCallback = Box::new(move |byte| {
            tx.send(byte.to_vec()).unwrap();
            true
        });
        let total_pieces = Box::new(|| 0);
        let prioritize_pieces = Box::new(|_: &[u32]| {});
        let sequential_mode = Box::new(|| {});
        let wrapper = TorrentWrapper::new("lorem.txt".to_string(), callback, total_pieces, prioritize_pieces, sequential_mode);
        let bytes = vec![2, 3];

        let result = wrapper.has_bytes(&bytes[..]);
        let result_byte = rx.recv_timeout(Duration::from_secs(3)).unwrap();

        assert!(result, "expected true to have been returned");
        assert_eq!(bytes, result_byte)
    }
}