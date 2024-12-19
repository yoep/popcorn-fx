use crate::torrent::peer::{Error, Result};
use log::debug;
use std::net::SocketAddr;
use std::sync::{mpsc, Arc};
use tokio::net::UdpSocket;
use tokio::runtime::Runtime;
use tokio::select;
use tokio_util::bytes::BytesMut;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, PartialEq)]
pub struct Packet {
    padding: usize,
    data: BytesMut,
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// Regular packet type
    Data = 0,
    /// Finalize the connection
    Fin = 1,
    /// State packet
    State = 2,
    /// Terminate the connection forcefully
    Reset = 3,
    /// Initiate a connection
    Syn = 4,
}

#[derive(Debug)]
pub struct UtpListener {
    port: u16,
    runtime: Arc<Runtime>,
    cancellation_token: CancellationToken,
}

impl UtpListener {
    pub fn new(port: u16, runtime: Arc<Runtime>) -> Result<Self> {
        let (tx_ready, rx) = mpsc::channel();
        let cancellation_token = CancellationToken::new();

        runtime.spawn(Self::start(port, tx_ready, cancellation_token.clone()));

        match rx.recv() {
            Ok(ready) => ready?,
            Err(_) => return Err(Error::Closed),
        }

        Ok(Self {
            port,
            runtime,
            cancellation_token,
        })
    }

    async fn start(
        port: u16,
        ready_sender: mpsc::Sender<Result<()>>,
        cancellation_token: CancellationToken,
    ) {
        let ipv4 = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], port)))
            .await
            .map_err(|e| Error::from(e));

        match ipv4 {
            Ok(socket) => {
                Self::send(Ok(()), ready_sender);

                loop {
                    select! {
                        _ = cancellation_token.cancelled() => break,
                    }
                }
            }
            Err(e) => Self::send(Err(e), ready_sender),
        }
    }

    fn send(result: Result<()>, ready_sender: mpsc::Sender<Result<()>>) {
        if let Err(e) = ready_sender.send(result) {
            debug!("Failed to send uTP ready, {}", e);
        }
    }
}

#[derive(Debug)]
struct UtpSocket {
    /// The underlying socket used by the utp
    socket: UdpSocket,
}

#[cfg(test)]
mod tests {
    use super::*;
    use popcorn_fx_core::{available_port, init_logger};

    #[test]
    fn test_utp_listener_new() {
        init_logger!();
        let port = available_port!(31000, 32000).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());

        let result = UtpListener::new(port, runtime);
        assert_eq!(
            true,
            result.is_ok(),
            "expected an utp listener, got {:?} instead",
            result
        );

        let result = result.unwrap();
        assert_eq!(port, result.port);
    }
}
