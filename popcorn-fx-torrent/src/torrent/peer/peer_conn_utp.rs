use crate::torrent::peer::protocol::{Handshake, Message, UtpStream};
use crate::torrent::peer::{
    ConnectionType, DataTransferStats, Error, PeerConn, PeerId, PeerResponse, Result,
};
use async_trait::async_trait;
use derive_more::Display;
use log::debug;
use std::net::SocketAddr;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Display)]
#[display(fmt = "{}[{}]", id, addr)]
pub struct UtpConnection {
    id: PeerId,
    addr: SocketAddr,
    stream: UtpStream,
    cancellation_token: CancellationToken,
}

impl UtpConnection {
    pub fn new(id: PeerId, addr: SocketAddr, stream: UtpStream) -> Self {
        let cancellation_token = CancellationToken::new();

        Self {
            id,
            addr,
            stream,
            cancellation_token,
        }
    }
}

#[async_trait]
impl PeerConn for UtpConnection {
    fn conn_type(&self) -> ConnectionType {
        ConnectionType::Utp
    }

    async fn recv(&self) -> Option<PeerResponse> {
        if self.stream.is_closed().await {
            debug!(
                "Utp stream {} is unable to receive any messages, stream is closed",
                self.stream
            );
            return None;
        }

        match self.stream.recv().await {
            None => None,
            Some(data) => {
                if data.len() == 68 {
                    match Handshake::from_bytes(&self.addr, data.as_slice()) {
                        Ok(handshake) => Some(PeerResponse::Handshake(handshake)),
                        Err(e) => Some(PeerResponse::Error(e)),
                    }
                } else {
                    let elapsed_micro = self.stream.latency().await as u128;
                    match Message::try_from(data.as_slice()) {
                        Ok(message) => Some(PeerResponse::Message(
                            message,
                            DataTransferStats {
                                transferred_bytes: data.len(),
                                elapsed_micro,
                            },
                        )),
                        Err(e) => Some(PeerResponse::Error(e)),
                    }
                }
            }
        }
    }

    async fn write<'a>(&'a self, bytes: &'a [u8]) -> Result<()> {
        if self.stream.is_closed().await {
            return Err(Error::Closed);
        }

        self.stream.send(bytes).await
    }

    async fn close(&self) -> Result<()> {
        if let Err(e) = self.stream.close().await {
            debug!(
                "Utp stream {} failed to close gracefully, {}",
                self.stream, e
            );
        }

        self.cancellation_token.cancel();
        Ok(())
    }
}
