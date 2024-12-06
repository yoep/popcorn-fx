use crate::torrent::operation::{
    TorrentFileValidationOperation, TorrentFilesOperation, TorrentMetadataOperation,
    TorrentPeersOperation, TorrentPendingRequestsOperation, TorrentPiecesOperation,
    TorrentRetrievePendingRequestsOperation, TorrentTrackersOperation,
};
#[cfg(feature = "extension-metadata")]
use crate::torrent::peer::extension::metadata::MetadataExtension;
use crate::torrent::peer::extension::Extensions;
use crate::torrent::peer::ProtocolExtensionFlags;
use crate::torrent::request_strategy::{PriorityRequestStrategy, RequestAvailabilityStrategy};
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
pub mod operation;
pub mod peer;
mod peer_pool;
mod piece;
pub mod request_strategy;
mod session;
mod torrent;
mod torrent_info;
mod torrent_request_buffer;
mod tracker;

const DEFAULT_TORRENT_PROTOCOL_EXTENSIONS: fn() -> ProtocolExtensionFlags =
    || ProtocolExtensionFlags::LTEP | ProtocolExtensionFlags::Fast;
const DEFAULT_TORRENT_EXTENSIONS: fn() -> Extensions = || {
    let mut extensions: Extensions = Vec::new();

    #[cfg(feature = "extension-metadata")]
    extensions.push(Box::new(MetadataExtension::new()));

    extensions
};
const DEFAULT_TORRENT_OPERATIONS: fn() -> TorrentOperations = || {
    vec![
        Box::new(TorrentTrackersOperation::new()),
        Box::new(TorrentPeersOperation::new()),
        Box::new(TorrentMetadataOperation::new()),
        Box::new(TorrentPiecesOperation::new()),
        Box::new(TorrentFilesOperation::new()),
        Box::new(TorrentFileValidationOperation::new()),
        Box::new(TorrentPendingRequestsOperation::new()),
        Box::new(TorrentRetrievePendingRequestsOperation::new()),
    ]
};
const DEFAULT_TORRENT_REQUEST_STRATEGIES: fn() -> Vec<Box<dyn RequestStrategy>> = || {
    vec![
        Box::new(PriorityRequestStrategy::new()),
        Box::new(RequestAvailabilityStrategy::new()),
    ]
};
