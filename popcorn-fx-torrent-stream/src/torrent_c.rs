use std::os::raw::c_char;

use popcorn_fx_core::{from_c_string, to_c_vec};

use crate::torrent::TorrentWrapper;

/// The callback to verify if the given byte is available.
pub type HasByteCallbackC = extern "C" fn(i32, *mut u64) -> bool;

/// The C compatible abi struct for a [Torrent].
/// This currently uses callbacks as it's a wrapper around a torrent implementation provided through C.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct TorrentC {
    filepath: *const c_char,
    has_byte_callback: HasByteCallbackC,
}

impl From<TorrentC> for TorrentWrapper {
    fn from(value: TorrentC) -> Self {
        TorrentWrapper::new(
            from_c_string(value.filepath),
            Box::new(move |bytes| -> bool {
                let (bytes, len) = to_c_vec(bytes.to_vec());

                (value.has_byte_callback)(len, bytes)
            }),
        )
    }
}