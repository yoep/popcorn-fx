use crate::torrents::operations::{
    TorrentFileValidationOperation, TorrentFilesOperation, TorrentMetadataOperation,
    TorrentPendingRequestsOperation, TorrentPiecesOperation, TorrentTrackersOperation,
};
#[cfg(feature = "extension-metadata")]
use crate::torrents::peers::extensions::metadata::MetadataExtension;
use crate::torrents::peers::extensions::Extensions;
use crate::torrents::request_strategies::{PriorityRequestStrategy, RequestAvailabilityStrategy};
pub use errors::*;
pub use file::*;
pub use info_hash::*;
pub use manager::*;
pub use piece::*;
pub use session::*;
pub use torrent::*;
pub use torrent_info::*;

mod errors;
mod file;
pub mod fs;
mod info_hash;
mod manager;
pub mod operations;
pub mod peers;
mod piece;
pub mod request_strategies;
mod session;
mod torrent;
mod torrent_info;
mod torrent_request_buffer;
mod trackers;

const DEFAULT_TORRENT_EXTENSIONS: fn() -> Extensions = || {
    let mut extensions: Extensions = Vec::new();

    #[cfg(feature = "extension-metadata")]
    extensions.push(Box::new(MetadataExtension::new()));

    extensions
};
const DEFAULT_TORRENT_OPERATIONS: fn() -> TorrentOperations = || {
    vec![
        Box::new(TorrentTrackersOperation::new()),
        Box::new(TorrentMetadataOperation::new()),
        Box::new(TorrentPiecesOperation::new()),
        Box::new(TorrentFilesOperation::new()),
        Box::new(TorrentFileValidationOperation::new()),
        Box::new(TorrentPendingRequestsOperation::new()),
    ]
};
const DEFAULT_TORRENT_REQUEST_STRATEGIES: fn() -> Vec<Box<dyn RequestStrategy>> = || {
    vec![
        Box::new(PriorityRequestStrategy::new()),
        Box::new(RequestAvailabilityStrategy::new()),
    ]
};
