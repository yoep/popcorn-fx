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
    use crate::torrent::peer::tests::new_tcp_peer_discovery;
    use crate::torrent::peer::{
        BitTorrentPeer, PeerDiscovery, PeerId, PeerStream, TcpPeerDiscovery, UtpPeerDiscovery,
    };

    use popcorn_fx_core::testing::read_test_file_to_bytes;
    use std::net::SocketAddr;
    use std::str::FromStr;
    use std::time::Duration;
    use tokio::net::TcpStream;
    use tokio::select;
    use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

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
            crate::torrent::tests::create_torrent_with_default_discoveries(
                $uri,
                $temp_dir,
                $options,
                crate::torrent::TorrentConfig::builder().build(),
                crate::torrent::DEFAULT_TORRENT_OPERATIONS(),
            )
            .await
        };
        ($uri:expr, $temp_dir:expr, $options:expr, $config:expr) => {
            crate::torrent::tests::create_torrent_with_default_discoveries(
                $uri,
                $temp_dir,
                $options,
                $config,
                crate::torrent::DEFAULT_TORRENT_OPERATIONS(),
            )
            .await
        };
        ($uri:expr, $temp_dir:expr, $options:expr, $config:expr, $operations:expr) => {
            crate::torrent::tests::create_torrent_with_default_discoveries(
                $uri,
                $temp_dir,
                $options,
                $config,
                $operations,
            )
            .await
        };
        ($uri:expr, $temp_dir:expr, $options:expr, $config:expr, $operations:expr, $discoveries:expr) => {
            crate::torrent::tests::create_torrent_from_uri(
                $uri,
                $temp_dir,
                $options,
                $config,
                $operations,
                $discoveries,
            )
            .await
        };
    }

    /// Create a new torrent instance from the given uri.
    /// The uri can either be a [Magnet] uri or a filename to a torrent file within the testing resources.
    pub async fn create_torrent_from_uri(
        uri: &str,
        temp_dir: &str,
        options: TorrentFlags,
        config: TorrentConfig,
        operations: Vec<TorrentOperationFactory>,
        discoveries: Vec<Box<dyn PeerDiscovery>>,
    ) -> Torrent {
        let torrent_info = create_metadata(uri);
        let mut request = Torrent::request();
        request
            .metadata(torrent_info)
            .peer_discoveries(discoveries)
            .options(options)
            .config(config)
            .operations(operations.iter().map(|e| e()).collect())
            .storage(Box::new(TorrentFileSystemStorage::new(temp_dir)));
        request.build().unwrap()
    }

    pub async fn create_torrent_with_default_discoveries(
        uri: &str,
        temp_dir: &str,
        options: TorrentFlags,
        config: TorrentConfig,
        operations: Vec<TorrentOperationFactory>,
    ) -> Torrent {
        let tcp_discovery = TcpPeerDiscovery::new()
            .await
            .expect("expected a new tcp peer discovery");
        let utp_discovery = UtpPeerDiscovery::new()
            .await
            .expect("expected a new utp peer discovery");
        let discoveries: Vec<Box<dyn PeerDiscovery>> =
            vec![Box::new(tcp_discovery), Box::new(utp_discovery)];

        create_torrent_from_uri(uri, temp_dir, options, config, operations, discoveries).await
    }

    /// Receive a message from the given receiver, or panic if the timeout is reached.
    #[macro_export]
    macro_rules! recv_timeout {
        ($receiver:expr, $timeout:expr) => {
            crate::torrent::tests::recv_timeout(
                $receiver,
                $timeout,
                "expected to receive an instance",
            )
            .await
        };
        ($receiver:expr, $timeout:expr, $message:expr) => {
            crate::torrent::tests::recv_timeout($receiver, $timeout, $message).await
        };
    }

    /// Receive a message from the given receiver, or panic if the timeout is reached.
    ///
    /// # Arguments
    ///
    /// * `receiver` - The receiver to receive the message from.
    /// * `timeout` - The timeout to wait for the message.
    /// * `message` - The message to print if the timeout is reached.
    ///
    /// # Returns
    ///
    /// It returns the received instance of `T`.
    pub(crate) async fn recv_timeout<T>(
        receiver: &mut UnboundedReceiver<T>,
        timeout: Duration,
        message: &str,
    ) -> T {
        select! {
            _ = tokio::time::sleep(timeout) => panic!("receiver timed-out, {}", message),
            result = receiver.recv() => result.expect(message)
        }
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
            .await
        };
        ($torrent:expr, $protocols:expr) => {
            crate::torrent::tests::create_tcp_peer_pair($torrent, $torrent, $protocols).await
        };
        ($incoming_torrent:expr, $outgoing_torrent:expr, $protocols:expr) => {
            crate::torrent::tests::create_tcp_peer_pair(
                $incoming_torrent,
                $outgoing_torrent,
                $protocols,
            )
            .await
        };
    }

    pub async fn create_tcp_peer_pair(
        incoming_torrent: &Torrent,
        outgoing_torrent: &Torrent,
        protocols: ProtocolExtensionFlags,
    ) -> (BitTorrentPeer, BitTorrentPeer) {
        let incoming_context = incoming_torrent.instance().unwrap();
        let outgoing_context = outgoing_torrent.instance().unwrap();
        let (tx, mut rx) = unbounded_channel();

        let incoming_context = incoming_context.clone();
        let extensions = incoming_context.extensions();
        let listener = new_tcp_peer_discovery().await.unwrap();
        let listener_port = listener.port();
        tokio::spawn(async move {
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
                        )
                        .await,
                    )
                    .unwrap()
                }
            }
        });

        let peer_context = outgoing_context.clone();
        let outgoing_extensions = outgoing_context.extensions();
        let addr = SocketAddr::new([127, 0, 0, 1].into(), listener_port);
        let stream = TcpStream::connect(addr).await.unwrap();
        let outgoing_peer = BitTorrentPeer::new_outbound(
            PeerId::new(),
            addr,
            PeerStream::Tcp(stream),
            peer_context,
            protocols,
            outgoing_extensions,
            Duration::from_secs(5),
        )
        .await
        .expect("expected the outgoing connection to succeed");

        let incoming_peer =
            recv_timeout!(&mut rx, Duration::from_secs(1), "expected an incoming peer")
                .expect("expected an incoming peer");
        (incoming_peer, outgoing_peer)
    }
}
