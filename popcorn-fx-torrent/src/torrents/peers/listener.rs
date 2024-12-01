use std::net::SocketAddr;
use std::sync::Arc;

use log::{debug, trace};
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::torrents::peers::Result;

#[derive(Debug)]
pub struct PeerListener {
    port: u16,
    cancellation_token: CancellationToken,
}

impl PeerListener {
    pub async fn new(port: u16, runtime: Arc<Runtime>) -> Result<Self> {
        trace!("Trying to create new peer listener on port {}", port);
        let cancellation_token = CancellationToken::new();
        let ipv4 = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port))).await?;
        let ipv6 = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], port))).await?;

        runtime.spawn(Self::start_listener(ipv4, ipv6, cancellation_token.clone()));

        debug!("Created new peer listener on port {}", port);
        Ok(Self {
            port,
            cancellation_token,
        })
    }

    /// Retrieve the port number of this listener.
    ///
    /// # Returns
    ///
    /// Returns the port this peer listener is listening on.
    pub fn port(&self) -> u16 {
        self.port
    }

    async fn start_listener(
        ipv4: TcpListener,
        ipv6: TcpListener,
        cancellation_token: CancellationToken,
    ) {
        trace!("Starting new peer listener");
        loop {
            select! {
                _ = cancellation_token.cancelled() => break,
                Ok((_stream, socket)) = ipv4.accept() => {
                    trace!("Received incoming peer connection {}", socket);
                },
                Ok((_stream, socket)) = ipv6.accept() => {
                    trace!("Received incoming peer connection {}", socket);
                },
            }
        }
        drop(ipv4);
        drop(ipv6);
        debug!("Peer listener has stopped");
    }
}

impl Drop for PeerListener {
    fn drop(&mut self) {
        trace!("Dropping {:?}", self);
        self.cancellation_token.cancel();
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::net::TcpStream;

    use popcorn_fx_core::testing::init_logger;

    use super::*;

    #[test]
    fn test_peer_listener_drop() {
        init_logger();
        let runtime = Arc::new(Runtime::new().unwrap());
        let addr: SocketAddr = ([127, 0, 0, 1], 6881).into();
        let listener = runtime
            .block_on(PeerListener::new(6881, runtime.clone()))
            .unwrap();

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
