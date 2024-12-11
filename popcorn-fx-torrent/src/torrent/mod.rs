use crate::torrent::operation::{
    TorrentConnectPeersOperation, TorrentCreateFilesOperation, TorrentCreatePiecesOperation,
    TorrentFileValidationOperation, TorrentMetadataOperation, TorrentTrackersOperation,
};
#[cfg(feature = "extension-metadata")]
use crate::torrent::peer::extension::metadata::MetadataExtension;
use crate::torrent::peer::extension::pex::PexExtension;
use crate::torrent::peer::ProtocolExtensionFlags;
pub use compact::*;
pub use errors::*;
pub use file::*;
pub use info_hash::*;
pub use manager::*;
pub use piece::*;
pub use session::*;
pub use torrent::*;
pub use torrent_info::*;

mod compact;
mod errors;
mod file;
pub mod fs;
mod info_hash;
mod manager;
pub mod operation;
pub mod peer;
mod peer_pool;
mod piece;
mod session;
mod torrent;
mod torrent_info;
mod tracker;

// TODO: fix the fast protocol
const DEFAULT_TORRENT_PROTOCOL_EXTENSIONS: fn() -> ProtocolExtensionFlags =
    || ProtocolExtensionFlags::LTEP;
const DEFAULT_TORRENT_EXTENSIONS: fn() -> ExtensionFactories = || {
    let mut extensions: ExtensionFactories = Vec::new();

    #[cfg(feature = "extension-metadata")]
    extensions.push(|| Box::new(MetadataExtension::new()));
    #[cfg(feature = "extension-pex")]
    extensions.push(|| Box::new(PexExtension::new()));

    extensions
};
const DEFAULT_TORRENT_OPERATIONS: fn() -> TorrentOperations = || {
    vec![
        Box::new(TorrentTrackersOperation::new()),
        Box::new(TorrentConnectPeersOperation::new()),
        Box::new(TorrentMetadataOperation::new()),
        Box::new(TorrentCreatePiecesOperation::new()),
        Box::new(TorrentCreateFilesOperation::new()),
        Box::new(TorrentFileValidationOperation::new()),
    ]
};

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::torrent::fs::DefaultTorrentFileStorage;
    use popcorn_fx_core::available_port;
    use popcorn_fx_core::core::torrents::magnet::Magnet;
    use popcorn_fx_core::testing::read_test_file_to_bytes;
    use std::str::FromStr;
    use std::sync::Arc;
    use tokio::runtime::Runtime;

    /// Create a new torrent instance from the given uri.
    /// The uri can either be a [Magnet] uri or a filename to a torrent file within the testing resources.
    pub fn create_torrent_from_uri(
        uri: &str,
        temp_dir: &str,
        options: TorrentFlags,
        operations: TorrentOperations,
    ) -> (Torrent, Arc<Runtime>) {
        let torrent_info: TorrentInfo;

        if uri.starts_with("magnet:") {
            let magnet = Magnet::from_str(uri).unwrap();
            torrent_info = TorrentInfo::try_from(magnet).unwrap();
        } else {
            let torrent_info_data = read_test_file_to_bytes(uri);
            torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
        }

        let port = available_port!(6881, 31000).unwrap();

        let runtime = Arc::new(Runtime::new().unwrap());
        (
            Torrent::request()
                .metadata(torrent_info)
                .peer_listener_port(port)
                .options(options)
                .operations(operations)
                .storage(Box::new(DefaultTorrentFileStorage::new(temp_dir)))
                .runtime(runtime.clone())
                .build()
                .unwrap(),
            runtime,
        )
    }
}
