use crate::torrent::peer::extension::Extensions;
use crate::torrent::peer::protocol::{UtpSocket, UtpStream};
use crate::torrent::peer::{
    BitTorrentPeer, Error, Peer, PeerDiscovery, PeerEntry, PeerId, PeerStream,
    ProtocolExtensionFlags, Result,
};
use crate::torrent::TorrentContext;
use async_trait::async_trait;
use derive_more::Display;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use fx_handle::Handle;
use log::{debug, trace, warn};
use std::io;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

/// The unique handle of an uTP peer discovery resource instance.
pub type UtpPeerDiscoveryHandle = Handle;

#[derive(Debug, Clone)]
pub struct UtpPeerDiscovery {
    inner: Arc<InnerUtpPeerDiscovery>,
}

impl UtpPeerDiscovery {
    /// Create a new uTP peer discovery instance.
    ///
    /// It will listen on a random port assigned by the OS.
    /// If you want to listen on a specific port, use [UtpPeerDiscovery::new_with_port] instead.
    ///
    /// # Returns
    ///
    /// It returns a new uTP peer discovery instance, else an error when the listener couldn't be bound.
    pub async fn new() -> Result<Self> {
        Self::new_with_port(0).await
    }

    /// Create a new uTP peer discovery instance.
    ///
    /// It will listen on the given port.
    /// If the port is already in use, it will return [Error::Io].
    ///
    /// # Returns
    ///
    /// It returns a new uTP peer discovery instance, else an error when the listener couldn't be bound.
    pub async fn new_with_port(port: u16) -> Result<Self> {
        let (sender, receiver) = unbounded_channel();
        let cancellation_token = CancellationToken::new();
        let sockets =
            InnerUtpPeerDiscovery::try_binding_sockets(port, Duration::from_secs(6)).await?;
        let port = sockets
            .get(0)
            .map(|e| e.addr().port())
            .ok_or(Error::Io(io::Error::new(
                io::ErrorKind::Other,
                "unable to get bound socket port",
            )))?;
        let inner = Arc::new(InnerUtpPeerDiscovery {
            handle: Default::default(),
            port,
            sockets,
            receiver: Mutex::new(receiver),
            connection_timeout: Duration::from_secs(6),
            cancellation_token,
        });

        let inner_main_loop = inner.clone();
        tokio::spawn(async move {
            inner_main_loop.start(sender).await;
        });

        Ok(Self { inner })
    }
}

#[async_trait]
impl PeerDiscovery for UtpPeerDiscovery {
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
        let socket = self
            .inner
            .sockets
            .iter()
            .find(|e| e.addr().is_ipv4() == peer_addr.is_ipv4());

        if let Some(socket) = socket {
            let stream = socket.connect(peer_addr).await?;

            return Ok(Box::new(
                BitTorrentPeer::new_outbound(
                    peer_id,
                    peer_addr,
                    PeerStream::Utp(stream),
                    torrent,
                    protocol_extensions,
                    extensions,
                    connection_timeout,
                )
                .await?,
            ));
        }

        Err(Error::Io(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("support for address \"{}\" has been disabled", peer_addr),
        )))
    }

    async fn recv(&self) -> Option<PeerEntry> {
        let mut receiver = self.inner.receiver.lock().await;
        match receiver.recv().await {
            None => None,
            Some(stream) => Some(PeerEntry {
                socket_addr: stream.addr(),
                stream: PeerStream::Utp(stream),
            }),
        }
    }

    fn close(&self) {
        self.inner.cancellation_token.cancel();
    }
}

impl Drop for UtpPeerDiscovery {
    fn drop(&mut self) {
        self.close();
    }
}

#[derive(Debug, Display)]
#[display(fmt = "{} (port {})", handle, port)]
struct InnerUtpPeerDiscovery {
    handle: UtpPeerDiscoveryHandle,
    port: u16,
    sockets: Vec<UtpSocket>,
    receiver: Mutex<UnboundedReceiver<UtpStream>>,
    connection_timeout: Duration,
    cancellation_token: CancellationToken,
}

impl InnerUtpPeerDiscovery {
    /// Start the main loop of the utp peer discovery.
    async fn start(&self, sender: UnboundedSender<UtpStream>) {
        debug!("UTP peer discovery {} started on port {}", self, self.port);
        let mut futures = FuturesUnordered::from_iter(
            self.sockets
                .iter()
                .map(|socket| self.accept_connections(socket, sender.clone())),
        )
        .fuse();
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(_) = futures.next() => {},
            }
        }

        debug!("UTP peer discovery {} main loop ended", self);
    }

    async fn accept_connections(&self, socket: &UtpSocket, sender: UnboundedSender<UtpStream>) {
        while let Some(stream) = socket.recv().await {
            if let Err(e) = sender.send(stream) {
                warn!(
                    "UTP peer discovery {} failed to send peer connection, {}",
                    self, e
                );
                break;
            }
        }
    }

    async fn try_binding_sockets(mut port: u16, timeout: Duration) -> Result<Vec<UtpSocket>> {
        let mut sockets = Vec::new();

        // attempt to bind the IPv6 address first in case dual stack is enabled
        let ipv6_addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, port));
        match UtpSocket::new(ipv6_addr, timeout, vec![]).await {
            Ok(socket) => {
                trace!("Created uTP IPv6 listener on {}", ipv6_addr);
                port = socket.addr().port();
                sockets.push(socket);
            }
            Err(e) => {
                if let Error::Io(io_err) = &e {
                    if io_err.kind() == io::ErrorKind::AddrInUse {
                        return Err(e);
                    }
                }

                debug!("Failed to bind uTP IPv6 socket on {}, {}", ipv6_addr, e)
            }
        }

        let ipv4_addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));
        match UtpSocket::new(ipv4_addr, timeout, vec![]).await {
            Ok(socket) => {
                trace!("Created uTP IPv4 listener on {}", ipv4_addr);
                sockets.push(socket);
            }
            Err(e) => {
                if sockets.is_empty() {
                    return Err(e);
                }

                trace!("Failed to bind uTP IPv4 socket on {}, {}", ipv4_addr, e)
            }
        }

        Ok(sockets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::create_torrent;
    use crate::torrent::{TorrentConfig, TorrentFlags};

    use popcorn_fx_core::init_logger;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_utp_discovery_new() {
        init_logger!();

        let utp_discovery = UtpPeerDiscovery::new().await;
        assert_eq!(
            true,
            utp_discovery.is_ok(),
            "expected an utp listener, got {:?} instead",
            utp_discovery
        );

        let result = utp_discovery.unwrap();
        assert_ne!(
            0,
            result.port(),
            "expected a port number to have been assigned"
        );
    }

    #[tokio::test]
    async fn test_utp_discovery_dial() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let listener = UtpPeerDiscovery::new()
            .await
            .expect("expected a new utp peer listener");
        let port = listener.port();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![],
            vec![Box::new(listener.clone())]
        );
        let context = torrent.instance().unwrap();

        let dialer = UtpPeerDiscovery::new()
            .await
            .expect("expected a new utp peer dialer");
        dialer
            .dial(
                PeerId::new(),
                SocketAddr::from((Ipv4Addr::LOCALHOST, port)),
                context.clone(),
                context.protocol_extensions(),
                context.extensions(),
                Duration::from_secs(2),
            )
            .await
            .expect("expected an utp connection to be established");
    }
}
