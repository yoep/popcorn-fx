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
    /// Start the main loop of the tcp peer listener.
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
    use rand::{rng, Rng};

    use crate::torrent::peer::PeerState;
    use crate::torrent::{TorrentConfig, TorrentFlags};
    use crate::{create_torrent, recv_timeout};

    use popcorn_fx_core::{available_port, init_logger};
    use tempfile::tempdir;

    // FIXME: unstable in Github actions
    #[ignore]
    #[tokio::test]
    async fn test_tcp_discovery_dial() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let listener = new_tcp_peer_discovery()
            .await
            .expect("expected a new tcp peer listener");
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![],
            vec![Box::new(listener)]
        );
        let context = torrent.instance().expect("expected a torrent context");
        let listener_port = context
            .peer_port()
            .expect("expected a torrent peer listener port");
        let dialer = new_tcp_peer_discovery()
            .await
            .expect("expected a new tcp peer dialer");

        let result = dialer
            .dial(
                PeerId::new(),
                SocketAddr::from(([127, 0, 0, 1], listener_port)),
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

    // FIXME: unstable in Github actions
    #[ignore]
    #[tokio::test]
    async fn test_tcp_discovery_port() {
        init_logger!();
        let listener = new_tcp_peer_discovery().await.unwrap();

        let result = listener.port();

        assert_eq!(listener.inner.port, result);
    }

    // FIXME: unstable in Github actions
    #[ignore]
    #[tokio::test]
    async fn test_tcp_discovery_recv() {
        init_logger!();
        let (tx, mut rx) = unbounded_channel();
        let listener = new_tcp_peer_discovery().await.unwrap();
        let port = listener.port();

        tokio::spawn(async move {
            if let Some(entry) = listener.recv().await {
                tx.send(entry).unwrap();
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

    // FIXME: unstable in Github actions
    #[ignore]
    #[tokio::test]
    async fn test_tcp_discovery_drop() {
        init_logger!();
        let listener = new_tcp_peer_discovery().await.unwrap();
        let addr: SocketAddr = ([127, 0, 0, 1], listener.port()).into();

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

    async fn new_tcp_peer_discovery() -> Result<TcpPeerDiscovery> {
        let mut attempts = 0;
        let mut port = available_port!(rng().random_range(10000..12000), 20000).unwrap();

        while attempts < 5 {
            return match TcpPeerDiscovery::new(port).await {
                Ok(e) => Ok(e),
                Err(e) => {
                    if let Error::Io(io_err) = &e {
                        if io_err.kind() == io::ErrorKind::AddrInUse {
                            attempts += 1;
                            port += 1;
                            continue;
                        }
                    }

                    Err(e)
                }
            };
        }

        Err(Error::PortUnavailable(port))
    }
}
