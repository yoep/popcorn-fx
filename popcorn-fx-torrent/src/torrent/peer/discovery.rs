use crate::torrent::peer::extension::Extensions;
use crate::torrent::peer::{Peer, PeerId, ProtocolExtensionFlags, Result};
use crate::torrent::TorrentContext;
use async_trait::async_trait;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

/// A peer discovery is responsible for discovering outgoing and incoming peer connections.
#[async_trait]
pub trait PeerDiscovery: Debug + Send + Sync {
    /// Tries to dial (_create outgoing connection with_) the given peer address.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The unique peer identifier of the torrent.
    /// * `peer_addr` - The address of the peer to dial.
    /// * `torrent` - The torrent context of the peer connection.
    /// * `protocol_extensions` - The peer protocol extensions that should be enabled for the connection. (BEP4)
    /// * `extensions` - The peer extensions that should be activated for the connection. (BEP10)
    /// * `connection_timeout` - The timeout of a peer connection.
    ///
    /// # Returns
    ///
    /// It returns a [Peer] if the connection was established.
    async fn dial(
        &self,
        peer_id: PeerId,
        peer_addr: SocketAddr,
        torrent: Arc<TorrentContext>,
        protocol_extensions: ProtocolExtensionFlags,
        extensions: Extensions,
        connection_timeout: Duration,
    ) -> Result<Box<dyn Peer>>;
}
