use crate::torrent::peer::extension::Extensions;
use crate::torrent::peer::protocol::{UtpSocket, UtpStream};
use crate::torrent::peer::{
    BitTorrentPeer, Error, Peer, PeerDiscovery, PeerEntry, PeerId, PeerStream,
    ProtocolExtensionFlags, Result,
};
use crate::torrent::TorrentContext;
use async_trait::async_trait;
use derive_more::Display;
use futures::FutureExt;
use fx_handle::Handle;
use log::{debug, trace};
use std::io;
use std::net::SocketAddr;
use std::sync::{mpsc, Arc};
use std::time::Duration;
use tokio::select;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

/// The unique handle of an uTP peer discovery resource instance.
pub type UtpPeerDiscoveryHandle = Handle;

#[derive(Debug, Clone)]
pub struct UtpPeerDiscovery {
    inner: Arc<InnerUtpPeerDiscovery>,
}

impl UtpPeerDiscovery {
    /// Create a new uTP peer discovery instance on the given port.
    pub fn new(port: u16) -> Result<Self> {
        let (tx_ready, rx) = mpsc::channel();
        let cancellation_token = CancellationToken::new();
        let inner = Arc::new(InnerUtpPeerDiscovery {
            handle: Default::default(),
            port,
            sockets: Default::default(),
            connection_timeout: Duration::from_secs(6),
            cancellation_token,
        });

        let inner_main_loop = inner.clone();
        tokio::spawn(async move {
            inner_main_loop.start(tx_ready).await;
        });

        match rx.recv() {
            Ok(ready) => ready?,
            Err(_) => return Err(Error::Closed),
        }

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
        let socket_mutex = self.inner.sockets.read().await;
        let socket = socket_mutex
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
        match self.inner.recv().await {
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
    sockets: RwLock<Vec<UtpSocket>>,
    connection_timeout: Duration,
    cancellation_token: CancellationToken,
}

impl InnerUtpPeerDiscovery {
    /// Start the main loop of the utp peer discovery.
    async fn start(&self, ready_sender: mpsc::Sender<Result<()>>) {
        match Self::try_create_listeners(self.port, self.connection_timeout).await {
            Ok(sockets) => {
                debug!("UTP peer discovery {} started on port {}", self, self.port);
                self.sockets.write().await.extend(sockets);
                self.send(Ok(()), ready_sender);

                loop {
                    select! {
                        _ = self.cancellation_token.cancelled() => break,
                    }
                }

                debug!("UTP peer discovery {} main loop ended", self);
            }
            Err(e) => self.send(Err(e), ready_sender),
        }
    }

    async fn recv(&self) -> Option<UtpStream> {
        let sockets = self.sockets.read().await;

        let mut futures = sockets.iter().map(|e| e.recv());
        if let Some(first) = futures.next().map(|e| e.fuse()) {
            return match first.await {
                Some(stream) => Some(stream),
                None => {
                    debug!("Utp discovery {} no streams available", self);
                    None
                }
            };
        }

        None
    }

    async fn try_create_listeners(port: u16, timeout: Duration) -> Result<Vec<UtpSocket>> {
        let addrs = vec![
            SocketAddr::from(([0, 0, 0, 0], port)),
            SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], port)),
        ];
        let mut sockets = Vec::new();

        for addr in addrs {
            match UtpSocket::new(addr, timeout, vec![]).await {
                Ok(socket) => {
                    trace!("Created uTP socket for {}, {:?}", addr, socket);
                    sockets.push(socket)
                }
                Err(e) => {
                    if let Error::PortUnavailable(_) = e {
                        return Err(e);
                    }

                    debug!("Failed to create uTP socket for {}, {}", addr, e)
                }
            }
        }

        if sockets.is_empty() {
            return Err(Error::Io(io::Error::new(
                io::ErrorKind::Other,
                "no uTP socket created",
            )));
        }

        Ok(sockets)
    }

    fn send(&self, result: Result<()>, ready_sender: mpsc::Sender<Result<()>>) {
        if let Err(e) = ready_sender.send(result) {
            debug!(
                "UTP peer discovery {} failed to send ready state, {}",
                self, e
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::create_torrent;
    use crate::torrent::{TorrentConfig, TorrentFlags};

    use popcorn_fx_core::{available_port, init_logger};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_utp_discovery_new() {
        init_logger!();
        let port = available_port!(31000, 32000).unwrap();

        let result = UtpPeerDiscovery::new(port);
        assert_eq!(
            true,
            result.is_ok(),
            "expected an utp listener, got {:?} instead",
            result
        );

        let result = result.unwrap();
        assert_eq!(port, result.port());
    }

    #[tokio::test]
    async fn test_utp_discovery_dial() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let port = available_port!(11000, 12000).unwrap();
        let discovery = UtpPeerDiscovery::new(port).unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![],
            vec![Box::new(discovery.clone())]
        );
        let context = torrent.instance().unwrap();

        discovery
            .dial(
                PeerId::new(),
                SocketAddr::from(([127, 0, 0, 1], port)),
                context.clone(),
                context.protocol_extensions(),
                context.extensions(),
                Duration::from_secs(2),
            )
            .await
            .expect("expected an utp connection to be established");
    }
}
