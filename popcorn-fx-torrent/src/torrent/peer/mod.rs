pub use discovery::*;
pub use discovery_tcp::*;
pub use discovery_utp::*;
pub use errors::*;
pub use listener::*;
pub use listener_tcp::*;
pub use peer::*;
pub use peer_id::*;

mod discovery;
mod discovery_tcp;
mod discovery_utp;
mod errors;
pub mod extension;
mod listener;
mod listener_tcp;
mod peer;
mod peer_connection;
mod peer_id;
mod protocol;
pub mod webseed;

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::torrent::peer::protocol::{UtpSocket, UtpSocketExtensions, UtpStream};
    use crate::torrent::{available_port, Torrent};
    use rand::{rng, Rng};
    use std::net::SocketAddr;
    use std::sync::mpsc::channel;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::runtime::Runtime;

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
            crate::torrent::peer::tests::create_utp_socket_pair(
                vec![],
                vec![],
                std::sync::Arc::new(tokio::runtime::Runtime::new().unwrap()),
            )
        };
        ($runtime:expr) => {
            crate::torrent::peer::tests::create_utp_socket_pair(vec![], vec![], $runtime)
        };
        ($incoming_extensions:expr, $outgoing_extensions:expr, $runtime:expr) => {
            crate::torrent::peer::tests::create_utp_socket_pair(
                $incoming_extensions,
                $outgoing_extensions,
                $runtime,
            )
        };
    }

    pub fn create_utp_socket(runtime: Arc<Runtime>) -> UtpSocket {
        let port_start = rng().random_range(6881..20000);
        let port = available_port(port_start, 31000).unwrap();
        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        runtime
            .clone()
            .block_on(UtpSocket::new(
                addr,
                Duration::from_secs(1),
                vec![],
                runtime,
            ))
            .expect("expected an utp socket")
    }

    pub fn create_utp_socket_with_port(port: u16, runtime: Arc<Runtime>) -> UtpSocket {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        runtime
            .clone()
            .block_on(UtpSocket::new(
                addr,
                Duration::from_secs(1),
                vec![],
                runtime,
            ))
            .expect("expected an utp socket")
    }

    pub fn create_utp_peer_pair(
        incoming_socket: &UtpSocket,
        outgoing_socket: &UtpSocket,
        incoming_torrent: &Torrent,
        outgoing_torrent: &Torrent,
        protocols: ProtocolExtensionFlags,
    ) -> (BitTorrentPeer, BitTorrentPeer) {
        let incoming_context = incoming_torrent.instance().unwrap();
        let outgoing_context = outgoing_torrent.instance().unwrap();
        let runtime = outgoing_context.runtime();
        let (tx, rx) = channel();

        // create the uTP stream pair
        let outgoing_stream = runtime
            .block_on(outgoing_socket.connect(incoming_socket.addr()))
            .expect("expected an outgoing utp stream");
        let incoming_stream = runtime
            .block_on(incoming_socket.recv())
            .expect("expected an incoming uTP stream");

        // create the incoming uTP peer handler thread
        let incoming_extensions = incoming_context.extensions();
        let incoming_runtime = runtime.clone();
        let incoming_addr = outgoing_socket.addr();
        runtime.spawn(async move {
            let peer = BitTorrentPeer::new_inbound(
                PeerId::new(),
                incoming_addr,
                PeerStream::Utp(incoming_stream),
                incoming_context,
                protocols,
                incoming_extensions,
                Duration::from_secs(50),
                incoming_runtime,
            )
            .await
            .expect("expected an incoming uTP peer");
            tx.send(peer).unwrap();
        });

        let outgoing_extensions = outgoing_context.extensions();
        let outgoing_peer = runtime
            .block_on(BitTorrentPeer::new_outbound(
                PeerId::new(),
                incoming_socket.addr(),
                PeerStream::Utp(outgoing_stream),
                outgoing_context.clone(),
                protocols,
                outgoing_extensions,
                Duration::from_secs(50),
                runtime.clone(),
            ))
            .expect("expected an outgoing uTP peer");

        let incoming_peer = rx.recv_timeout(Duration::from_secs(1)).unwrap();

        (incoming_peer, outgoing_peer)
    }

    pub fn create_utp_socket_pair(
        incoming_extensions: UtpSocketExtensions,
        outgoing_extensions: UtpSocketExtensions,
        runtime: Arc<Runtime>,
    ) -> (UtpSocket, UtpSocket) {
        let mut rng = rng();

        let port = available_port(rng.random_range(20000..21000), 21000).unwrap();
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let left = runtime
            .block_on(UtpSocket::new(
                addr,
                Duration::from_secs(2),
                incoming_extensions,
                runtime.clone(),
            ))
            .expect("expected a new utp socket");

        let port = available_port(rng.random_range(21000..22000), 22000).unwrap();
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let right = runtime
            .block_on(UtpSocket::new(
                addr,
                Duration::from_secs(2),
                outgoing_extensions,
                runtime.clone(),
            ))
            .expect("expected a new utp socket");

        (left, right)
    }

    pub fn create_utp_stream_pair(
        incoming: &UtpSocket,
        outgoing: &UtpSocket,
    ) -> (UtpStream, UtpStream) {
        let runtime = outgoing.context().runtime();
        let target_addr = incoming.addr();
        let outgoing_stream = runtime
            .block_on(outgoing.connect(target_addr))
            .expect("expected an outgoing utp stream");
        let incoming_stream = runtime
            .block_on(incoming.recv())
            .expect("expected an incoming uTP stream");

        (incoming_stream, outgoing_stream)
    }
}
