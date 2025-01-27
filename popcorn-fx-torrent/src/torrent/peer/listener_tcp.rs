use crate::torrent::peer::{Error, PeerEntry, PeerListener, PeerStream};
use async_trait::async_trait;
use log::{debug, trace, warn};
use std::net::SocketAddr;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use tokio::select;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct TcpPeerListener {
    port: u16,
    receiver: UnboundedReceiver<PeerEntry>,
    runtime: Arc<Runtime>,
    cancellation_token: CancellationToken,
}

impl TcpPeerListener {
    /// Create a new peer listener on the given port.
    /// It tries to bind to both ipv4 and ipv6 addresses for the given port.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use tokio::runtime::Runtime;
    /// use popcorn_fx_torrent::torrent::peer::TcpPeerListener;
    ///
    /// fn new_listener() -> TcpPeerListener {
    ///     let runtime = Arc::new(Runtime::new().unwrap());
    ///     let port = 6891;
    ///
    ///     TcpPeerListener::new(port, runtime).unwrap()
    /// }
    ///
    /// ```
    pub fn new(port: u16, runtime: Arc<Runtime>) -> crate::torrent::peer::Result<Self> {
        trace!("Trying to create new peer listener on port {}", port);
        let cancellation_token = CancellationToken::new();
        let (tx_ready, rx_ready) = channel();
        let (sender, receiver) = unbounded_channel();

        let listener_cancellation = cancellation_token.clone();
        runtime
            .spawn(async move { Self::start(port, sender, tx_ready, listener_cancellation).await });

        // wait for the listeners to be ready
        match rx_ready.recv() {
            Ok(ready) => ready?,
            Err(_) => return Err(Error::Closed),
        }

        debug!("Created new peer listener on port {}", port);
        Ok(Self {
            port,
            receiver,
            runtime,
            cancellation_token,
        })
    }

    async fn start(
        port: u16,
        sender: UnboundedSender<PeerEntry>,
        ready_sender: Sender<crate::torrent::peer::Result<()>>,
        cancellation_token: CancellationToken,
    ) {
        let ipv4: TcpListener;
        let ipv6: TcpListener;

        // try to create the tcp listeners
        // if one of them fails, we can't start the listener and respond with an error
        match Self::try_create_listeners(port).await {
            Ok((ipv4_result, ipv6_result)) => {
                ipv4 = ipv4_result;
                ipv6 = ipv6_result;
                ready_sender.send(Ok(())).unwrap();
            }
            Err(e) => {
                ready_sender.send(Err(e)).unwrap();
                return;
            }
        }
        drop(ready_sender);

        loop {
            select! {
                _ = cancellation_token.cancelled() => break,
                Ok((stream, socket)) = ipv4.accept() => Self::accept_incoming_connection(stream, socket, &sender),
                Ok((stream, socket)) = ipv6.accept() => Self::accept_incoming_connection(stream, socket, &sender),
            }
        }

        debug!("Peer listener (port {}) has stopped", port);
    }

    async fn try_create_listeners(
        port: u16,
    ) -> crate::torrent::peer::Result<(TcpListener, TcpListener)> {
        let ipv4 = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port))).await?;
        let ipv6 = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], port))).await?;

        Ok((ipv4, ipv6))
    }

    fn accept_incoming_connection(
        stream: TcpStream,
        socket_addr: SocketAddr,
        sender: &UnboundedSender<PeerEntry>,
    ) {
        trace!("Received incoming peer connection {}", socket_addr);
        if let Err(e) = sender.send(PeerEntry {
            socket_addr,
            stream: PeerStream::Tcp(stream),
        }) {
            warn!("Failed to send peer connection, {}", e);
        }
    }
}

#[async_trait]
impl PeerListener for TcpPeerListener {
    fn port(&self) -> u16 {
        self.port
    }

    async fn recv(&mut self) -> Option<PeerEntry> {
        self.receiver.recv().await
    }

    fn close(&self) {
        trace!("Peer listener (port {}) is being closed", self.port);
        self.cancellation_token.cancel();
    }
}

impl Drop for TcpPeerListener {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use log::error;
    use popcorn_fx_core::{available_port, init_logger};
    use std::sync::mpsc::channel;
    use std::time::Duration;
    use tokio::net::TcpStream;

    use super::*;

    #[test]
    fn test_port() {
        init_logger!();
        let runtime = Arc::new(Runtime::new().unwrap());
        let expected_port = available_port!(31000, 32000).unwrap();
        let listener = TcpPeerListener::new(expected_port, runtime.clone()).unwrap();

        let result = listener.port();

        assert_eq!(expected_port, result);
    }

    #[test]
    fn test_recv() {
        init_logger!();
        let runtime = Arc::new(Runtime::new().unwrap());
        let (tx, rx) = channel();
        let port = available_port!(31000, 32000).unwrap();
        let mut listener = TcpPeerListener::new(port, runtime.clone()).unwrap();

        runtime.spawn(async move {
            if let Some(entry) = listener.recv().await {
                if let Err(e) = tx.send(entry) {
                    error!("Failed to send peer connection, {}", e);
                }
            }
        });

        let addr: SocketAddr = ([127, 0, 0, 1], port).into();
        runtime
            .block_on(TcpStream::connect(addr))
            .expect("expected the connection to succeed");

        let _ = rx
            .recv_timeout(Duration::from_millis(200))
            .expect("expected to receive an incoming connection");
    }

    #[test]
    fn test_peer_listener_drop() {
        init_logger!();
        let runtime = Arc::new(Runtime::new().unwrap());
        let addr: SocketAddr = ([127, 0, 0, 1], 6881).into();
        let listener = TcpPeerListener::new(6881, runtime.clone()).unwrap();

        drop(listener);
        std::thread::sleep(Duration::from_millis(100));

        let result = runtime.block_on(TcpStream::connect(addr));
        match result {
            Err(e) => {
                assert_eq!(std::io::ErrorKind::ConnectionRefused, e.kind());
            }
            Ok(_) => assert!(false, "expected the peer listener to have been closed"),
        }
    }
}
