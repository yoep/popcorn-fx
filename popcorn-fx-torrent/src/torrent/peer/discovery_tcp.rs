use crate::torrent::peer::discovery::PeerDiscovery;
use crate::torrent::peer::extension::Extensions;
use crate::torrent::peer::{
    BitTorrentPeer, Error, Peer, PeerId, PeerStream, ProtocolExtensionFlags, Result,
};
use crate::torrent::TorrentContext;
use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::{select, time};

/// A peer dialer which establishes TCP peer connections.
#[derive(Debug)]
pub struct TcpPeerDiscovery;

impl TcpPeerDiscovery {
    /// Create a new tcp connection peer dialer.
    pub fn new() -> Self {
        Self {}
    }

    /// Try to create a new BitTorrent peer from the given TCP stream.
    async fn create_peer_from_stream(
        peer_id: PeerId,
        peer_addr: SocketAddr,
        stream: TcpStream,
        torrent: Arc<TorrentContext>,
        protocol_extensions: ProtocolExtensionFlags,
        extensions: Extensions,
        connection_timeout: Duration,
    ) -> Result<Box<dyn Peer>> {
        let runtime = torrent.runtime().clone();
        Ok(Box::new(
            BitTorrentPeer::new_outbound(
                peer_id,
                peer_addr,
                PeerStream::Tcp(stream),
                torrent,
                protocol_extensions,
                extensions,
                connection_timeout,
                runtime,
            )
            .await?,
        ))
    }
}

#[async_trait]
impl PeerDiscovery for TcpPeerDiscovery {
    async fn dial(
        &self,
        peer_id: PeerId,
        peer_addr: SocketAddr,
        torrent: Arc<TorrentContext>,
        protocol_extensions: ProtocolExtensionFlags,
        extensions: Extensions,
        connection_timeout: Duration,
    ) -> Result<Box<dyn Peer>> {
        select! {
            _ = time::sleep(connection_timeout) => {
                Err(Error::Io(format!("failed to connect to {}, connection timed out", peer_addr)))
            },
            stream = TcpStream::connect(&peer_addr) =>
                Self::create_peer_from_stream(peer_id, peer_addr, stream?, torrent, protocol_extensions, extensions, connection_timeout).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_torrent;
    use crate::torrent::peer::{PeerState, TcpPeerListener};
    use crate::torrent::{TorrentConfig, TorrentFlags};
    use popcorn_fx_core::{available_port, init_logger};
    use tempfile::tempdir;
    use tokio::runtime::Runtime;

    #[test]
    fn test_tcp_dialer_dial() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let port = available_port!(10000, 11000).unwrap();
        let listener = TcpPeerListener::new(port, runtime.clone()).unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![],
            vec![],
            vec![Box::new(listener)]
        );
        let context = torrent.instance().unwrap();
        let dialer = TcpPeerDiscovery::new();

        let result = runtime
            .block_on(dialer.dial(
                PeerId::new(),
                SocketAddr::from((
                    [127, 0, 0, 1],
                    context.peer_port().expect("expected a peer port"),
                )),
                context.clone(),
                context.protocol_extensions(),
                context.extensions(),
                Duration::from_secs(1),
            ))
            .expect("expected a tcp peer connection to have been established");
        let state = runtime.block_on(result.state());
        assert_ne!(PeerState::Error, state);

        let total_peers = runtime.block_on(context.peer_pool().active_peer_connections());
        assert_eq!(
            1, total_peers,
            "expected the connection to have been established with the torrent listener"
        );
    }
}
