use crate::torrent::peer::extension::Extensions;
use crate::torrent::peer::protocol::UtpSocket;
use crate::torrent::peer::{
    Error, Peer, PeerDiscovery, PeerEntry, PeerId, PeerListener, ProtocolExtensionFlags, Result,
};
use crate::torrent::TorrentContext;
use async_trait::async_trait;
use derive_more::Display;
use log::{debug, trace};
use popcorn_fx_core::core::Handle;
use std::net::SocketAddr;
use std::sync::{mpsc, Arc};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::select;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

/// The unique handle of an uTP peer discovery resource instance.
pub type UtpPeerDiscoveryHandle = Handle;

#[derive(Debug, Clone)]
pub struct UtpPeerDialerListener {
    inner: Arc<InnerUtpPeerDiscovery>,
}

impl UtpPeerDialerListener {
    /// Create a new uTP peer discovery instance on the given port.
    pub fn new(port: u16, runtime: Arc<Runtime>) -> Result<Self> {
        let (tx_ready, rx) = mpsc::channel();
        let cancellation_token = CancellationToken::new();
        let inner = Arc::new(InnerUtpPeerDiscovery {
            handle: Default::default(),
            port,
            sockets: Default::default(),
            connection_timeout: Duration::from_secs(6),
            cancellation_token,
            runtime,
        });

        let inner_main_loop = inner.clone();
        inner.runtime.spawn(async move {
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
impl PeerListener for UtpPeerDialerListener {
    fn port(&self) -> u16 {
        self.inner.port
    }

    async fn recv(&mut self) -> Option<PeerEntry> {
        None
    }

    fn close(&self) {
        self.inner.cancellation_token.cancel();
    }
}

#[async_trait]
impl PeerDiscovery for UtpPeerDialerListener {
    async fn dial(
        &self,
        peer_id: PeerId,
        peer_addr: SocketAddr,
        torrent: Arc<TorrentContext>,
        protocol_extensions: ProtocolExtensionFlags,
        extensions: Extensions,
        connection_timeout: Duration,
    ) -> crate::torrent::peer::Result<Box<dyn Peer>> {
        let socket_mutex = self.inner.sockets.read().await;
        let socket = socket_mutex
            .iter()
            .find(|e| e.addr().is_ipv4() == peer_addr.is_ipv4());

        if let Some(socket) = socket {
            socket.connect(peer_addr).await?;

            return Err(Error::Io("not yet implemented".to_string()));
        }

        Err(Error::Io(format!(
            "support for address \"{}\" has been disabled",
            peer_addr
        )))
    }
}

impl Drop for UtpPeerDialerListener {
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
    runtime: Arc<Runtime>,
}

impl InnerUtpPeerDiscovery {
    /// Start the main loop of the utp peer discovery.
    async fn start(&self, ready_sender: mpsc::Sender<Result<()>>) {
        match Self::try_create_listeners(self.port, self.connection_timeout, &self.runtime).await {
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

    async fn try_create_listeners(
        port: u16,
        timeout: Duration,
        runtime: &Arc<Runtime>,
    ) -> Result<Vec<UtpSocket>> {
        let addrs = vec![
            SocketAddr::from(([0, 0, 0, 0], port)),
            SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], port)),
        ];
        let mut sockets = Vec::new();

        for addr in addrs {
            match UtpSocket::new(addr, timeout, runtime.clone()).await {
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
            return Err(Error::Io("failed to create any uTP socket".to_string()));
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

    #[test]
    fn test_utp_discovery_new() {
        init_logger!();
        let port = available_port!(31000, 32000).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());

        let result = UtpPeerDialerListener::new(port, runtime);
        assert_eq!(
            true,
            result.is_ok(),
            "expected an utp listener, got {:?} instead",
            result
        );

        let result = result.unwrap();
        assert_eq!(port, result.port());
    }

    #[test]
    fn test_utp_discovery_dial() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let port = available_port!(11000, 12000).unwrap();
        let discovery = UtpPeerDialerListener::new(port, runtime.clone()).unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![],
            vec![],
            vec![Box::new(discovery.clone())]
        );
        let context = torrent.instance().unwrap();

        runtime
            .block_on(discovery.dial(
                PeerId::new(),
                SocketAddr::from(([127, 0, 0, 1], port)),
                context.clone(),
                context.protocol_extensions(),
                context.extensions(),
                Duration::from_secs(2),
            ))
            .expect("expected an utp connection to be established");
    }
}
