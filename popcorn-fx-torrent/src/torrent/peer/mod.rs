pub use discovery::*;
pub use discovery_tcp::*;
pub use discovery_utp::*;
pub use errors::*;
pub use peer::*;
pub use peer_id::*;

mod discovery;
mod discovery_tcp;
mod discovery_utp;
mod errors;
pub mod extension;
mod peer;
mod peer_connection;
mod peer_id;
mod protocol;
pub mod webseed;

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::recv_timeout;
    use crate::torrent::peer::protocol::{UtpSocket, UtpSocketExtensions, UtpStream};
    use crate::torrent::{available_port, Torrent};

    use rand::{rng, Rng};
    use std::net::SocketAddr;
    use std::time::Duration;
    use tokio::sync::mpsc::unbounded_channel;

    /// Create a new uTP socket.
    #[macro_export]
    macro_rules! create_utp_socket {
        ($runtime:expr) => {
            crate::torrent::peer::tests::create_utp_socket($runtime)
        };
        ($port:expr, $runtime:expr) => {
            crate::torrent::peer::tests::create_utp_socket_with_port($port, $runtime)
        };
    }

    /// Create a new uTP socket pair which don't overlap with port ranges.
    #[macro_export]
    macro_rules! create_utp_socket_pair {
        () => {
            crate::torrent::peer::tests::create_utp_socket_pair(vec![], vec![]).await
        };
        ($incoming_extensions:expr, $outgoing_extensions:expr) => {
            crate::torrent::peer::tests::create_utp_socket_pair(
                $incoming_extensions,
                $outgoing_extensions,
            )
            .await
        };
    }

    pub async fn create_utp_socket() -> UtpSocket {
        let port_start = rng().random_range(6881..20000);
        let port = available_port(port_start, 31000).unwrap();
        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        UtpSocket::new(addr, Duration::from_secs(1), vec![])
            .await
            .expect("expected an utp socket")
    }

    pub async fn create_utp_socket_with_port(port: u16) -> UtpSocket {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        UtpSocket::new(addr, Duration::from_secs(1), vec![])
            .await
            .expect("expected an utp socket")
    }

    pub async fn create_utp_peer_pair(
        incoming_socket: &UtpSocket,
        outgoing_socket: &UtpSocket,
        incoming_torrent: &Torrent,
        outgoing_torrent: &Torrent,
        protocols: ProtocolExtensionFlags,
    ) -> (BitTorrentPeer, BitTorrentPeer) {
        let incoming_context = incoming_torrent.instance().unwrap();
        let outgoing_context = outgoing_torrent.instance().unwrap();
        let (tx, mut rx) = unbounded_channel();

        // create the uTP stream pair
        let outgoing_stream = outgoing_socket
            .connect(incoming_socket.addr())
            .await
            .expect("expected an outgoing utp stream");
        let incoming_stream = incoming_socket
            .recv()
            .await
            .expect("expected an incoming uTP stream");

        // create the incoming uTP peer handler thread
        let incoming_extensions = incoming_context.extensions();
        let incoming_addr = outgoing_socket.addr();
        tokio::spawn(async move {
            let peer = BitTorrentPeer::new_inbound(
                PeerId::new(),
                incoming_addr,
                PeerStream::Utp(incoming_stream),
                incoming_context,
                protocols,
                incoming_extensions,
                Duration::from_secs(50),
            )
            .await
            .expect("expected an incoming uTP peer");
            tx.send(peer).unwrap();
        });

        let outgoing_extensions = outgoing_context.extensions();
        let outgoing_peer = BitTorrentPeer::new_outbound(
            PeerId::new(),
            incoming_socket.addr(),
            PeerStream::Utp(outgoing_stream),
            outgoing_context.clone(),
            protocols,
            outgoing_extensions,
            Duration::from_secs(50),
        )
        .await
        .expect("expected an outgoing uTP peer");

        let incoming_peer = recv_timeout!(&mut rx, Duration::from_secs(1));

        (incoming_peer, outgoing_peer)
    }

    pub async fn create_utp_socket_pair(
        incoming_extensions: UtpSocketExtensions,
        outgoing_extensions: UtpSocketExtensions,
    ) -> (UtpSocket, UtpSocket) {
        let mut rng = rng();

        let port = available_port(rng.random_range(20000..21000), 21000).unwrap();
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let left = UtpSocket::new(addr, Duration::from_secs(2), incoming_extensions)
            .await
            .expect("expected a new utp socket");

        let port = available_port(rng.random_range(21000..22000), 22000).unwrap();
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let right = UtpSocket::new(addr, Duration::from_secs(2), outgoing_extensions)
            .await
            .expect("expected a new utp socket");

        (left, right)
    }

    pub async fn create_utp_stream_pair(
        incoming: &UtpSocket,
        outgoing: &UtpSocket,
    ) -> (UtpStream, UtpStream) {
        let target_addr = incoming.addr();
        let outgoing_stream = outgoing
            .connect(target_addr)
            .await
            .expect("expected an outgoing utp stream");
        let incoming_stream = incoming
            .recv()
            .await
            .expect("expected an incoming uTP stream");

        (incoming_stream, outgoing_stream)
    }
}
