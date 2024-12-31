use async_trait::async_trait;
use std::fmt::Debug;
use std::net::SocketAddr;
use tokio::net::TcpStream;

/// A received peer entry incoming connection.
#[derive(Debug)]
pub struct PeerEntry {
    /// The peer address
    pub socket_addr: SocketAddr,
    /// The peer incoming tcp stream
    pub stream: PeerStream,
}

/// The underlying stream of the incoming peer connection
#[derive(Debug)]
pub enum PeerStream {
    /// The peer is a TCP stream
    Tcp(TcpStream),
    /// The peer is a UTP stream
    Utp,
}

impl PartialEq for PeerStream {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PeerStream::Tcp(_), PeerStream::Tcp(_)) => true,
            _ => false,
        }
    }
}

/// The peer listener accepts incoming peer connections.
/// It listeners on a [PeerListener::port] and accepts incoming connections.
///
/// # Notice
///
/// Every implementation of the peer listener should correctly stop listening when dropped.
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

    /// Close the peer listener.
    /// This will prevent any new incoming connections from being received.
    fn close(&self);
}
