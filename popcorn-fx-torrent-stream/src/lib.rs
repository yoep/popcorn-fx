extern crate core;

pub use torrent_c::*;
pub use torrent_stream_c::*;

pub mod torrent;
mod torrent_stream_c;
mod torrent_c;