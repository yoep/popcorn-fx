use std::fmt::{Display, Formatter};
use std::os::raw::c_char;
use std::path::PathBuf;
use std::sync::Arc;

use popcorn_fx_core::{from_c_string, to_c_vec};
use popcorn_fx_core::core::torrent::{Torrent, TorrentCallback, TorrentState};

use crate::torrent::TorrentWrapper;

/// The callback to verify if the given byte is available.
pub type HasByteCallbackC = extern "C" fn(i32, *mut u64) -> bool;

/// The callback to retrieve the total pieces of the torrent.
pub type TotalPiecesCallbackC = extern "C" fn() -> i32;

/// The callback for prioritizing pieces.
pub type PrioritizePiecesCallbackC = extern "C" fn(i32, *mut u32);

/// The callback for update the torrent mode to sequential.
pub type SequentialModeCallbackC = extern "C" fn();

/// The C compatible abi struct for a [Torrent].
/// This currently uses callbacks as it's a wrapper around a torrent implementation provided through C.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct TorrentC {
    /// The filepath to the torrent file
    pub filepath: *const c_char,
    pub has_byte_callback: HasByteCallbackC,
    pub total_pieces: TotalPiecesCallbackC,
    pub prioritize_pieces: PrioritizePiecesCallbackC,
    pub sequential_mode: SequentialModeCallbackC,
}

impl From<TorrentC> for TorrentWrapper {
    fn from(value: TorrentC) -> Self {
        TorrentWrapper::new(
            from_c_string(value.filepath),
            Box::new(move |bytes| -> bool {
                let (bytes, len) = to_c_vec(bytes.to_vec());
                (value.has_byte_callback)(len, bytes)
            }),
            Box::new(move || (value.total_pieces)()),
            Box::new(move |pieces| {
                let (pieces, len) = to_c_vec(pieces.to_vec());
                (value.prioritize_pieces)(len, pieces)
            }),
            Box::new(move || (value.sequential_mode)()),
        )
    }
}

/// The wrapper communication between rust and C.
/// This is a temp wrapper which will be replaced in the future.
#[repr(C)]
#[derive(Debug)]
pub struct TorrentWrapperC {
    wrapper: Arc<TorrentWrapper>,
}

impl TorrentWrapperC {
    pub fn wrapper(&self) -> &Arc<TorrentWrapper> {
        &self.wrapper
    }

    pub fn state_changed(&self, state: TorrentState) {
        self.wrapper.state_changed(state)
    }

    pub fn piece_finished(&self, piece: u32) {
        self.wrapper.piece_finished(piece)
    }
}

impl From<TorrentC> for TorrentWrapperC {
    fn from(value: TorrentC) -> Self {
        Self {
            wrapper: Arc::new(TorrentWrapper::from(value))
        }
    }
}

impl Display for TorrentWrapperC {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.wrapper.fmt(f)
    }
}

impl Torrent for &'static TorrentWrapperC {
    fn file(&self) -> PathBuf {
        self.wrapper.file()
    }

    fn has_bytes(&self, bytes: &[u64]) -> bool {
        self.wrapper.has_bytes(bytes)
    }

    fn prioritize_bytes(&self, bytes: &[u64]) {
        self.wrapper.prioritize_bytes(bytes)
    }

    fn prioritize_pieces(&self, pieces: &[u32]) {
        self.wrapper.prioritize_pieces(pieces)
    }

    fn total_pieces(&self) -> i32 {
        self.wrapper.total_pieces()
    }

    fn sequential_mode(&self) {
        self.wrapper.sequential_mode()
    }

    fn register(&self, callback: TorrentCallback) {
        self.wrapper.register(callback)
    }
}

#[cfg(test)]
mod test {
    use popcorn_fx_core::into_c_string;

    use super::*;

    #[no_mangle]
    pub extern "C" fn has_bytes_callback(_: i32, _: *mut u64) -> bool {
        true
    }

    #[no_mangle]
    pub extern "C" fn total_pieces_callback() -> i32 {
        10
    }

    #[no_mangle]
    pub extern "C" fn prioritize_pieces_callback(_: i32, _: *mut u32) {}

    #[no_mangle]
    pub extern "C" fn sequential_mode_callback() {}

    #[test]
    pub fn test_from_torrent_c_to_wrapper() {
        let torrent = TorrentC {
            filepath: into_c_string("lorem.csv".to_string()),
            has_byte_callback: has_bytes_callback,
            total_pieces: total_pieces_callback,
            prioritize_pieces: prioritize_pieces_callback,
            sequential_mode: sequential_mode_callback,
        };

        let wrapper = TorrentWrapper::from(torrent);

        assert_eq!(10, wrapper.total_pieces())
    }

    #[test]
    pub fn test_has_bytes() {
        let torrent = TorrentC {
            filepath: into_c_string("lorem.csv".to_string()),
            has_byte_callback: has_bytes_callback,
            total_pieces: total_pieces_callback,
            prioritize_pieces: prioritize_pieces_callback,
            sequential_mode: sequential_mode_callback,
        };
        let bytes = vec![10, 11, 12];

        let wrapper = TorrentWrapper::from(torrent);

        assert_eq!(true, wrapper.has_bytes(&bytes[..]))
    }
}