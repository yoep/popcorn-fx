use crate::torrent::peer::extension::{Error, Extension, ExtensionNumber, Result};
use crate::torrent::peer::protocol::Message;
use crate::torrent::peer::{ConnectionType, PeerCommandEvent, PeerContext, PeerEvent};
use crate::torrent::{CompactIpv4Addrs, CompactIpv6Addrs, PeerInfo, TorrentEvent};
use async_trait::async_trait;
use bitmask_enum::bitmask;
use log::{debug, trace, warn};
use popcorn_fx_core::core::callback::Callback;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

const EXTENSION_NAME_PEX: &str = "ut_pex";

/// The Peer Exchange message.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PexMessage {
    /// The added ipv4 peer addresses
    #[serde(default, with = "crate::torrent::compact::compact_ipv4")]
    pub added: CompactIpv4Addrs,
    /// The flags of the added ipv4 peer addresses
    #[serde(rename = "added.f", with = "pex_flags")]
    pub added_flags: Vec<PexFlag>,
    /// The added ipv6 peer addresses
    #[serde(default, with = "crate::torrent::compact::compact_ipv6")]
    pub added6: CompactIpv6Addrs,
    /// The flags of the added ipv6 peer addresses
    #[serde(rename = "added6.f", with = "pex_flags")]
    pub added6_flags: Vec<PexFlag>,
    /// The dropped ipv4 peer addresses
    #[serde(default, with = "crate::torrent::compact::compact_ipv4")]
    pub dropped: CompactIpv4Addrs,
    /// The dropped ipv6 peer addresses
    #[serde(default, with = "crate::torrent::compact::compact_ipv6")]
    pub dropped6: CompactIpv6Addrs,
}

impl PexMessage {
    /// Get all the discovered peers by the swarm
    fn discovered_peers(&self) -> Vec<SocketAddr> {
        let mut peers: Vec<SocketAddr> = self.added.iter().map(|e| SocketAddr::from(e)).collect();
        peers.extend(self.added6.iter().map(|e| SocketAddr::from(e)));
        peers
    }

    /// Get all the dropped peers from the swarm
    fn dropped_peers(&self) -> Vec<SocketAddr> {
        let mut peers: Vec<SocketAddr> = self.dropped.iter().map(|e| SocketAddr::from(e)).collect();
        peers.extend(self.dropped6.iter().map(|e| SocketAddr::from(e)));
        peers
    }

    /// Check if the message is empty.
    /// It returns `true` if the message is empty, else `false`.
    fn is_empty(&self) -> bool {
        self.added.is_empty()
            && self.added6.is_empty()
            && self.dropped.is_empty()
            && self.dropped6.is_empty()
    }
}

#[bitmask(u8)]
#[bitmask_config(vec_debug, flags_iter)]
pub enum PexFlag {
    /// prefers encryption, as indicated by e field in extension handshake
    EncryptionPreferred = 0x01,
    /// seed/upload_only
    UploadOnly = 0x02,
    /// supports utp
    UtpSupported = 0x04,
    /// peer indicated ut_holepunch support in extension handshake
    HolepunchSupported = 0x08,
    /// outgoing connection
    OutgoingConnection = 0x10,
}

impl Serialize for PexFlag {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde::Serialize::serialize(&self.bits(), serializer)
    }
}

impl<'de> Deserialize<'de> for PexFlag {
    fn deserialize<D>(deserializer: D) -> std::result::Result<PexFlag, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value: u8 = Deserialize::deserialize(deserializer)?;
        Ok(PexFlag::from(value))
    }
}

/// The PEX extensions as defined in BEP11.
#[derive(Debug)]
pub struct PexExtension {
    /// The pool which is used to manage the pex peer addresses
    pool: PexPool,
}

impl PexExtension {
    pub fn new() -> Self {
        Self {
            pool: PexPool::new(),
        }
    }

    fn subscribe_to_torrent(&self, peer: &PeerContext) {
        let torrent = peer.torrent();
        let pool = self.pool.clone();

        if let Some(extension_number) = peer.find_client_extension_number(EXTENSION_NAME_PEX) {
            let mut receiver = torrent.subscribe();

            let event_sender = peer.event_sender().clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(90));

                loop {
                    tokio::select! {
                        _ = pool.inner.cancellation_token.cancelled() => break,
                        event = receiver.recv() => {
                            match event {
                                Some(event) => pool.handle_event(&*event).await,
                                None => break,
                            }
                        }
                        _ = interval.tick() => {
                            // if the event sender is closed
                            // it means that the peer is closed, so we stop this task
                            if !pool.inform_peer(&event_sender, &extension_number).await {
                                break;
                            }
                        },
                    }
                }
            });
        } else {
            debug!(
                "Unable to subscribe to peer torrent, client extension {} not found",
                EXTENSION_NAME_PEX
            );
        }
    }
}

#[async_trait]
impl Extension for PexExtension {
    fn name(&self) -> &str {
        EXTENSION_NAME_PEX
    }

    async fn handle<'a>(&'a self, payload: &'a [u8], peer: &'a PeerContext) -> Result<()> {
        let message: PexMessage = serde_bencode::from_bytes(payload)?;
        debug!("Received PEX message {:?} from peer {}", message, peer);

        let discovered_peers = message.discovered_peers();
        if discovered_peers.len() > 0 {
            peer.invoke_event(PeerEvent::PeersDiscovered(discovered_peers));
        }

        let dropped_peers = message.dropped_peers();
        if dropped_peers.len() > 0 {
            peer.invoke_event(PeerEvent::PeersDropped(dropped_peers));
        }

        Ok(())
    }

    async fn on<'a>(&'a self, event: &'a PeerEvent, peer: &'a PeerContext) {
        if let PeerEvent::ExtendedHandshakeCompleted = event {
            self.subscribe_to_torrent(peer)
        }
    }

    fn clone_boxed(&self) -> Box<dyn Extension> {
        Box::new(Self::new())
    }
}

impl Drop for PexExtension {
    fn drop(&mut self) {
        self.pool.close();
    }
}

#[derive(Debug, Clone)]
struct PexPool {
    inner: Arc<InnerPexPool>,
}

impl PexPool {
    fn new() -> Self {
        Self {
            inner: Arc::new(InnerPexPool::new()),
        }
    }

    async fn handle_event(&self, event: &TorrentEvent) {
        match event {
            TorrentEvent::PeerConnected(peer) => self.inner.peer_added(peer).await,
            TorrentEvent::PeerDisconnected(peer) => self.inner.peer_removed(peer).await,
            _ => {}
        }
    }

    async fn inform_peer(
        &self,
        sender: &UnboundedSender<PeerCommandEvent>,
        extension_number: &ExtensionNumber,
    ) -> bool {
        let message = self.inner.message().await;

        if !message.is_empty() {
            let message_info = format!("{:?}", message);
            return match self
                .try_inform_peer(sender, message, extension_number.clone())
                .await
            {
                Ok(_) => {
                    debug!("Send PEX message {} to peer", message_info);
                    true
                }
                Err(e) => {
                    debug!("Failed to send PEX message {}, {}", message_info, e);
                    false
                }
            };
        }

        true
    }

    async fn try_inform_peer(
        &self,
        sender: &UnboundedSender<PeerCommandEvent>,
        message: PexMessage,
        extension_number: ExtensionNumber,
    ) -> Result<()> {
        let message_bytes = serde_bencode::to_bytes(&message)?;
        sender
            .send(PeerCommandEvent::Send(Message::ExtendedPayload(
                extension_number.clone(),
                message_bytes,
            )))
            .map_err(|e| Error::Operation(e.to_string()))?;

        Ok(())
    }

    fn close(&self) {
        self.inner.cancellation_token.cancel();
    }
}

#[derive(Debug)]
struct InnerPexPool {
    added_peers: RwLock<Vec<PexPeer>>,
    dropped_peers: RwLock<Vec<PexPeer>>,
    cancellation_token: CancellationToken,
}

impl InnerPexPool {
    fn new() -> Self {
        Self {
            added_peers: Default::default(),
            dropped_peers: Default::default(),
            cancellation_token: Default::default(),
        }
    }

    async fn peer_added(&self, peer: &PeerInfo) {
        let mut flags = PexFlag::none();

        if peer.connection_type == ConnectionType::Outbound {
            flags |= PexFlag::OutgoingConnection;
        }

        self.added_peers.write().await.push(PexPeer {
            addr: peer.addr.clone(),
            flags,
        });
    }

    async fn peer_removed(&self, peer: &PeerInfo) {
        let mut flags = PexFlag::none();

        if peer.connection_type == ConnectionType::Outbound {
            flags |= PexFlag::OutgoingConnection;
        }

        self.dropped_peers.write().await.push(PexPeer {
            addr: peer.addr.clone(),
            flags,
        });
    }

    /// Get the PEX message to send to the peer and reset the pool.
    async fn message(&self) -> PexMessage {
        let (added_peers, dropped_peers) = {
            let mut added_lock = self.added_peers.write().await;
            let mut dropped_lock = self.dropped_peers.write().await;
            (
                added_lock.drain(..).collect::<Vec<_>>(),
                dropped_lock.drain(..).collect::<Vec<_>>(),
            )
        };
        let mut added: CompactIpv4Addrs = vec![];
        let mut added_flags = vec![];
        let mut added6: CompactIpv6Addrs = vec![];
        let mut added6_flags = vec![];
        let mut dropped: CompactIpv4Addrs = vec![];
        let mut dropped6: CompactIpv6Addrs = vec![];

        for peer in added_peers {
            if peer.addr.is_ipv4() {
                match peer.addr.try_into() {
                    Ok(compact) => {
                        added.push(compact);
                        added_flags.push(peer.flags);
                    }
                    Err(e) => warn!("Failed to convert peer address to compact, {}", e),
                }
            } else {
                match peer.addr.try_into() {
                    Ok(compact) => {
                        added6.push(compact);
                        added6_flags.push(peer.flags);
                    }
                    Err(e) => warn!("Failed to convert peer address to compact, {}", e),
                }
            }
        }
        for peer in dropped_peers {
            if peer.addr.is_ipv4() {
                match peer.addr.try_into() {
                    Ok(compact) => dropped.push(compact),
                    Err(e) => warn!("Failed to convert peer address to compact, {}", e),
                }
            } else {
                match peer.addr.try_into() {
                    Ok(compact) => dropped6.push(compact),
                    Err(e) => warn!("Failed to convert peer address to compact, {}", e),
                }
            }
        }

        PexMessage {
            added,
            added_flags,
            added6,
            added6_flags,
            dropped,
            dropped6,
        }
    }
}

#[derive(Debug)]
struct PexPeer {
    addr: SocketAddr,
    flags: PexFlag,
}

mod pex_flags {
    use super::*;
    use serde::Deserializer;

    struct PexFlagsVisitor;

    impl<'de> serde::de::Visitor<'de> for PexFlagsVisitor {
        type Value = Vec<PexFlag>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("expected a bytes array or sequence of bytes")
        }

        fn visit_bytes<E>(self, value: &[u8]) -> std::result::Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let mut flags = Vec::new();

            for byte in value {
                flags.push(PexFlag::from(*byte));
            }

            Ok(flags)
        }

        fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut flags = Vec::new();

            while let Some(byte) = seq.next_element::<u8>()? {
                flags.push(PexFlag::from(byte));
            }

            Ok(flags)
        }
    }

    pub fn serialize<S>(flags: &Vec<PexFlag>, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = flags.iter().map(|f| f.bits()).collect::<Vec<u8>>();
        serde::Serialize::serialize(&bytes, serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<Vec<PexFlag>, D::Error>
    where
        D: Deserializer<'de>,
    {
        D::deserialize_any(deserializer, PexFlagsVisitor {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pex_flags() {
        let expected_result = PexFlag::UtpSupported | PexFlag::OutgoingConnection;
        let bytes = serde_bencode::to_bytes(&expected_result).unwrap();

        let result = serde_bencode::from_bytes::<PexFlag>(&bytes).unwrap();
        assert_eq!(expected_result, result);
    }
}
