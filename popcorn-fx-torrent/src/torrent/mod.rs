pub use compact::*;
pub use errors::*;
pub use file::*;
pub use info_hash::*;
pub use magnet::*;
pub use piece::*;
pub use session::*;
use std::net::{SocketAddr, TcpListener};
pub use torrent::*;
pub use torrent_health::*;
pub use torrent_metadata::*;

mod compact;
mod errors;
mod file;
pub mod fs;
mod info_hash;
mod magnet;
mod merkle;
pub mod operation;
pub mod peer;
mod peer_pool;
mod piece;
mod session;
mod torrent;
mod torrent_health;
mod torrent_metadata;
mod tracker;

use crate::torrent::operation::{
    TorrentConnectPeersOperation, TorrentCreateFilesOperation, TorrentCreatePiecesOperation,
    TorrentFileValidationOperation, TorrentMetadataOperation, TorrentTrackersOperation,
};
#[cfg(feature = "extension-donthave")]
use crate::torrent::peer::extension::donthave::DontHaveExtension;
#[cfg(feature = "extension-metadata")]
use crate::torrent::peer::extension::metadata::MetadataExtension;
#[cfg(feature = "extension-pex")]
use crate::torrent::peer::extension::pex::PexExtension;
use crate::torrent::peer::ProtocolExtensionFlags;

const DEFAULT_TORRENT_PROTOCOL_EXTENSIONS: fn() -> ProtocolExtensionFlags = || {
    ProtocolExtensionFlags::LTEP | ProtocolExtensionFlags::Fast | ProtocolExtensionFlags::SupportV2
};
const DEFAULT_TORRENT_EXTENSIONS: fn() -> ExtensionFactories = || {
    let mut extensions: ExtensionFactories = Vec::new();

    #[cfg(feature = "extension-metadata")]
    extensions.push(|| Box::new(MetadataExtension::new()));
    #[cfg(feature = "extension-pex")]
    extensions.push(|| Box::new(PexExtension::new()));
    #[cfg(feature = "extension-donthave")]
    extensions.push(|| Box::new(DontHaveExtension::new()));

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

/// Retrieves an available port on the local machine.
///
/// This function searches for an available port on all network interfaces at the time of invocation.
/// However, it's important to note that while a port may be available when retrieved, it may become
/// unavailable by the time you attempt to bind to it, as this function does not reserve the port.
///
/// # Arguments
///
/// * `lower_bound` - The lower bound of the available port range.
/// * `upper_bound` - The upper bound of the available port range.
///
/// # Returns
///
/// Returns an available port if one is found, else `None`.
pub(crate) fn available_port(lower_bound: u16, upper_bound: u16) -> Option<u16> {
    let supported_ports: Vec<u16> = (lower_bound..=upper_bound).collect();

    for port in supported_ports {
        let socket: SocketAddr = ([0, 0, 0, 0], port).into();
        if TcpListener::bind(socket).is_ok() {
            return Some(port);
        }
    }

    None
}

/// Retrieves an available port on the local machine.
///
/// This function searches for an available port on all network interfaces at the time of invocation.
/// However, it's important to note that while a port may be available when retrieved, it may become
/// unavailable by the time you attempt to bind to it, as this function does not reserve the port.
///
/// # Arguments
///
/// * `lower_bound` - The lower bound of the available port range (optional, default = 1000).
/// * `upper_bound` - The upper bound of the available port range (optional, default = [u16::MAX]).
///
/// # Returns
///
/// Returns an available port if one is found, else `None`.
#[macro_export]
macro_rules! available_port {
    ($lower_bound:expr, $upper_bound:expr) => {
        crate::torrent::available_port($lower_bound, $upper_bound)
    };
    ($lower_bound:expr) => {
        crate::torrent::available_port($lower_bound, u16::MAX)
    };
    () => {
        crate::torrent::available_port(1000, u16::MAX)
    };
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::torrent::fs::TorrentFileSystemStorage;
    use crate::torrent::peer::{
        BitTorrentPeer, PeerDiscovery, PeerId, PeerListener, PeerStream, TcpPeerDiscovery,
        TcpPeerListener, UtpPeerDiscovery,
    };
    use popcorn_fx_core::testing::read_test_file_to_bytes;
    use rand::{rng, Rng};
    use std::net::SocketAddr;
    use std::str::FromStr;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::net::TcpStream;
    use tokio::runtime::Runtime;

    /// Create the torrent metadata from the given uri.
    /// The uri can either point to a `.torrent` file or a magnet link.
    pub fn create_metadata(uri: &str) -> TorrentMetadata {
        if uri.starts_with("magnet:") {
            let magnet = Magnet::from_str(uri).unwrap();
            TorrentMetadata::try_from(magnet).unwrap()
        } else {
            let torrent_info_data = read_test_file_to_bytes(uri);
            TorrentMetadata::try_from(torrent_info_data.as_slice()).unwrap()
        }
    }

    #[macro_export]
    macro_rules! create_torrent {
        ($uri:expr, $temp_dir:expr, $options:expr) => {
            crate::torrent::tests::create_torrent_with_default_dialers_and_listeners(
                $uri,
                $temp_dir,
                $options,
                crate::torrent::TorrentConfig::builder().build(),
                crate::torrent::DEFAULT_TORRENT_OPERATIONS(),
            )
        };
        ($uri:expr, $temp_dir:expr, $options:expr, $config:expr) => {
            crate::torrent::tests::create_torrent_with_default_dialers_and_listeners(
                $uri,
                $temp_dir,
                $options,
                $config,
                crate::torrent::DEFAULT_TORRENT_OPERATIONS(),
            )
        };
        ($uri:expr, $temp_dir:expr, $options:expr, $config:expr, $operations:expr) => {
            crate::torrent::tests::create_torrent_with_default_dialers_and_listeners(
                $uri,
                $temp_dir,
                $options,
                $config,
                $operations,
            )
        };
        ($uri:expr, $temp_dir:expr, $options:expr, $config:expr, $operations:expr, $dialers:expr) => {
            crate::torrent::tests::create_torrent_with_default_listeners(
                $uri,
                $temp_dir,
                $options,
                $config,
                $operations,
                $dialers,
            )
        };
        ($uri:expr, $temp_dir:expr, $options:expr, $config:expr, $operations:expr, $dialers:expr, $listeners:expr) => {
            crate::torrent::tests::create_torrent_from_uri(
                $uri,
                $temp_dir,
                $options,
                $config,
                $operations,
                $dialers,
                $listeners,
                std::sync::Arc::new(tokio::runtime::Runtime::new().unwrap()),
            )
        };
        ($uri:expr, $temp_dir:expr, $options:expr, $config:expr, $operations:expr, $dialers:expr, $listeners:expr, $runtime:expr) => {
            crate::torrent::tests::create_torrent_from_uri(
                $uri,
                $temp_dir,
                $options,
                $config,
                $operations,
                $dialers,
                $listeners,
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
        config: TorrentConfig,
        operations: Vec<TorrentOperationFactory>,
        discoveries: Vec<Box<dyn PeerDiscovery>>,
        listeners: Vec<Box<dyn PeerListener>>,
        runtime: Arc<Runtime>,
    ) -> Torrent {
        let torrent_info = create_metadata(uri);

        Torrent::request()
            .metadata(torrent_info)
            .peer_dialers(discoveries)
            .peer_listeners(listeners)
            .options(options)
            .config(config)
            .operations(operations.iter().map(|e| e()).collect())
            .storage(Box::new(TorrentFileSystemStorage::new(temp_dir)))
            .runtime(runtime)
            .build()
            .unwrap()
    }

    pub fn create_torrent_with_default_listeners(
        uri: &str,
        temp_dir: &str,
        options: TorrentFlags,
        config: TorrentConfig,
        operations: Vec<TorrentOperationFactory>,
        discoveries: Vec<Box<dyn PeerDiscovery>>,
    ) -> Torrent {
        let runtime = Arc::new(Runtime::new().unwrap());
        let listeners = default_listeners(runtime.clone());

        create_torrent_from_uri(
            uri,
            temp_dir,
            options,
            config,
            operations,
            discoveries,
            listeners,
            runtime,
        )
    }

    pub fn create_torrent_with_default_dialers_and_listeners(
        uri: &str,
        temp_dir: &str,
        options: TorrentFlags,
        config: TorrentConfig,
        operations: Vec<TorrentOperationFactory>,
    ) -> Torrent {
        let runtime = Arc::new(Runtime::new().unwrap());
        let mut rng = rng();
        let tcp_port_start = rng.random_range(6881..10000);
        let utp_port_start = rng.random_range(6881..10000);
        let utp_discovery = UtpPeerDiscovery::new(
            available_port(utp_port_start, 31000).unwrap(),
            runtime.clone(),
        )
        .unwrap();
        let listeners: Vec<Box<dyn PeerListener>> = vec![
            Box::new(
                TcpPeerListener::new(
                    available_port(tcp_port_start, 31000).unwrap(),
                    runtime.clone(),
                )
                .unwrap(),
            ),
            Box::new(utp_discovery.clone()),
        ];
        let dialers: Vec<Box<dyn PeerDiscovery>> =
            vec![Box::new(TcpPeerDiscovery::new()), Box::new(utp_discovery)];

        create_torrent_from_uri(
            uri, temp_dir, options, config, operations, dialers, listeners, runtime,
        )
    }

    #[macro_export]
    macro_rules! create_peer_pair {
        ($torrent:expr) => {
            crate::torrent::tests::create_tcp_peer_pair(
                $torrent,
                $torrent,
                $torrent
                    .instance()
                    .expect("expected a valid torrent context")
                    .protocol_extensions()
                    .clone(),
            )
        };
        ($torrent:expr, $protocols:expr) => {
            crate::torrent::tests::create_tcp_peer_pair($torrent, $torrent, $protocols)
        };
        ($incoming_torrent:expr, $outgoing_torrent:expr, $protocols:expr) => {
            crate::torrent::tests::create_tcp_peer_pair(
                $incoming_torrent,
                $outgoing_torrent,
                $protocols,
            )
        };
    }

    pub fn create_tcp_peer_pair(
        incoming_torrent: &Torrent,
        outgoing_torrent: &Torrent,
        protocols: ProtocolExtensionFlags,
    ) -> (BitTorrentPeer, BitTorrentPeer) {
        let incoming_context = incoming_torrent.instance().unwrap();
        let outgoing_context = outgoing_torrent.instance().unwrap();
        let incoming_runtime = incoming_context.runtime();
        let outgoing_runtime = outgoing_context.runtime();
        let port_start = rng().random_range(6881..10000);
        let port = available_port!(port_start, 31000).unwrap();
        let (tx, rx) = std::sync::mpsc::channel();

        let incoming_context = incoming_context.clone();
        let extensions = incoming_context.extensions();
        let incoming_runtime_thread = incoming_runtime.clone();
        let mut listener = TcpPeerListener::new(port, incoming_runtime_thread.clone()).unwrap();
        incoming_runtime.spawn(async move {
            if let Some(peer) = listener.recv().await {
                if let PeerStream::Tcp(stream) = peer.stream {
                    tx.send(
                        BitTorrentPeer::new_inbound(
                            PeerId::new(),
                            peer.socket_addr,
                            PeerStream::Tcp(stream),
                            incoming_context,
                            protocols.clone(),
                            extensions,
                            Duration::from_secs(5),
                            incoming_runtime_thread,
                        )
                        .await,
                    )
                    .unwrap()
                }
            }
        });

        let peer_context = outgoing_context.clone();
        let outgoing_extensions = outgoing_context.extensions();
        let addr = SocketAddr::new([127, 0, 0, 1].into(), port);
        let stream = incoming_runtime.block_on(TcpStream::connect(addr)).unwrap();
        let outgoing_peer = incoming_runtime
            .block_on(BitTorrentPeer::new_outbound(
                PeerId::new(),
                addr,
                PeerStream::Tcp(stream),
                peer_context,
                protocols,
                outgoing_extensions,
                Duration::from_secs(5),
                outgoing_runtime.clone(),
            ))
            .expect("expected the outgoing connection to succeed");

        let incoming_peer = rx
            .recv_timeout(Duration::from_secs(1))
            .expect("expected an incoming peer")
            .expect("expected the incoming connection to succeed");

        (incoming_peer, outgoing_peer)
    }

    /// Create the default test peer listeners
    pub fn default_listeners(runtime: Arc<Runtime>) -> Vec<Box<dyn PeerListener>> {
        let mut rng = rng();
        let tcp_port_start = rng.random_range(6881..10000);
        let utp_port_start = rng.random_range(6881..10000);

        vec![
            Box::new(
                TcpPeerListener::new(
                    available_port!(tcp_port_start, 31000).unwrap(),
                    runtime.clone(),
                )
                .unwrap(),
            ),
            Box::new(
                UtpPeerDiscovery::new(available_port!(utp_port_start, 31000).unwrap(), runtime)
                    .unwrap(),
            ),
        ]
    }
}
