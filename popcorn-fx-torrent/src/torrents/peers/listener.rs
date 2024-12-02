use crate::torrents::peers::{Error, Result};
use async_trait::async_trait;
use log::{debug, error, trace, warn};
use popcorn_fx_core::core::block_in_place;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use tokio::select;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct PeerEntry {
    /// The peer address
    pub socket_addr: SocketAddr,
    /// The peer incoming tcp stream
    pub stream: TcpStream,
}

#[async_trait]
pub trait PeerListener: Debug + Send + Sync {
    /// Get the port this peer listener is listening on.
    fn port(&self) -> u16;

    /// Receive an incoming tcp stream from the peer listener.
    ///
    /// # Returns
    ///
    /// Returns [None] when the listener has been dropped.
    async fn recv(&mut self) -> Option<PeerEntry>;
}

#[derive(Debug)]
pub struct DefaultPeerListener {
    port: u16,
    receiver: UnboundedReceiver<PeerEntry>,
    runtime: Arc<Runtime>,
    cancellation_token: CancellationToken,
}

impl DefaultPeerListener {
    /// Create a new peer listener on the given port.
    /// It tries to bind to both ipv4 and ipv6 addresses for the given port.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use tokio::runtime::Runtime;
    /// use popcorn_fx_torrent::torrents::peers::DefaultPeerListener;
    ///
    /// fn new_listener() -> DefaultPeerListener {
    ///     let runtime = Arc::new(Runtime::new().unwrap());
    ///     let port = 6891;
    ///
    ///     DefaultPeerListener::new(port, runtime).unwrap()
    /// }
    ///
    /// ```
    pub fn new(port: u16, runtime: Arc<Runtime>) -> Result<Self> {
        trace!("Trying to create new peer listener on port {}", port);
        let cancellation_token = CancellationToken::new();
        let (result_sender, result_receiver) = channel();
        let (sender, receiver) = unbounded_channel();

        let listener_cancellation = cancellation_token.clone();
        runtime.spawn(async move {
            Self::start(port, result_sender, sender, listener_cancellation).await
        });

        // wait for the listeners to be ready
        result_receiver
            .recv()
            .expect("expected a result to have been returned")?;

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
        result_sender: Sender<Result<()>>,
        sender: UnboundedSender<PeerEntry>,
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
                result_sender.send(Ok(())).unwrap();
            }
            Err(e) => {
                result_sender.send(Err(Error::Io(e.to_string()))).unwrap();
                return;
            }
        }

        loop {
            select! {
                _ = cancellation_token.cancelled() => break,
                Ok((stream, socket)) = ipv4.accept() => Self::accept_incoming_connection(stream, socket, &sender),
                Ok((stream, socket)) = ipv6.accept() => Self::accept_incoming_connection(stream, socket, &sender),
            }
        }

        debug!("Peer listener (port {}) has stopped", port);
    }

    async fn try_create_listeners(port: u16) -> Result<(TcpListener, TcpListener)> {
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
            stream,
        }) {
            warn!("Failed to send peer connection, {}", e);
        }
    }
}

#[async_trait]
impl PeerListener for DefaultPeerListener {
    fn port(&self) -> u16 {
        self.port
    }

    async fn recv(&mut self) -> Option<PeerEntry> {
        self.receiver.recv().await
    }
}

impl Drop for DefaultPeerListener {
    fn drop(&mut self) {
        trace!("Peer listener (port {}) is being dropped", self.port);
        self.cancellation_token.cancel();
    }
}

#[cfg(test)]
mod tests {
    use log::error;
    use popcorn_fx_core::available_port;
    use popcorn_fx_core::testing::init_logger;
    use std::sync::mpsc::channel;
    use std::time::Duration;
    use tokio::net::TcpStream;

    use super::*;

    #[test]
    fn test_port() {
        init_logger();
        let runtime = Arc::new(Runtime::new().unwrap());
        let expected_port = available_port!(31000, 32000).unwrap();
        let listener = DefaultPeerListener::new(expected_port, runtime.clone()).unwrap();

        let result = listener.port();

        assert_eq!(expected_port, result);
    }

    #[test]
    fn test_recv() {
        init_logger();
        let runtime = Arc::new(Runtime::new().unwrap());
        let (tx, rx) = channel();
        let port = available_port!(31000, 32000).unwrap();
        let mut listener = DefaultPeerListener::new(port, runtime.clone()).unwrap();

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
        init_logger();
        let runtime = Arc::new(Runtime::new().unwrap());
        let addr: SocketAddr = ([127, 0, 0, 1], 6881).into();
        let listener = DefaultPeerListener::new(6881, runtime.clone()).unwrap();

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
