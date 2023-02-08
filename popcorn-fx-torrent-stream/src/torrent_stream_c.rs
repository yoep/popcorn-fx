use std::os::raw::c_char;
use std::sync::Arc;

use popcorn_fx_core::core::torrent::TorrentStream;
use popcorn_fx_core::into_c_string;

#[repr(C)]
#[derive(Debug)]
pub struct TorrentStreamC {
    url: *const c_char,
}

impl From<Arc<dyn TorrentStream>> for TorrentStreamC {
    fn from(value: Arc<dyn TorrentStream>) -> Self {
        TorrentStreamC {
            url: into_c_string(value.url().to_string()),
        }
    }
}
