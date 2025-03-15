use crate::torrent::peer::discovery::PeerDiscovery;
use crate::torrent::peer::extension::Extensions;
use crate::torrent::peer::{
    BitTorrentPeer, Error, Peer, PeerEntry, PeerId, PeerStream, ProtocolExtensionFlags, Result,
};
use crate::torrent::TorrentContext;
use async_trait::async_trait;
use derive_more::Display;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use fx_handle::Handle;
use log::{debug, trace, warn};
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio::{select, time};
use tokio_util::sync::CancellationToken;

/// The unique handle of an TCP peer discovery resource instance.
pub type TcpPeerDiscoveryHandle = Handle;

/// A peer dialer which establishes TCP peer connections.
#[derive(Debug)]
pub struct TcpPeerDiscovery {
    inner: Arc<InnerTcpPeerDiscovery>,
}

impl TcpPeerDiscovery {
    /// Create a new tcp connection peer dialer.
    pub async fn new(port: u16) -> Result<Self> {
        let (sender, receiver) = unbounded_channel();
        let inner = Arc::new(InnerTcpPeerDiscovery {
            handle: TcpPeerDiscoveryHandle::new(),
            port,
            receiver: Mutex::new(receiver),
            cancellation_token: Default::default(),
        });

        let sockets = InnerTcpPeerDiscovery::try_create_listeners(port).await?;
        let inner_loop = inner.clone();
        tokio::spawn(async move {
            inner_loop.start(sender, sockets).await;
        });

        Ok(Self { inner })
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
        Ok(Box::new(
            BitTorrentPeer::new_outbound(
                peer_id,
                peer_addr,
                PeerStream::Tcp(stream),
                torrent,
                protocol_extensions,
                extensions,
                connection_timeout,
            )
            .await?,
        ))
    }
}

impl Drop for TcpPeerDiscovery {
    fn drop(&mut self) {
        self.inner.cancellation_token.cancel();
    }
}

#[async_trait]
impl PeerDiscovery for TcpPeerDiscovery {
    fn port(&self) -> u16 {
        self.inner.port
    }

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
                Err(Error::Io(io::Error::new(io::ErrorKind::TimedOut, format!("connection with {} timed out", peer_addr))))
            },
            stream = TcpStream::connect(&peer_addr) =>
                Self::create_peer_from_stream(peer_id, peer_addr, stream?, torrent, protocol_extensions, extensions, connection_timeout).await,
        }
    }

    async fn recv(&self) -> Option<PeerEntry> {
        self.inner.receiver.lock().await.recv().await
    }

    fn close(&self) {
        self.inner.cancellation_token.cancel();
    }
}

#[derive(Debug, Display)]
#[display(fmt = "{} (port {})", handle, port)]
struct InnerTcpPeerDiscovery {
    handle: TcpPeerDiscoveryHandle,
    port: u16,
    receiver: Mutex<UnboundedReceiver<PeerEntry>>,
    cancellation_token: CancellationToken,
}

impl InnerTcpPeerDiscovery {
    async fn start(&self, sender: UnboundedSender<PeerEntry>, sockets: Vec<TcpListener>) {
        debug!("TCP peer discovery {} started on port {}", self, self.port);
        let mut futures = FuturesUnordered::from_iter(
            sockets
                .into_iter()
                .map(|socket| self.accept_connections(socket, sender.clone())),
        )
        .fuse();
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(_) = futures.next() => {},
            }
        }

        debug!("TCP peer discovery {} has stopped", self);
    }

    async fn accept_connections(&self, socket: TcpListener, sender: UnboundedSender<PeerEntry>) {
        while let Ok((stream, socket_addr)) = socket.accept().await {
            trace!(
                "TCP peer discovery {} received connection from {}",
                self,
                socket_addr
            );
            if let Err(e) = sender.send(PeerEntry {
                socket_addr,
                stream: PeerStream::Tcp(stream),
            }) {
                warn!(
                    "TCP peer discovery {} failed to send peer connection, {}",
                    self, e
                );
                break;
            }
        }
    }

    async fn try_create_listeners(port: u16) -> Result<Vec<TcpListener>> {
        let addrs = vec![
            SocketAddr::from(([0, 0, 0, 0], port)),
            SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], port)),
        ];
        let mut sockets = Vec::new();

        for addr in addrs {
            match TcpListener::bind(addr).await {
                Ok(socket) => {
                    trace!("Created TCP listener for {}, {:?}", addr, socket);
                    sockets.push(socket)
                }
                Err(e) => {
                    if let io::ErrorKind::AddrInUse = e.kind() {
                        return Err(Error::Io(e));
                    }
                    debug!("Failed to create TCP socket for {}, {}", addr, e)
                }
            }
        }

        if sockets.is_empty() {
            return Err(Error::Io(io::Error::new(
                io::ErrorKind::Other,
                "no TCP listeners created",
            )));
        }

        Ok(sockets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::torrent::peer::PeerState;
    use crate::torrent::{TorrentConfig, TorrentFlags};
    use crate::{create_torrent, recv_timeout};

    use popcorn_fx_core::{available_port, init_logger};
    use tempfile::tempdir;
    use tokio::sync::mpsc::channel;

    #[tokio::test]
    async fn test_tcp_discovery_dial() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let port = available_port!(10000, 11000).unwrap();
        let listener = TcpPeerDiscovery::new(port).await.unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![],
            vec![Box::new(listener)]
        );
        let context = torrent.instance().unwrap();
        let port = available_port!(11000, 12000).unwrap();
        let dialer = TcpPeerDiscovery::new(port).await.unwrap();

        let result = dialer
            .dial(
                PeerId::new(),
                SocketAddr::from((
                    [127, 0, 0, 1],
                    context.peer_port().expect("expected a peer port"),
                )),
                context.clone(),
                context.protocol_extensions(),
                context.extensions(),
                Duration::from_secs(1),
            )
            .await
            .expect("expected a tcp peer connection to have been established");
        let state = result.state().await;
        assert_ne!(PeerState::Error, state);

        let total_peers = context.peer_pool().active_peer_connections().await;
        assert_eq!(
            1, total_peers,
            "expected the connection to have been established with the torrent listener"
        );
    }

    #[tokio::test]
    async fn test_tcp_discovery_port() {
        init_logger!();
        let expected_port = available_port!(31000, 32000).unwrap();
        let listener = TcpPeerDiscovery::new(expected_port).await.unwrap();

        let result = listener.port();

        assert_eq!(expected_port, result);
    }

    #[tokio::test]
    async fn test_tcp_discovery_recv() {
        init_logger!();
        let (tx, mut rx) = channel(1);
        let port = available_port!(31000, 32000).unwrap();
        let listener = TcpPeerDiscovery::new(port).await.unwrap();

        tokio::spawn(async move {
            if let Some(entry) = listener.recv().await {
                tx.send(entry).await.unwrap();
            }
        });

        let addr: SocketAddr = ([127, 0, 0, 1], port).into();
        TcpStream::connect(addr)
            .await
            .expect("expected the connection to succeed");

        let result = recv_timeout!(
            &mut rx,
            Duration::from_millis(200),
            "expected to receive an incoming connection"
        );
        if let PeerStream::Tcp(_) = result.stream {
        } else {
            assert!(
                false,
                "expected PeerStream::Tcp, but got {:?} instead",
                result
            );
        }
    }

    #[tokio::test]
    async fn test_tcp_discovery_drop() {
        init_logger!();
        let addr: SocketAddr = ([127, 0, 0, 1], 6881).into();
        let listener = TcpPeerDiscovery::new(6881).await.unwrap();

        drop(listener);
        time::sleep(Duration::from_millis(100)).await;

        let result = TcpStream::connect(addr).await;
        match result {
            Err(e) => {
                assert_eq!(io::ErrorKind::ConnectionRefused, e.kind());
            }
            Ok(_) => assert!(false, "expected the peer listener to have been closed"),
        }
    }
}
