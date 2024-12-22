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
pub use torrent_metadata::*;

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
mod torrent_metadata;
mod tracker;

const DEFAULT_TORRENT_PROTOCOL_EXTENSIONS: fn() -> ProtocolExtensionFlags =
    || ProtocolExtensionFlags::LTEP | ProtocolExtensionFlags::Fast;
const DEFAULT_TORRENT_EXTENSIONS: fn() -> ExtensionFactories = || {
    let mut extensions: ExtensionFactories = Vec::new();

    #[cfg(feature = "extension-metadata")]
    extensions.push(|| Box::new(MetadataExtension::new()));
    #[cfg(feature = "extension-pex")]
    extensions.push(|| Box::new(PexExtension::new()));

    extensions
};
/// The default operations applied to a torrent.
/// These include the necessary chain of actions to be executed during the torrent lifecycle.
const DEFAULT_TORRENT_OPERATIONS: fn() -> Vec<TorrentOperationFactory> = || {
    vec![
        || Box::new(TorrentTrackersOperation::new()),
        || Box::new(TorrentConnectPeersOperation::new()),
        || Box::new(TorrentMetadataOperation::new()),
        || Box::new(TorrentCreatePiecesOperation::new()),
        || Box::new(TorrentCreateFilesOperation::new()),
        || Box::new(TorrentFileValidationOperation::new()),
    ]
};

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::torrent::fs::DefaultTorrentFileStorage;
    use crate::torrent::peer::{DefaultPeerListener, PeerId, PeerListener, TcpPeer};
    use popcorn_fx_core::available_port;
    use popcorn_fx_core::core::torrents::magnet::Magnet;
    use popcorn_fx_core::testing::read_test_file_to_bytes;
    use rand::{thread_rng, Rng};
    use std::net::SocketAddr;
    use std::str::FromStr;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::runtime::Runtime;

    #[macro_export]
    macro_rules! create_torrent {
        ($uri:expr, $temp_dir:expr, $options:expr) => {
            crate::torrent::tests::create_torrent_from_uri(
                $uri,
                $temp_dir,
                $options,
                crate::torrent::DEFAULT_TORRENT_OPERATIONS(),
                std::sync::Arc::new(tokio::runtime::Runtime::new().unwrap()),
            )
        };
        ($uri:expr, $temp_dir:expr, $options:expr, $operations:expr) => {
            crate::torrent::tests::create_torrent_from_uri(
                $uri,
                $temp_dir,
                $options,
                $operations,
                std::sync::Arc::new(tokio::runtime::Runtime::new().unwrap()),
            )
        };
        ($uri:expr, $temp_dir:expr, $options:expr, $operations:expr, $runtime:expr) => {
            crate::torrent::tests::create_torrent_from_uri(
                $uri,
                $temp_dir,
                $options,
                $operations,
                $runtime,
            )
        };
    }

    /// Create a new torrent instance from the given uri.
    /// The uri can either be a [Magnet] uri or a filename to a torrent file within the testing resources.
    pub fn create_torrent_from_uri(
        uri: &str,
        temp_dir: &str,
        options: TorrentFlags,
        operations: Vec<TorrentOperationFactory>,
        runtime: Arc<Runtime>,
    ) -> Torrent {
        let torrent_info: TorrentMetadata;

        if uri.starts_with("magnet:") {
            let magnet = Magnet::from_str(uri).unwrap();
            torrent_info = TorrentMetadata::try_from(magnet).unwrap();
        } else {
            let torrent_info_data = read_test_file_to_bytes(uri);
            torrent_info = TorrentMetadata::try_from(torrent_info_data.as_slice()).unwrap();
        }

        let port_start = thread_rng().gen_range(6881..10000);
        Torrent::request()
            .metadata(torrent_info)
            .peer_listener_port(available_port!(port_start, 31000).unwrap())
            .options(options)
            .operations(operations)
            .storage(Box::new(DefaultTorrentFileStorage::new(temp_dir)))
            .runtime(runtime)
            .build()
            .unwrap()
    }

    #[macro_export]
    macro_rules! create_peer_pair {
        ($torrent:expr) => {
            crate::torrent::tests::create_peer_pair(
                $torrent,
                $torrent
                    .instance()
                    .expect("expected a valid torrent context")
                    .protocol_extensions()
                    .clone(),
            )
        };
        ($torrent:expr, $protocols:expr) => {
            crate::torrent::tests::create_peer_pair($torrent, $protocols)
        };
    }

    pub fn create_peer_pair(
        torrent: &Torrent,
        protocols: ProtocolExtensionFlags,
    ) -> (TcpPeer, TcpPeer) {
        let context = torrent.instance().unwrap();
        let runtime = context.runtime();
        let port = available_port!(6881, 31000).unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let extensions = context.extensions();

        let incoming_context = context.clone();
        let incoming_runtime = runtime.clone();
        let incoming_extensions = context.extensions();
        let mut listener = DefaultPeerListener::new(port, runtime.clone()).unwrap();
        runtime.spawn(async move {
            if let Some(peer) = listener.recv().await {
                tx.send(
                    TcpPeer::new_inbound(
                        PeerId::new(),
                        peer.socket_addr,
                        peer.stream,
                        incoming_context,
                        protocols.clone(),
                        incoming_extensions,
                        Duration::from_secs(5),
                        incoming_runtime,
                    )
                    .await,
                )
                .unwrap();
            }
        });

        let peer_context = context.clone();
        let outgoing_peer = runtime
            .block_on(TcpPeer::new_outbound(
                PeerId::new(),
                SocketAddr::new([127, 0, 0, 1].into(), port),
                peer_context,
                protocols,
                extensions,
                Duration::from_secs(5),
                runtime.clone(),
            ))
            .expect("expected the outgoing connection to succeed");

        let incoming_peer = rx
            .recv_timeout(Duration::from_secs(1))
            .expect("expected an incoming peer")
            .expect("expected the incoming connection to succeed");

        (outgoing_peer, incoming_peer)
    }
}
