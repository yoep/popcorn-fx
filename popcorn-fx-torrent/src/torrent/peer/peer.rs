use crate::torrent::merkle::LEAF_BLOCK_SIZE;
use crate::torrent::peer::extension::{
    Extension, ExtensionName, ExtensionNumber, ExtensionRegistry, Extensions,
};
use crate::torrent::peer::peer_connection::PeerConnection;
use crate::torrent::peer::protocol::UtpStream;
use crate::torrent::peer::protocol::{
    ExtendedHandshake, Handshake, HashRequest, Message, Piece, Request,
};
use crate::torrent::peer::{Error, PeerId, Result};
use crate::torrent::{
    calculate_byte_rate, CompactIp, InfoHash, PeerPriority, PieceIndex, TorrentContext,
    TorrentEvent, TorrentMetadata, TorrentMetadataInfo, MAX_PIECE_PART_SIZE,
};
use async_trait::async_trait;
use bit_vec::BitVec;
use bitmask_enum::bitmask;
use byteorder::BigEndian;
use byteorder::ByteOrder;
use derive_more::Display;
use futures::future;
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use fx_handle::Handle;
use log::{debug, error, trace, warn};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::{Mutex, OwnedSemaphorePermit, RwLock};
use tokio::time::timeout;
use tokio::{select, time};
use tokio_util::sync::CancellationToken;

const KEEP_ALIVE_SECONDS: u64 = 90;
/// The maximum amount of in-flight pieces a peer can request
const MAX_PENDING_PIECES: usize = 3;

/// The peer's unique identifier handle.
pub type PeerHandle = Handle;

/// The [Peer] is a connection to a remote peer for exchanging piece data of a specific torrent.
#[async_trait]
pub trait Peer: Debug + Display + Send + Sync + Callback<PeerEvent> {
    /// Get the unique identifier handle of the peer.
    ///
    /// # Returns
    ///
    /// It returns the unique identifier handle of the peer.
    fn handle(&self) -> PeerHandle;

    /// Get the unique identifier handle of the peer as reference.
    ///
    /// # Returns
    ///
    /// Returns the unique identifier reference handle of the peer.
    fn handle_as_ref(&self) -> &PeerHandle;

    /// Get the peer client information.
    ///
    /// # Returns
    ///
    /// It returns an owned instance of the client info.
    fn client(&self) -> PeerClientInfo;

    /// Get the address of the remote peer.
    ///
    /// # Returns
    ///
    /// It returns the socket address of the remote peer.
    fn addr(&self) -> SocketAddr;

    /// Get the address reference of the remote peer.
    ///
    /// # Returns
    ///
    /// It returns the socket address of the remote peer as reference.
    fn addr_as_ref(&self) -> &SocketAddr;

    /// Get the current state of the peer.
    ///
    /// # Returns
    ///
    /// It returns the current state if the peer.
    async fn state(&self) -> PeerState;

    /// Get the connection stats with the remote peer.
    ///
    /// # Returns
    ///
    /// It returns the metrics of the connection transfer data.
    async fn stats(&self) -> PeerStats;

    /// Get the available pieces of the remote peer as a bit vector.
    ///
    /// # Returns
    ///
    /// It returns an empty bit vector when the handshake has not yet been completed, else the known [BitVec] of available pieces.
    async fn remote_piece_bitfield(&self) -> BitVec;

    /// Notify the remote peer that we have new piece(s) available for download.
    /// This operation is offloaded to the main loop of the [Peer], resulting in a non-blocking operation.
    ///
    /// In normal circumstances, this operation is only called by the [Torrent] of the peer.
    ///
    /// # Arguments
    ///
    /// * `pieces` - The piece indexes that have become available.
    fn notify_piece_availability(&self, pieces: Vec<PieceIndex>);

    /// Close the peer connection, cancelling any queued operation.
    /// The connection with the remote peer will be closed and this peer can no longer be used.
    async fn close(&self);
}

/// A peer connection is responsible for sending and receiving Bittorrent [Message]s to and from a remote peer.
#[async_trait]
pub(crate) trait PeerConn: Debug + Send + Sync {
    /// Get the protocol being used by the peer connection for communication with the remote peer.
    fn protocol(&self) -> ConnectionProtocol;

    /// Try to receive the next available Bittorrent [Message] from the remote peer.
    ///
    /// # Returns
    ///
    /// It returns a message when available, else [None] when the remote peer connection is closed.
    async fn recv(&self) -> Option<PeerResponse>;

    /// Write the given bytes to the remote peer.
    ///
    /// # Returns
    ///
    /// It returns an error when writing to the remote peer failed.
    async fn write<'a>(&'a self, bytes: &'a [u8]) -> Result<()>;

    /// Close the peer connection.
    ///
    /// # Returns
    ///
    /// It returns an error when the peer connection couldn't be closed gracefully.
    async fn close(&self) -> Result<()>;
}

/// The response of a remote peer connection.
#[derive(Debug, PartialEq)]
pub(crate) enum PeerResponse {
    /// The remote peer sent a handshake.
    Handshake(Handshake),
    /// The remote peer sent a message.
    Message(Message, DataTransferStats),
    /// The remote peer connection encountered an error.
    Error(Error),
    /// The remote peer has closed the connection.
    Closed,
}

/// The underlying stream implementation of a [Peer] connection.
/// This stream is used to connect with, or receive from, a remote peer.
#[derive(Debug)]
pub enum PeerStream {
    /// The peer is a TCP stream
    Tcp(TcpStream),
    /// The peer is a UTP stream
    Utp(UtpStream),
}

/// The choke states of a peer.
#[repr(u8)]
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
pub enum ChokeState {
    #[display(fmt = "choked")]
    Choked = 0,
    #[display(fmt = "un-choked")]
    UnChoked = 1,
}

impl PartialOrd for ChokeState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self == &ChokeState::Choked && other == &ChokeState::UnChoked {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Greater)
        }
    }
}

impl Ord for ChokeState {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

/// The interest states of a peer.
#[repr(u8)]
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
pub enum InterestState {
    #[display(fmt = "not interested")]
    NotInterested = 0,
    #[display(fmt = "interested")]
    Interested = 1,
}

impl PartialOrd<Self> for InterestState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self == &InterestState::NotInterested && other == &InterestState::Interested {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Greater)
        }
    }
}

impl Ord for InterestState {
    fn cmp(&self, other: &Self) -> Ordering {
        if self == other {
            Ordering::Equal
        } else if self == &InterestState::NotInterested && other == &InterestState::Interested {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

/// The underlying network protocol used by the peer to communicate with the remote peer.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ConnectionProtocol {
    Tcp,
    Utp,
    Http,
    Other,
}

impl Display for ConnectionProtocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// The connection direction type of the peer.
/// This indicates if the initial established connection with the remote peer was an inbound or outbound connection.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionDirection {
    Inbound = 0,
    Outbound = 1,
}

/// The state that a peer is in
#[derive(Debug, Display, Copy, Clone, PartialEq)]
pub enum PeerState {
    /// The peer is currently exchanging the handshake
    #[display(fmt = "performing peer handshake")]
    Handshake,
    /// The peer is currently trying to retrieve the metadata
    #[display(fmt = "retrieving metadata")]
    RetrievingMetadata,
    /// The peer is currently paused
    #[display(fmt = "paused")]
    Paused,
    /// The peer is currently idle
    #[display(fmt = "idle")]
    Idle,
    #[display(fmt = "downloading")]
    Downloading,
    #[display(fmt = "uploading")]
    Uploading,
    #[display(fmt = "error")]
    Error,
    #[display(fmt = "closed")]
    Closed,
}

/// The extension flags of the protocol.
/// See BEP4 (https://www.bittorrent.org/beps/bep_0004.html) for more info.
///
/// _The known collisions mentioned in BEP4, are ignored within these flags._
#[bitmask(u16)]
#[bitmask_config(vec_debug, flags_iter)]
pub enum ProtocolExtensionFlags {
    None,
    /// Azureus Messaging Protocol
    Azureus,
    /// Libtorrent Extension Protocol, aka Extensions
    LTEP,
    /// Extension Negotiation Protocol
    ENP,
    /// BitTorrent DHT
    Dht,
    /// XBT Peer Exchange
    XbtPeerExchange,
    /// suggest, haveall, havenone, reject request, and allow fast extensions
    Fast,
    /// NAT Traversal
    Nat,
    /// hybrid torrent legacy to v2 upgrade
    SupportV2,
}

impl Display for ProtocolExtensionFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut extensions = Vec::new();

        if self.contains(Self::Azureus) {
            extensions.push("Azureus");
        }
        if self.contains(Self::LTEP) {
            extensions.push("LTEP");
        }
        if self.contains(Self::ENP) {
            extensions.push("ENP");
        }
        if self.contains(Self::Dht) {
            extensions.push("DHT");
        }
        if self.contains(Self::XbtPeerExchange) {
            extensions.push("XBT");
        }
        if self.contains(Self::Fast) {
            extensions.push("Fast");
        }
        if self.contains(Self::Nat) {
            extensions.push("Nat");
        }
        if self.contains(Self::SupportV2) {
            extensions.push("SupportV2");
        }

        write!(f, "{}", extensions.join(" | "))
    }
}

/// The remote peer information
#[derive(Debug, Clone, PartialEq)]
pub struct RemotePeer {
    pub peer_id: PeerId,
    pub protocol_extensions: ProtocolExtensionFlags,
    pub extensions: ExtensionRegistry,
    pub client_name: Option<String>,
    /// Indicates that the connection has been upgraded to v2
    pub is_v2: bool,
}

#[derive(Clone, PartialEq)]
pub enum PeerEvent {
    /// Indicates that the handshake with the remote has been completed
    HandshakeCompleted,
    /// Indicates that the extended handshake with the remote peer has been completed
    ExtendedHandshakeCompleted,
    /// Indicates that the state of this peer has changed
    StateChanged(PeerState),
    /// Indicates that remote pieces have become available to be downloaded
    RemoteAvailablePieces(Vec<PieceIndex>),
    /// Indicates that remote pieces have become unavailable and can no longer be downloaded
    RemoteUnavailablePieces(Vec<PieceIndex>),
    /// Indicates that one or more peers has been discovered by the swarm
    PeersDiscovered(Vec<SocketAddr>),
    /// Indicates that one or more peers are dropped from the swarm
    PeersDropped(Vec<SocketAddr>),
}

impl Debug for PeerEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PeerEvent::HandshakeCompleted => write!(f, "HandshakeCompleted"),
            PeerEvent::ExtendedHandshakeCompleted => write!(f, "ExtendedHandshakeCompleted"),
            PeerEvent::StateChanged(state) => write!(f, "StateChanged({:?})", state),
            PeerEvent::RemoteAvailablePieces(pieces) => {
                write!(f, "RemoteAvailablePieces(len {})", pieces.len())
            }
            PeerEvent::RemoteUnavailablePieces(pieces) => {
                write!(f, "RemoteUnavailablePieces(len {})", pieces.len())
            }
            PeerEvent::PeersDiscovered(peers) => {
                write!(f, "PeersDiscovered(len {})", peers.len())
            }
            PeerEvent::PeersDropped(peers) => {
                write!(f, "PeersDropped(len {})", peers.len())
            }
        }
    }
}

impl Display for PeerEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PeerEvent::HandshakeCompleted => write!(f, "handshake completed"),
            PeerEvent::ExtendedHandshakeCompleted => write!(f, "extended handshake completed"),
            PeerEvent::StateChanged(state) => write!(f, "state changed to {}", state),
            PeerEvent::RemoteAvailablePieces(pieces) => {
                write!(f, "{} remote pieces have become available", pieces.len())
            }
            PeerEvent::RemoteUnavailablePieces(pieces) => {
                write!(f, "{} remote pieces have become unavailable", pieces.len())
            }
            PeerEvent::PeersDiscovered(peers) => {
                write!(f, "swarm discovered {} peers", peers.len())
            }
            PeerEvent::PeersDropped(peers) => {
                write!(f, "swarm dropped {} peers", peers.len())
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct PeerStats {
    /// The bytes that have been transferred to the peer.
    pub upload: usize,
    /// The bytes that contain actual piece data transferred to the peer.
    pub upload_useful: usize,
    /// The bytes that have been transferred from the peer.
    pub download: usize,
    /// The bytes that contain actual piece data transferred from the peer.
    pub download_useful: usize,
}

/// The client information of a connected peer.
#[derive(Debug, Display, Clone, PartialEq)]
#[display(fmt = "{}[{}:{}]", id, connection_protocol, addr)]
pub struct PeerClientInfo {
    /// The unique handle of the peer
    pub handle: PeerHandle,
    /// The unique peer id communicated with the remote peer
    pub id: PeerId,
    /// The remote peer address the client is connected to
    pub addr: SocketAddr,
    /// The connection direction of the peer client
    pub connection_type: ConnectionDirection,
    /// The connection protocol of the peer client used for communicating with the remote peer.
    pub connection_protocol: ConnectionProtocol,
}

impl PeerClientInfo {
    /// Get the canonical peer priority (BEP-40) of this peer compared against.
    pub fn peer_priority(&self, other: &Self) -> Option<u32> {
        PeerPriority::from((self, other)).take()
    }
}

impl From<(&PeerClientInfo, &PeerClientInfo)> for PeerPriority {
    fn from(value: (&PeerClientInfo, &PeerClientInfo)) -> Self {
        Self::from((&value.0.addr, &value.1.addr))
    }
}

/// The BitTorrent peer protocol implementation.
/// This [Peer] exchanges torrent data with remote peers through the specified BEP3 BitTorrent protocol.
///
/// It communicates with remote peers over TCP or uTP, see [PeerConn] for more info.
#[derive(Debug)]
pub struct BitTorrentPeer {
    inner: Arc<PeerContext>,
}

impl BitTorrentPeer {
    /// Create a new outgoing BitTorrent peer connection for the given network stream.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::net::SocketAddr;
    /// use std::sync::Arc;
    /// use std::time::Duration;
    /// use tokio::net::TcpStream;
    /// use tokio::runtime::Runtime;
    /// use popcorn_fx_torrent::torrent::peer::{BitTorrentPeer, PeerId, PeerStream, ProtocolExtensionFlags, Result};
    /// use popcorn_fx_torrent::torrent::peer::extension::Extension;
    /// use popcorn_fx_torrent::torrent::TorrentContext;
    ///
    /// async fn create_new_peer(torrent: Arc<TorrentContext>) -> Result<BitTorrentPeer> {
    ///     let peer_id = PeerId::new();
    ///     let addr = SocketAddr::from(([127,0,0,1], 6881));
    ///     let stream = PeerStream::Tcp(TcpStream::connect(addr).await?);
    ///     let protocol_extensions = ProtocolExtensionFlags::LTEP | ProtocolExtensionFlags::Fast;
    ///     let extensions : Vec<Box<dyn Extension>> = vec![];
    ///
    ///     BitTorrentPeer::new_outbound(
    ///         peer_id,
    ///         addr,
    ///         stream,
    ///         torrent,
    ///         protocol_extensions,
    ///         extensions,
    ///         Duration::from_secs(10),
    ///     ).await
    /// }
    /// ```
    pub async fn new_outbound(
        peer_id: PeerId,
        addr: SocketAddr,
        stream: PeerStream,
        torrent: Arc<TorrentContext>,
        protocol_extensions: ProtocolExtensionFlags,
        extensions: Extensions,
        timeout: Duration,
    ) -> Result<Self> {
        trace!("Trying to create outgoing peer connection to {}", addr);
        let connection: Box<dyn PeerConn> = match stream {
            PeerStream::Tcp(stream) => {
                Box::new(PeerConnection::<TcpStream>::new_tcp(peer_id, addr, stream))
            }
            PeerStream::Utp(stream) => {
                Box::new(PeerConnection::<UtpStream>::new_utp(peer_id, addr, stream))
            }
        };
        let connection_protocol = connection.protocol();

        Self::process_connection_stream(
            peer_id,
            addr,
            connection,
            ConnectionDirection::Outbound,
            connection_protocol,
            torrent,
            protocol_extensions,
            extensions,
            timeout,
        )
        .await
    }

    /// Try to accept a new incoming BitTorrent peer connection for the given network stream.
    pub async fn new_inbound(
        peer_id: PeerId,
        addr: SocketAddr,
        stream: PeerStream,
        torrent: Arc<TorrentContext>,
        protocol_extensions: ProtocolExtensionFlags,
        extensions: Extensions,
        timeout: Duration,
    ) -> Result<Self> {
        let connection: Box<dyn PeerConn> = match stream {
            PeerStream::Tcp(stream) => {
                Box::new(PeerConnection::<TcpStream>::new_tcp(peer_id, addr, stream))
            }
            PeerStream::Utp(stream) => {
                Box::new(PeerConnection::<UtpStream>::new_utp(peer_id, addr, stream))
            }
        };
        let connection_protocol = connection.protocol();

        trace!("Trying to receive incoming peer connection from {}", addr);
        select! {
            _ = time::sleep(timeout) => {
                Err(Error::Io(io::Error::new(io::ErrorKind::TimedOut, format!("connection from {} timed out", addr))))
            },
            result = Self::process_connection_stream(
                peer_id,
                addr,
                connection,
                ConnectionDirection::Inbound,
                connection_protocol,
                torrent,
                protocol_extensions,
                extensions,
                timeout,
            ) => result
        }
    }

    /// Get the connection type of the peer.
    ///
    /// # Returns
    ///
    /// Returns the connection type of the peer.
    pub fn connection_type(&self) -> ConnectionDirection {
        self.inner.client.connection_type
    }

    /// Retrieve the remote peer id.
    /// This is only available after the handshake with the peer has been completed.
    ///
    /// # Returns
    ///
    /// Returns the remote peer id when the handshake has been completed, else `None`.
    pub async fn remote_id(&self) -> Option<PeerId> {
        self.inner.remote_id().await
    }

    /// Get the remote peer information.
    /// This is only available after the handshake with the peer has been completed.
    ///
    /// # Returns
    ///
    /// Returns the remote peer information when the handshake has been completed, else `None`.
    pub async fn remote_peer(&self) -> Option<RemotePeer> {
        self.inner.remote.read().await.as_ref().map(|e| e.clone())
    }

    /// Get the known supported extensions of the remote peer.
    /// This might still be `None` when the handshake with the peer has not been completed yet.
    ///
    /// # Returns
    ///
    /// Returns the supported extensions of the remote peer.
    pub async fn remote_supported_extensions(&self) -> ProtocolExtensionFlags {
        let mutex = self.inner.remote.read().await;
        mutex
            .as_ref()
            .map(|e| e.protocol_extensions.clone())
            .unwrap_or(ProtocolExtensionFlags::None)
    }

    /// Get the known extension registry of the remote peer.
    /// This might still be `None` when the handshake with the peer has not been completed yet.
    ///
    /// # Returns
    ///
    /// Returns the extension registry of the remote peer.
    pub async fn remote_extension_registry(&self) -> Option<ExtensionRegistry> {
        let mutex = self.inner.remote.read().await;
        mutex.as_ref().map(|e| e.extensions.clone())
    }

    /// Get the remote peer choke state.
    pub async fn remote_choke_state(&self) -> ChokeState {
        self.inner.remote_choke_state.read().await.clone()
    }

    /// Get the interested state of the remote peer.
    pub async fn remote_interest_state(&self) -> InterestState {
        self.inner.remote_interest_state.read().await.clone()
    }

    /// Verify if the remote peer has the given piece.
    ///
    /// # Arguments
    ///
    /// * `piece` - The piece index that should be checked.
    ///
    /// # Returns
    ///
    /// Returns true when the remote peer has the piece available, else false.
    pub async fn remote_has_piece(&self, piece: PieceIndex) -> bool {
        let mutex = self.inner.remote_pieces.read().await;
        mutex.get(piece as usize).unwrap_or(false)
    }

    /// Check if the remote peer is a seed.
    /// This means that the remote peer has all pieces available and is seeding the torrent.
    pub async fn is_seed(&self) -> bool {
        self.inner.is_seed().await
    }

    /// Retrieve the torrent info hash.
    /// This info hash is used during the handshake with the peer and is immutable for the
    /// lifetime of the peer connection.
    pub async fn info_hash(&self) -> Result<InfoHash> {
        Ok(self
            .inner
            .torrent
            .metadata_lock()
            .read()
            .await
            .info_hash
            .clone())
    }

    /// Get the state of the peer.
    pub async fn state(&self) -> PeerState {
        self.inner.state().await
    }

    /// Get the client choke state of the peer.
    pub async fn choke_state(&self) -> ChokeState {
        self.inner.choke_state().await
    }

    /// Verify if the peer supports the given extension name with the remote peer.
    /// There is a plausibility for a "false-negative" when the extended handshake has not yet been executed.
    ///
    /// # Arguments
    ///
    /// * `extension_name` - The name of the extension to check for
    ///
    /// # Returns
    ///
    /// Returns true when the extension is supported, else false
    pub async fn supports_extension(&self, extension_name: ExtensionName) -> bool {
        // both the remote peer and this peer should support the given extension name
        self.inner
            .remote_extension_registry()
            .await
            .iter()
            .find(|e| e.contains_key(extension_name.as_str()))
            .is_some()
            && self
                .inner
                .extensions
                .iter()
                .find(|e| e.name() == extension_name)
                .is_some()
    }

    async fn send_initial_messages(&self) -> Result<()> {
        let mut is_fast_have_sent = false;

        // the extended handshake should be sent immediately after the standard bittorrent handshake to any peer that supports the extension protocol
        if self
            .inner
            .is_protocol_enabled(ProtocolExtensionFlags::LTEP)
            .await
        {
            trace!("Peer {} exchanging extended handshake", self);
            self.inner
                .send_command_event(PeerCommandEvent::State(PeerState::Handshake));
            if let Err(e) = self.inner.send_extended_handshake().await {
                warn!("Peer {} failed to send extended handshake, {}", self, e);
                // remove the LTEP extension flag from the remote peer
                // as the extended handshake has failed to complete
                if let Some(mutex) = self.inner.remote.write().await.as_mut() {
                    mutex.protocol_extensions &= !ProtocolExtensionFlags::LTEP;
                }
            }
        }

        // check if the fast protocol is enabled
        // if so, we send the initial fast messages to the remote peer
        let bitfield = self.inner.torrent.piece_bitfield().await;
        let is_bitfield_known = bitfield.len() > 0;
        let is_fast_enabled = self
            .inner
            .is_protocol_enabled(ProtocolExtensionFlags::Fast)
            .await;
        if is_fast_enabled && is_bitfield_known {
            let mut message: Option<Message> = None;
            let is_metadata_known = self
                .inner
                .torrent
                .metadata_lock()
                .read()
                .await
                .info
                .is_some();

            if is_metadata_known && bitfield.all() {
                message = Some(Message::HaveAll);
            } else if !is_metadata_known || bitfield.none() {
                message = Some(Message::HaveNone);
            }

            if let Some(message) = message {
                let message_type = format!("{:?}", message);
                match (&self.inner).send(message).await {
                    Ok(_) => {
                        is_fast_have_sent = true;
                        debug!("Peer {} sent message {}", self, message_type);
                    }
                    Err(e) => {
                        warn!(
                            "Peer {} failed to send message {}, {}",
                            self, message_type, e
                        );
                        self.inner
                            .send_command_event(PeerCommandEvent::State(PeerState::Error));
                    }
                }
            }
        }

        // we try to send the bitfield with completed pieces if none of the initial fast messages have been sent
        // this is only done if at least one piece is completed
        if !is_fast_have_sent && is_bitfield_known && bitfield.any() {
            let message = Message::Bitfield(bitfield.clone());
            let message_type = format!("{:?}", message);
            match self.inner.send(message).await {
                Ok(_) => debug!("Peer {} sent message {}", self, message_type),
                Err(e) => {
                    warn!("Peer {} failed to send bitfield message, {}", self, e);
                    self.inner
                        .send_command_event(PeerCommandEvent::State(PeerState::Error));
                }
            }
        }

        // store the bitfield of the torrent as initial state
        *self.inner.client_pieces.write().await = bitfield;

        // request missing hashes if needed
        if self.inner.should_request_hashes().await {
            self.inner
                .send_command_event(PeerCommandEvent::RequestMissingHashes);
        }

        self.inner
            .send_command_event(PeerCommandEvent::State(PeerState::Idle));
        Ok(())
    }

    async fn process_connection_stream(
        peer_id: PeerId,
        addr: SocketAddr,
        connection: Box<dyn PeerConn>,
        connection_type: ConnectionDirection,
        connection_protocol: ConnectionProtocol,
        torrent: Arc<TorrentContext>,
        protocol_extensions: ProtocolExtensionFlags,
        extensions: Extensions,
        timeout: Duration,
    ) -> Result<Self> {
        let (event_sender, event_receiver) = unbounded_channel();
        let extension_registry = Self::create_extension_registry(&extensions);
        let peer_handle = PeerHandle::new();
        let total_pieces = torrent.total_pieces().await;
        let client = PeerClientInfo {
            handle: peer_handle,
            id: peer_id,
            addr,
            connection_type,
            connection_protocol,
        };
        let inner = Arc::new(PeerContext {
            client,
            // the remote information is unknown until the handshake has been completed
            remote: RwLock::new(None),
            torrent,
            state: RwLock::new(PeerState::Handshake),
            protocol_extensions,
            // connections should always start in the choked state
            client_choke_state: RwLock::new(ChokeState::Choked),
            remote_choke_state: RwLock::new(ChokeState::Choked),
            // connections should always start in the not interested state
            client_interest_state: RwLock::new(InterestState::NotInterested),
            remote_interest_state: RwLock::new(InterestState::NotInterested),
            extensions,
            extension_registry,
            client_pieces: RwLock::new(BitVec::from_elem(total_pieces, false)),
            remote_pieces: RwLock::new(BitVec::from_elem(total_pieces, false)),
            remote_fast_pieces: RwLock::new(BitVec::from_elem(total_pieces, false)),
            // create new peer request buffers which are not running as the peer connection starts in the state choked
            client_pending_requests: RwLock::new(HashMap::with_capacity(0)),
            client_pending_request_permits: Mutex::new(HashMap::with_capacity(0)),
            remote_pending_requests: RwLock::new(Vec::with_capacity(0)),
            remote_pending_request_permit: Mutex::new(None),
            connection,
            incoming_data_stats: RwLock::new(PeerTransferStats::default()),
            outgoing_data_stats: RwLock::new(PeerTransferStats::default()),
            event_sender,
            callbacks: MultiThreadedCallback::new(),
            cancellation_token: CancellationToken::new(),
            timeout,
        });
        let peer = Self { inner };

        if connection_type == ConnectionDirection::Outbound {
            // as this is an outgoing connection, we're the once who initiate the handshake
            peer.inner.send_handshake().await?;
        }

        // retrieve the incoming handshake from the reader
        // as the handshake is always 68 bytes long, we request a buffer of 68 bytes from the reader
        trace!("Peer {} is awaiting the remote handshake", peer);
        let handshake =
            Self::try_receive_handshake(&peer.inner.client.addr, &peer.inner.connection, timeout)
                .await?;
        peer.inner.validate_handshake(handshake).await?;

        if connection_type == ConnectionDirection::Inbound {
            // as this is an incoming connection, we need to send our own handshake after receiving the peer handshake
            peer.inner.send_handshake().await?;
        }

        // start the main loop of the inner peer
        let main_loop = peer.inner.clone();
        let torrent_receiver = peer.inner.torrent.subscribe();
        tokio::spawn(async move { main_loop.start(event_receiver, torrent_receiver).await });

        peer.send_initial_messages().await?;
        Ok(peer)
    }

    /// Try to receive/read the incoming handshake from the remote peer.
    async fn try_receive_handshake(
        addr: &SocketAddr,
        connection: &Box<dyn PeerConn>,
        timeout: Duration,
    ) -> Result<Handshake> {
        select! {
            _ = time::sleep(timeout) => Err(Error::Handshake(
                addr.clone(),
                format!("handshake has timed out after {}.{:03} seconds", timeout.as_secs(), timeout.subsec_millis())
            )),
            result = connection.recv() => {
                if let Some(message) = result {
                    match message {
                        PeerResponse::Handshake(handshake) => Ok(handshake),
                        PeerResponse::Error(e) => Err(e),
                        PeerResponse::Closed => Err(Error::Closed),
                        _ => Err(Error::Handshake(addr.clone(), "invalid handshake received".to_string())),
                    }
                } else {
                    Err(Error::Closed)
                }
            },
        }
    }

    /// Create an extension registry for the given extensions.
    ///
    /// # Arguments
    ///
    /// * `extensions` - The extensions which should be registered in the registry.
    ///
    /// # Returns
    ///
    /// Returns the created extension registry for the given extensions.
    fn create_extension_registry(extensions: &Extensions) -> ExtensionRegistry {
        let mut extension_index = 0u8;

        extensions
            .iter()
            .map(|e| {
                extension_index += 1;
                (e.name().to_string(), extension_index)
            })
            .collect()
    }
}

#[async_trait]
impl Peer for BitTorrentPeer {
    fn handle(&self) -> PeerHandle {
        self.inner.client.handle
    }

    fn handle_as_ref(&self) -> &PeerHandle {
        &self.inner.client.handle
    }

    fn client(&self) -> PeerClientInfo {
        self.inner.client.clone()
    }

    fn addr(&self) -> SocketAddr {
        self.inner.addr()
    }

    fn addr_as_ref(&self) -> &SocketAddr {
        self.inner.addr_as_ref()
    }

    async fn state(&self) -> PeerState {
        self.inner.state().await
    }

    async fn stats(&self) -> PeerStats {
        self.inner.stats().await
    }

    async fn remote_piece_bitfield(&self) -> BitVec {
        self.inner.remote_piece_bitfield().await
    }

    fn notify_piece_availability(&self, pieces: Vec<PieceIndex>) {
        self.inner
            .send_command_event(PeerCommandEvent::ClientHasPieces(pieces))
    }

    async fn close(&self) {
        self.inner.close(CloseReason::Client).await
    }
}

impl Callback<PeerEvent> for BitTorrentPeer {
    fn subscribe(&self) -> Subscription<PeerEvent> {
        self.inner.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<PeerEvent>) {
        self.inner.subscribe_with(subscriber)
    }
}

impl Display for BitTorrentPeer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl PartialEq for BitTorrentPeer {
    fn eq(&self, other: &Self) -> bool {
        self.inner.client == other.inner.client
    }
}

/// Information about transferred data over the peer connection.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DataTransferStats {
    /// The total amount of bytes that have been transferred
    pub transferred_bytes: usize,
    /// The time it took in micro seconds to transfer the bytes
    pub elapsed_micro: u128,
}

impl DataTransferStats {
    /// Get the rate of bytes transferred per second.
    pub fn rate(&self) -> u64 {
        calculate_byte_rate(self.transferred_bytes, self.elapsed_micro)
    }
}

/// The reason a peer connection is being closed
#[derive(Debug, Display, PartialEq)]
pub(crate) enum CloseReason {
    /// The peer connection has been closed by the client
    #[display(fmt = "client closed connection")]
    Client,
    /// The peer connection has been closed by the remote peer
    #[display(fmt = "remote closed connection")]
    Remote,
    /// The client has closed the connection due to an invalid received fast protocol message
    #[display(fmt = "invalid fast protocol message received")]
    FastProtocol,
}

#[derive(Debug, Clone)]
struct PeerTransferStats {
    /// The amount of bytes that have been transferred.
    transferred_bytes: usize,
    /// The actual useful bytes that have been transferred.
    /// This only counts the actual piece payload data that has been transferred and excludes everything of the Bittorrent message protocol.
    transferred_bytes_useful: usize,
    /// The total amount of bytes that have been transferred during the lifetime of the connection.
    total_transferred_bytes: u64,
    /// The actual useful total bytes that have been transferred during the lifetime of the connection.
    /// This only counts the actual piece payload data that has been transferred and excludes everything of the Bittorrent message protocol.
    total_transferred_bytes_useful: u64,
}

impl Default for PeerTransferStats {
    fn default() -> Self {
        Self {
            transferred_bytes: 0,
            transferred_bytes_useful: 0,
            total_transferred_bytes: 0,
            total_transferred_bytes_useful: 0,
        }
    }
}

/// The piece that should be requested from the remote peer.
#[derive(Debug)]
pub struct RequestPieceData {
    /// The piece index to request
    pub piece: PieceIndex,
    /// The acquired permit from the torrent to download the data
    pub permit: OwnedSemaphorePermit,
}

impl PartialEq for RequestPieceData {
    fn eq(&self, other: &Self) -> bool {
        self.piece.eq(&other.piece)
    }
}

/// The internal peer command events which are executed on the main loop of the peer.
/// These can be used to offload async operations to the main loop.
#[derive(Debug, PartialEq)]
pub enum PeerCommandEvent {
    /// Indicates that the torrent has completed one or more pieces and the remote peer needs to be notified
    ClientHasPieces(Vec<PieceIndex>),
    /// Indicates that the choke state of the peer needs to be changed
    ClientChokeState(ChokeState),
    /// Indicates that the state if the peer needs to be changed
    State(PeerState),
    /// Indicates that the remote peer wants to receive piece data
    RemoteRequest(Request),
    /// Indicates that a peer event has occurred and the extensions need to be triggered
    Event(PeerEvent),
    /// Indicates that a certain message needs to be sent to the remote peer
    Send(Message),
    /// Indicates that the given piece data should be requested from the remote peer
    RequestPieceData(RequestPieceData),
    /// Indicates that wanted pieces by the torrent should be requested
    RequestWantedPieces,
    /// Indicates that an attempt should be made to request fast pieces
    RequestFastPieces,
    /// Indicates that the interest state should be determined for this client.
    /// It will check if the remote peer has pieces which are wanted by the torrent.
    DetermineClientInterestState,
    /// Indicates that a request upload permit should be obtained from the torrent
    RequestUploadPermit,
    /// Indicates that missing v2 hashes should be requested from the remote peer.
    RequestMissingHashes,
}

#[derive(Debug, Display)]
#[display(fmt = "{}", client)]
pub struct PeerContext {
    /// The client information of the peer
    client: PeerClientInfo,
    /// The remote peer information, known after the initial handshake.
    remote: RwLock<Option<RemotePeer>>,
    /// The immutable torrent this peer connection belongs to.
    /// This is a weak reference to the [Torrent] and might be invalid if the peer is kept alive for invalid reasons.
    torrent: Arc<TorrentContext>,
    /// The state of the client peer connection with the remote peer
    state: RwLock<PeerState>,
    /// The peer client supported/enabled protocol extensions
    protocol_extensions: ProtocolExtensionFlags,

    /// The client choke state
    client_choke_state: RwLock<ChokeState>,
    /// The choke state of the remote peer
    remote_choke_state: RwLock<ChokeState>,

    /// The client interest state for the pieces of the remote peer
    client_interest_state: RwLock<InterestState>,
    /// The interest state of the remote peer for our available pieces
    remote_interest_state: RwLock<InterestState>,

    /// The extensions which are support by the application
    /// These are immutable once the peer has been created
    extensions: Extensions,
    extension_registry: ExtensionRegistry,

    /// The torrent pieces
    client_pieces: RwLock<BitVec>,
    /// The pieces of the remote peer
    remote_pieces: RwLock<BitVec>,
    /// The allowed fast pieces of the remote peer
    remote_fast_pieces: RwLock<BitVec>,

    /// The clients pending requests to the remote peer
    /// The are requests which we've requested from the remote peer
    client_pending_requests: RwLock<HashMap<PieceIndex, Vec<Request>>>,
    /// The client pending permits of pending requests which have been sent to the remote peer.
    /// These are permits are based on the piece index and should be released once all requests of the piece are processed.
    client_pending_request_permits: Mutex<HashMap<PieceIndex, OwnedSemaphorePermit>>,
    /// The remote pending requests for this client.
    /// These are the requests the remote peer is interested in
    remote_pending_requests: RwLock<Vec<Request>>,
    /// The permit of the peer to upload pieces to the remote peer.
    remote_pending_request_permit: Mutex<Option<OwnedSemaphorePermit>>,

    /// The underlying peer connection
    connection: Box<dyn PeerConn>,

    /// The data transfer info of the incoming channel (from the remote peer)
    incoming_data_stats: RwLock<PeerTransferStats>,
    /// The data transfer info of the outgoing channel (to the remote peer)
    outgoing_data_stats: RwLock<PeerTransferStats>,

    /// The sender for internal events
    event_sender: UnboundedSender<PeerCommandEvent>,
    /// The callbacks which are triggered by this peer when an event is raised
    callbacks: MultiThreadedCallback<PeerEvent>,
    /// The timeout of the connection
    timeout: Duration,
    /// The cancellation token to cancel any async task within this peer on closure
    cancellation_token: CancellationToken,
}

impl PeerContext {
    /// Start the main loop of this peer.
    /// It handles the peer reader events and processing of the pending requests.
    async fn start(
        &self,
        mut event_receiver: UnboundedReceiver<PeerCommandEvent>,
        mut torrent_receiver: Subscription<TorrentEvent>,
    ) {
        let mut interval = time::interval(Duration::from_secs(2));

        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                _ = time::sleep(Duration::from_secs(KEEP_ALIVE_SECONDS)) => self.send_keep_alive().await,
                Some(event) = self.connection.recv() => self.handle_reader_event(event).await,
                Some(event) = event_receiver.recv() => self.handle_command_event(event).await,
                Some(event) = torrent_receiver.recv() => self.handle_torrent_event(&*event).await,
                _ = interval.tick() => {
                    self.check_for_wanted_pieces().await;
                    self.request_upload_permit_if_needed(false).await;
                },
            }
        }

        self.update_state(PeerState::Closed).await;
        trace!("Peer {} main loop ended", self);
    }

    /// Get the address of the remote peer.
    pub fn addr(&self) -> SocketAddr {
        self.client.addr
    }

    /// Get the address reference of the remote peer.
    pub fn addr_as_ref(&self) -> &SocketAddr {
        &self.client.addr
    }

    /// Get the underlying torrent of the peer.
    pub fn torrent(&self) -> &TorrentContext {
        &*self.torrent
    }

    /// Get the event sender of the peer.
    /// This sender can be used to trigger events on the peer.
    pub fn event_sender(&self) -> &UnboundedSender<PeerCommandEvent> {
        &self.event_sender
    }

    /// Get the state of the peer.
    pub async fn state(&self) -> PeerState {
        self.state.read().await.clone()
    }

    /// Get the connection transfer metrics of the peer.
    pub async fn stats(&self) -> PeerStats {
        let mut stats = PeerStats::default();

        {
            let mutex = self.outgoing_data_stats.read().await;
            stats.upload = mutex.transferred_bytes;
            stats.upload_useful = mutex.transferred_bytes_useful;
        }
        {
            let mutex = self.incoming_data_stats.read().await;
            stats.download = mutex.transferred_bytes;
            stats.download_useful = mutex.transferred_bytes_useful;
        }

        stats
    }

    /// Get the client choke state of the peer.
    pub async fn choke_state(&self) -> ChokeState {
        self.client_choke_state.read().await.clone()
    }

    /// Retrieve the remote peer id.
    ///
    /// # Returns
    ///
    /// Returns the remote peer id when known, else `None`.
    pub async fn remote_id(&self) -> Option<PeerId> {
        let mutex = self.remote.read().await;
        mutex.as_ref().map(|e| e.peer_id.clone())
    }

    /// Get the remote peer information.
    ///
    /// # Returns
    ///
    /// Returns the remote peer information when the handshake has been completed, else `None`.
    pub async fn remote_peer(&self) -> Option<RemotePeer> {
        let mutex = self.remote.read().await;
        mutex.as_ref().map(|e| e.clone())
    }

    /// Get the supported extension registry of the remote peer.
    ///
    /// # Returns
    ///
    /// Returns the extension registry of the remote peer if known, else `None`.
    pub async fn remote_extension_registry(&self) -> Option<ExtensionRegistry> {
        self.remote
            .read()
            .await
            .as_ref()
            .map(|e| e.extensions.clone())
    }

    /// Get the supported protocol extensions of the remote peer.
    /// This might still be `None` when the handshake with the peer has not been completed yet.
    pub async fn remote_protocol_extensions(&self) -> Option<ProtocolExtensionFlags> {
        self.remote
            .read()
            .await
            .as_ref()
            .map(|e| e.protocol_extensions.clone())
    }

    /// Get the available pieces of the remote peer as a bit vector.
    /// It might return an empty bit vector when the handshake has not been completed yet.
    pub async fn remote_piece_bitfield(&self) -> BitVec {
        self.remote_pieces.read().await.clone()
    }

    /// Check if the remote has all pieces available.
    /// The remote has all pieces if either an `HaveAll` message or completed `Bitfield` has been received by the remote.
    ///
    /// It returns true when the remote has all pieces and the metadata is known, else false.
    pub async fn remote_has_all_pieces(&self) -> bool {
        let remote_pieces = self.remote_pieces.read().await;
        let torrent_total_pieces = self.torrent.total_pieces().await;

        // the received bitfield can be greater than the actual total pieces due to byte alignment
        remote_pieces.len() >= torrent_total_pieces && remote_pieces.all()
    }

    /// Check if the remote peer is a seed.
    /// This means that the remote peer has all pieces available and is seeding the torrent.
    pub async fn is_seed(&self) -> bool {
        let mutex = self.remote_pieces.read().await;
        mutex.len() > 0 && mutex.all()
    }

    /// Check if a specific protocol extension is supported by the remote peer.
    /// If the client or the remote peer don't support the given extension, `false` is returned.
    pub async fn is_protocol_enabled(&self, extension: ProtocolExtensionFlags) -> bool {
        self.protocol_extensions.contains(extension)
            && self
                .remote
                .read()
                .await
                .as_ref()
                .map(|e| e.protocol_extensions.contains(extension))
                .unwrap_or(false)
    }

    /// Check if the client peer is currently interested in pieces from the remote peer.
    pub async fn is_client_interested(&self) -> bool {
        *self.client_interest_state.read().await == InterestState::Interested
    }

    /// Check if fast requests are allowed for the given piece.
    /// It returns true when fast requests are allowed for the given piece, else false.
    pub async fn is_fast_allowed(&self, piece: &PieceIndex) -> bool {
        self.remote_fast_pieces
            .read()
            .await
            .get(*piece)
            .unwrap_or(false)
    }

    /// Get the known metadata from the torrent.
    /// This info is requested from the torrent that created this peer.
    pub async fn metadata(&self) -> TorrentMetadata {
        self.torrent.metadata_lock().read().await.clone()
    }

    /// Check if the remote peer supports v2.
    pub async fn is_v2_supported(&self) -> bool {
        if let Some(remote) = self.remote.read().await.as_ref() {
            return remote.is_v2.clone();
        }

        false
    }

    /// Update the underlying torrent metadata.
    /// This method can be used by extensions to update the torrent metadata when the current
    /// connection is based on a magnet link.
    ///
    /// # Arguments
    ///
    /// * `metadata` - The new torrent metadata
    pub async fn update_metadata(&self, metadata: TorrentMetadataInfo) {
        self.torrent.add_metadata(metadata).await;
    }

    /// Get the client peer extensions registry.
    /// This is the registry of our own client.
    ///
    /// # Returns
    ///
    /// Returns a reference to the client extension registry.
    pub fn client_extension_registry(&self) -> &ExtensionRegistry {
        &self.extension_registry
    }

    /// Check if the remote peer has wanted piece data that are not yet being requested.
    /// If it least one piece is available by the remote peer and wanted by the torrent, it returns `true`.
    pub async fn has_wanted_piece(&self) -> bool {
        let mutex = self.client_pending_requests.read().await;
        let remote_has_all_pieces = self.remote_has_all_pieces().await;
        let remote_pieces = self.remote_pieces.read().await;
        let wanted_pieces = self.torrent.wanted_pieces().await;

        wanted_pieces
            .into_iter()
            .filter(|piece| remote_has_all_pieces || remote_pieces.get(*piece).unwrap_or(false))
            .any(|e| !mutex.contains_key(&e))
    }

    /// Handle events that are sent from the peer reader.
    async fn handle_reader_event(&self, event: PeerResponse) {
        match event {
            PeerResponse::Closed => self.close(CloseReason::Remote).await,
            PeerResponse::Message(message, data_transfer) => {
                self.update_read_data_transfer_stats(&message, data_transfer)
                    .await;

                if let Message::ExtendedPayload(extension_number, payload) = message {
                    trace!(
                        "Trying to find extension for extended payload number {}",
                        extension_number
                    );
                    if let Some(extension) = self.find_extension_by_number(extension_number).await {
                        let extension_name = extension.name();
                        trace!(
                            "Processing extension {} message payload for peer {}",
                            extension_name,
                            self
                        );
                        if let Err(e) = extension.handle(payload.as_ref(), self).await {
                            error!(
                                "Failed to process extension {} message for peer {}, {}",
                                extension_name, self, e
                            );
                        }
                    } else {
                        warn!(
                            "Received unsupported extension message of peer {} for extension number {}",
                            self, extension_number
                        );
                    }
                } else {
                    self.handle_received_message(message).await
                }
            }
            PeerResponse::Error(e) => {
                debug!("Peer {} encountered an error, {}", self, e);
                self.update_state(PeerState::Error).await;
            }
            _ => {}
        }
    }

    async fn handle_received_message(&self, message: Message) {
        debug!("Peer {} received remote message {:?}", self, message);
        match message {
            Message::KeepAlive => {
                trace!("Peer {} received keep alive", self);
            }
            Message::Choke => {
                self.update_remote_peer_choke_state(ChokeState::Choked)
                    .await
            }
            Message::Unchoke => {
                self.update_remote_peer_choke_state(ChokeState::UnChoked)
                    .await
            }
            Message::Interested => {
                self.update_remote_peer_interest_state(InterestState::Interested)
                    .await
            }
            Message::NotInterested => {
                self.update_remote_peer_interest_state(InterestState::NotInterested)
                    .await
            }
            Message::Have(piece) => self.remote_has_piece(piece as PieceIndex, true).await,
            Message::HaveAll => self.update_remote_fast_have(true).await,
            Message::HaveNone => self.update_remote_fast_have(false).await,
            Message::Bitfield(pieces) => self.update_remote_pieces(pieces).await,
            Message::Request(request) => self.add_remote_pending_request(request).await,
            Message::RejectRequest(request) => self.handle_rejected_client_request(request).await,
            Message::Cancel(request) => self.cancel_remote_pending_request(request).await,
            Message::Suggest(piece) => self.handle_piece_suggestion(piece as PieceIndex).await,
            Message::AllowedFast(piece) => self.remote_fast_piece(piece as PieceIndex).await,
            Message::Piece(piece) => self.handle_received_piece(piece).await,
            Message::ExtendedHandshake(handshake) => {
                self.update_extended_handshake(handshake).await
            }
            Message::HashRequest(request) => self.handle_hash_request(request).await,
            _ => warn!("Message not yet implemented for {:?}", message),
        }
    }

    /// Process a pending request requested by the remote peer.
    /// This tries to retrieve the requested data from the torrent.
    async fn handle_remote_pending_request(&self, request: Request) {
        // check if the request is still queued
        // if not, it has been cancelled in the meantime
        let mut mutex = self.remote_pending_requests.write().await;
        if let Some(position) = mutex.iter().position(|e| e == &request) {
            let _ = mutex.remove(position);
            drop(mutex);
        } else {
            debug!(
                "Remote pending {:?} is no longer queued for {}",
                request, self
            );
            return;
        }

        // check if the client is choked, if so, we reject the request
        // this can happen if the client choke's while the request was still queued in the command channel
        let is_client_choked = *self.client_choke_state.read().await == ChokeState::Choked;
        if is_client_choked {
            self.send_reject_request(request).await;
            return;
        }

        if self.torrent.has_piece(request.index).await {
            if *self.state.read().await != PeerState::Uploading {
                self.send_command_event(PeerCommandEvent::State(PeerState::Uploading));
            }

            let request_end = request.begin + request.length;
            match self
                .torrent
                .read_piece_bytes(request.index, request.begin..request_end)
                .await
            {
                Ok(data) => {
                    let data_len = data.len();
                    match self
                        .send(Message::Piece(Piece {
                            index: request.index,
                            begin: request.begin,
                            data,
                        }))
                        .await
                    {
                        Ok(_) => debug!(
                            "Peer {} sent piece {} data part (size {}) to remote peer",
                            self, request.index, data_len
                        ),
                        Err(e) => warn!(
                            "Peer {} failed to sent piece {} data part (size {}) to remote peer, {}",
                            self, request.index, data_len, e
                        ),
                    }
                }
                Err(e) => {
                    // FIXME: we currently reject requests if the piece if overlapping multiple files, but only 1 file is actually written to disk
                    warn!(
                        "Peer {} failed read piece {} data, {}",
                        self, request.index, e
                    );
                    self.send_reject_request(request).await;
                }
            }
        } else {
            let piece = request.index;
            debug!(
                "Peer {} is unable to provide piece {} data, torrent doesn't have the piece data available",
                self, piece
            );

            self.send_reject_request(request).await;
        }
    }

    /// Handle an event that has been triggered by the [Torrent].
    async fn handle_torrent_event(&self, event: &TorrentEvent) {
        match event {
            TorrentEvent::PiecesChanged(_) => {
                trace!("Peer {} updating client piece bitfield", self);
                // retrieve the torrent pieces bitfield and store it as the client bitfield
                let piece_bitfield = self.torrent.piece_bitfield().await;
                let bitfield_len = piece_bitfield.len();
                *self.client_pieces.write().await = piece_bitfield;

                // extend the remote pieces bitfield if needed
                let mut mutex = self.remote_pieces.write().await;
                if mutex.len() < bitfield_len {
                    let additional_len = bitfield_len - mutex.len();
                    mutex.extend(vec![false; additional_len]);
                }

                self.send_command_event(PeerCommandEvent::DetermineClientInterestState);
                self.send_command_event(PeerCommandEvent::RequestFastPieces);
            }
            TorrentEvent::PiecePrioritiesChanged => {
                self.send_command_event(PeerCommandEvent::DetermineClientInterestState);
                self.send_command_event(PeerCommandEvent::RequestFastPieces);
            }
            TorrentEvent::OptionsChanged => {
                self.send_command_event(PeerCommandEvent::DetermineClientInterestState);
                self.send_command_event(PeerCommandEvent::RequestFastPieces);
                self.request_upload_permit_if_needed(false).await;
            }
            _ => {}
        }
    }

    /// Process a request which has been rejected by the remote peer.
    /// This can be the case when we've request piece data that is no longer available, or the remote peer cannot serve it at the moment.
    async fn handle_rejected_client_request(&self, request: Request) {
        debug!("Peer {} remote rejected request {:?}", self, request);
        self.remove_client_pending_request(&request).await;
        self.torrent
            .pending_request_rejected(request.index, request.begin, &self.client)
            .await;
    }

    /// Handle a received piece data message
    async fn handle_received_piece(&self, piece: Piece) {
        let piece_index = &piece.index;
        let request: Option<Request> = self.remove_client_pending_request(&piece.request()).await;

        if let Some(request) = request {
            trace!("Received piece data for {:?} from {}", request, self);
            if let Some(part) = self.torrent.piece_part(piece.index, piece.begin).await {
                let data_size = piece.data.len();
                if part.length == data_size {
                    self.torrent.piece_part_completed(part, piece.data);
                } else {
                    debug!(
                    "Peer {} received invalid piece part {:?} data, received data length {}, expected length {}",
                        self,
                        part,
                        piece.data.len(),
                        data_size
                    );

                    self.release_client_pending_request_permit(piece_index)
                        .await;
                    self.torrent.invalid_piece_data_received(part, &self.client);
                    self.check_for_wanted_pieces().await;
                }
            } else {
                debug!(
                    "Received piece {} data from peer {} for a part that is unknown to the torrent",
                    piece.index, self
                );
                self.release_client_pending_request_permit(piece_index)
                    .await;
            }
        } else {
            debug!(
                "Received piece {} data from peer {} for an unwanted (not queued) request",
                piece.index, self
            );
        }
    }

    /// Handle a received hash request from the remote peer.
    async fn handle_hash_request(&self, _request: HashRequest) {
        // check if the torrent hash is a v2
        let metadata = self.torrent.metadata().await;
        let metadata_version = metadata.metadata_version().unwrap_or(0);
        if metadata_version != 2 {
            warn!(
                "Peer {} is unable to handle hash request for torrent with metadata version {}",
                self, metadata_version
            );
            return;
        }
    }

    /// Handle an internal peer command event.
    async fn handle_command_event(&self, event: PeerCommandEvent) {
        trace!("Peer {} handling command event {:?}", self, event);
        match event {
            PeerCommandEvent::ClientHasPieces(pieces) => {
                self.update_client_piece_availability(pieces).await
            }
            PeerCommandEvent::ClientChokeState(state) => {
                self.update_client_choke_state(state).await
            }
            PeerCommandEvent::State(state) => self.update_state(state).await,
            PeerCommandEvent::RemoteRequest(request) => {
                self.handle_remote_pending_request(request).await
            }
            PeerCommandEvent::Event(event) => self.inform_extensions_of_event(event).await,
            PeerCommandEvent::Send(message) => {
                if let Err(e) = self.send(message).await {
                    warn!("Failed to send message to peer {}, {}", self, e);
                }
            }
            PeerCommandEvent::RequestPieceData(piece) => self.request_piece_data(piece).await,
            PeerCommandEvent::RequestWantedPieces => self.request_wanted_pieces().await,
            PeerCommandEvent::RequestFastPieces => self.request_fast_pieces().await,
            PeerCommandEvent::DetermineClientInterestState => {
                self.determine_client_interest_state().await
            }
            PeerCommandEvent::RequestUploadPermit => self.request_upload_permit().await,
            PeerCommandEvent::RequestMissingHashes => self.request_missing_hashes().await,
        }
    }

    /// Check if the remote peer has at least one wanted piece available.
    /// If so, trigger the necessary commands to retrieve this piece.
    async fn check_for_wanted_pieces(&self) {
        if !self.torrent.is_download_allowed().await || self.torrent.is_completed().await {
            return;
        }

        let has_wanted_pieces = self.has_wanted_piece().await;
        let pending_requests = self.client_pending_requests.read().await.len();
        let state = self.torrent.state().await;
        trace!(
            "Peer {} has {} pending requests, wanted pieces {:?}, torrent state {}",
            self,
            pending_requests,
            has_wanted_pieces,
            state
        );
        if pending_requests < MAX_PENDING_PIECES && has_wanted_pieces {
            let is_client_interested = self.is_client_interested().await;
            if !is_client_interested {
                self.send_command_event(PeerCommandEvent::DetermineClientInterestState);
            }

            let is_remote_unchoked = *self.remote_choke_state.read().await == ChokeState::UnChoked;
            if is_remote_unchoked {
                self.send_command_event(PeerCommandEvent::RequestWantedPieces);
            }
        }
    }

    /// Check if v2 hashes should be requested for the current torrent.
    async fn should_request_hashes(&self) -> bool {
        let metadata = self.torrent.metadata_lock().read().await;
        if let Some(metadata_version) = metadata.metadata_version() {
            return metadata_version == 2 && self.is_v2_supported().await;
        }

        false
    }

    /// Request any missing hashes from the remote peer.
    async fn request_missing_hashes(&self) {
        if let Some(info) = self.torrent.metadata_lock().read().await.info.clone() {
            trace!("Peer {} is requesting missing v2 hashes", self);
            let piece_length = info.piece_length as usize;
            let _base_layer = (piece_length + LEAF_BLOCK_SIZE - 1) / LEAF_BLOCK_SIZE;
        } else {
            warn!(
                "Peer {} is unable to request missing hashes, torrent metadata info is unknown",
                self
            );
        }
    }

    /// Informs the enabled extensions of the peer event.
    async fn inform_extensions_of_event(&self, event: PeerEvent) {
        trace!(
            "Peer {} handling peer event for extensions with {:?}",
            self,
            event
        );
        let extensions = self.remote_extension_registry().await;

        if let Some(extensions) = extensions {
            let futures: Vec<_> = self
                .extensions
                .iter()
                .filter(|e| extensions.contains_key(&e.name().to_string()))
                .map(|e| e.on(&event, &self))
                .collect();
            let total_extensions = futures.len();
            trace!(
                "Peer {} is informing a total of {} extensions about event {:?}",
                self,
                total_extensions,
                event
            );
            let start_time = Instant::now();
            future::join_all(futures).await;
            let elapsed = start_time.elapsed();
            trace!(
                "Peer {} invoked {} extensions in {}.{:03}ms",
                self,
                total_extensions,
                elapsed.as_millis(),
                elapsed.subsec_micros() % 1000
            );
        }
    }

    /// Determine if our peer client is interested in pieces from the remote peer.
    async fn determine_client_interest_state(&self) {
        let state: InterestState;
        let is_download_allowed = self.torrent.is_download_allowed().await;

        // check if downloading is allowed by the torrent
        if is_download_allowed {
            let has_wanted_pieces = self.has_wanted_piece().await;
            if has_wanted_pieces {
                state = InterestState::Interested;
            } else {
                state = InterestState::NotInterested;
            }
        } else {
            state = InterestState::NotInterested;
        }

        self.update_client_interest_state(state).await;
    }

    /// Try to retrieve an upload permit from the torrent.
    /// If the peer already has an upload permit, this call will be a no-op.
    async fn request_upload_permit(&self) {
        if *self.client_choke_state.read().await == ChokeState::UnChoked {
            return;
        }

        let is_upload_allowed = self.torrent.is_upload_allowed().await;
        let has_upload_permit = self.remote_pending_request_permit.lock().await.is_some();

        if !is_upload_allowed || has_upload_permit {
            return;
        }

        trace!(
            "Peer {} is requesting an upload permit from the torrent",
            self
        );
        if let Some(permit) = self.torrent.request_upload_permit().await {
            // store the permit
            *self.remote_pending_request_permit.lock().await = Some(permit);
            debug!(
                "Peer {} acquired torrent {} upload permit",
                self, self.torrent
            );
            // unchoke the client peer to accept incoming requests from the remote peer
            // don't put the state change on the command channel as requests might have already been queued
            self.update_client_choke_state(ChokeState::UnChoked).await;
        } else {
            trace!(
                "Peer {} failed to request an upload permit from the torrent",
                self
            )
        }
    }

    /// Request an upload permit from the torrent if needed.
    ///
    /// If the remote peer is [InterestState::Interested] and the torrent allows uploads,
    /// then we queue the command to try to obtain an upload permit.
    async fn request_upload_permit_if_needed(&self, ignore_remote_interest_state: bool) {
        if *self.client_choke_state.read().await == ChokeState::UnChoked {
            return;
        }

        let is_interested = ignore_remote_interest_state
            || *self.remote_interest_state.read().await == InterestState::Interested;
        let has_no_upload_permit = self.remote_pending_request_permit.lock().await.is_none();
        let is_upload_allowed = self.torrent.is_upload_allowed().await;

        if is_interested && has_no_upload_permit && is_upload_allowed {
            self.send_command_event(PeerCommandEvent::RequestUploadPermit);
        }
    }

    async fn validate_handshake(&self, handshake: Handshake) -> Result<()> {
        let info_hash = self.torrent.metadata_lock().read().await.info_hash.clone();
        let mut v2_enabled = false;
        let mut is_valid = false;
        trace!("Peer {} received handshake {:?}", self, handshake);

        // check if v2 support is enabled
        if self
            .protocol_extensions
            .contains(ProtocolExtensionFlags::SupportV2)
            && handshake
                .supported_extensions
                .contains(ProtocolExtensionFlags::SupportV2)
        {
            // use the v2 info hash for validation
            if let Some(v2_hash) = info_hash.v2_as_short() {
                trace!("Peer {} is validating v2 handshake {:?}", self, v2_hash);
                if v2_hash == handshake.info_hash.short_info_hash_bytes() {
                    debug!("Peer {} has successfully upgraded to v2", self);
                    v2_enabled = true;
                    is_valid = true;
                } else {
                    debug!(
                        "Peer {} failed to upgrade to v2, invalid v2 handshake, falling back to v1 handshake validation",
                        self
                    );
                }
            } else {
                warn!(
                    "Peer {} is unable to upgrade to v2, metadata v2 hash is missing",
                    self
                )
            }
        }

        // check if the v2 handshake didn't succeed and we're using v1 handshake validation
        if !is_valid && info_hash != handshake.info_hash {
            self.update_state(PeerState::Error).await;
            return Err(Error::Handshake(
                self.client.addr.clone(),
                "received incorrect info hash from peer".to_string(),
            ));
        }

        // store the remote peer information
        trace!(
            "Peer {} is updating remote peer information with {:?}",
            self,
            handshake
        );
        {
            let mut mutex = self.remote.write().await;
            *mutex = Some(RemotePeer {
                peer_id: handshake.peer_id,
                protocol_extensions: handshake.supported_extensions,
                extensions: ExtensionRegistry::default(),
                client_name: None,
                is_v2: v2_enabled,
            });
        }

        debug!(
            "Peer {} handshake has been validated, {:?}",
            self, handshake
        );
        self.invoke_event(PeerEvent::HandshakeCompleted);
        Ok(())
    }

    /// Updates the choke state of the client peer.
    pub async fn update_client_choke_state(&self, state: ChokeState) {
        // check if we're already in the expected state
        if *self.client_choke_state.read().await == state {
            return;
        }

        {
            let mut mutex = self.client_choke_state.write().await;
            *mutex = state;
        }

        let send_result: Result<()>;
        if state == ChokeState::Choked {
            send_result = self.send(Message::Choke).await;
            self.reject_remote_pending_requests().await;
            // remove the upload permit
            let _ = self.remote_pending_request_permit.lock().await.take();
        } else {
            send_result = self.send(Message::Unchoke).await;
        }

        if let Err(e) = send_result {
            debug!(
                "Peer {} failed to sent {:?} state update, {}",
                self, state, e
            );
            self.send_command_event(PeerCommandEvent::State(PeerState::Error));
            // remove the upload permit
            let _ = self.remote_pending_request_permit.lock().await.take();
            return;
        }

        debug!("Peer {} client entered {} state", self, state);
    }

    /// Updates the choke state of the remote peer.
    async fn update_remote_peer_choke_state(&self, state: ChokeState) {
        // update the choke state of the remote peer
        {
            let mut mutex = self.remote_choke_state.write().await;
            *mutex = state;
        }

        if state == ChokeState::Choked {
            // if the remote is choked and the fast protocol is disabled,
            // then all pending requests are implicitly rejected
            if !self.is_protocol_enabled(ProtocolExtensionFlags::Fast).await {
                self.client_pending_requests.write().await.clear();
            }
        } else if self.torrent.is_download_allowed().await {
            self.send_command_event(PeerCommandEvent::RequestWantedPieces);
        }

        trace!("Peer {} remote entered {} state", self, state);
    }

    /// Updates the interest state of the client peer.
    /// This will notify the remote peer about the new interest state of our client if it changed.
    pub async fn update_client_interest_state(&self, state: InterestState) {
        // check if we're already in the expected state
        if *self.client_interest_state.read().await == state {
            return;
        }

        {
            let mut mutex = self.client_interest_state.write().await;
            *mutex = state;
        }

        let send_result: Result<()>;
        if state == InterestState::NotInterested {
            send_result = self.send(Message::NotInterested).await;
        } else {
            send_result = self.send(Message::Interested).await;
        }

        if let Err(e) = send_result {
            debug!(
                "Failed to send state {} to remote peer {}, {}",
                state, self, e
            );
        } else {
            debug!("Peer {} client entered {} state", self, state);
        }
    }

    /// Updates the interest state of the remote peer.
    async fn update_remote_peer_interest_state(&self, state: InterestState) {
        let mut mutex = self.remote_interest_state.write().await;
        if state == *mutex {
            return;
        }

        *mutex = state;
        drop(mutex);
        debug!("Peer {} remote entered {} state", self, state);

        // if the remote peer is no longer interested
        // choke the client so that another peer can obtain the permit
        if state == InterestState::NotInterested {
            self.send_command_event(PeerCommandEvent::ClientChokeState(ChokeState::Choked));
        }

        self.request_upload_permit_if_needed(false).await;
    }

    /// Updates the state of the peer.
    pub async fn update_state(&self, state: PeerState) {
        let mut mutex = self.state.write().await;
        if *mutex == state {
            return;
        }

        *mutex = state;
        debug!("Peer {} state updated to {:?}", self, state);

        self.invoke_event(PeerEvent::StateChanged(state));
    }

    /// Set the client peer as having the given pieces.
    /// This updates the peer client bitfield availability and informs the remote peer about the newly available pieces.
    async fn update_client_piece_availability(&self, pieces: Vec<PieceIndex>) {
        {
            let mut mutex = self.client_pieces.write().await;
            for piece in pieces.iter() {
                // we might not have the bitfield stored if it was unknown when this peer was created
                // if that's the case, copy the whole bitfield from the torrent instead
                if mutex.len() <= *piece {
                    *mutex = self.torrent.piece_bitfield().await;
                } else {
                    mutex.set(piece.clone(), true);
                }
            }
        }

        for piece in pieces.iter() {
            match self.send(Message::Have(*piece as u32)).await {
                Ok(_) => trace!(
                    "Peer {} notified remote about {} piece availability",
                    self,
                    piece
                ),
                Err(e) => warn!(
                    "Peer {} failed to notify remote peer about {} piece availability, {}",
                    self, piece, e
                ),
            }
        }

        self.request_upload_permit_if_needed(false).await;
    }

    /// Update the remote piece availabilities with given piece.
    ///
    /// The range of the piece will be checked against the known pieces of the torrent, if known.
    /// If the piece is out-of-range, the update will be ignored.
    pub async fn remote_has_piece(&self, piece: PieceIndex, has_piece: bool) {
        let total_pieces = self.torrent.total_pieces().await;
        let is_metadata_known = self.torrent.is_metadata_known().await;

        {
            let mut mutex = self.remote_pieces.write().await;
            // ensure the BitVec is large enough to accommodate the piece index
            if piece >= mutex.len() {
                let is_piece_bounds_known = is_metadata_known && total_pieces != 0;
                // check if the given piece index is out of bounds
                if is_piece_bounds_known && total_pieces < piece {
                    warn!(
                        "Peer {} received remote has piece index {} out of bounds ({})",
                        self,
                        piece,
                        mutex.len()
                    );
                    return;
                }

                // increase the size of the BitVec if metadata is still being retrieved
                let additional_len = piece as usize + 1 - mutex.len();
                mutex.extend(vec![false; additional_len]);
            }

            mutex.set(piece, has_piece);
        }

        if has_piece {
            if !self.is_client_interested().await {
                self.send_command_event(PeerCommandEvent::DetermineClientInterestState);
            }
            self.invoke_event(PeerEvent::RemoteAvailablePieces(vec![piece]));
        } else {
            // if fast is not enabled, we need to cancel any pending requests for the piece index
            // otherwise, it will be explicitly rejected by a fast message
            if !self.is_protocol_enabled(ProtocolExtensionFlags::Fast).await {
                self.release_client_pending_request_permit(&piece).await;
            }

            self.invoke_event(PeerEvent::RemoteUnavailablePieces(vec![piece]));
        }
    }

    /// Update the remote piece availability based on the supplied [BitVec].
    async fn update_remote_pieces(&self, pieces: BitVec) {
        {
            let mut mutex = self.remote_pieces.write().await;
            *mutex = pieces.clone();
            debug!(
                "Peer {} updated {}/{} remote available pieces",
                self,
                pieces.count_ones(),
                pieces.len()
            );
        }

        // notify subscribers about each available piece
        let piece_indexes: Vec<_> = pieces
            .into_iter()
            .enumerate()
            .filter(|(_, v)| *v)
            .map(|(piece, _)| piece as PieceIndex)
            .collect();

        if !piece_indexes.is_empty() {
            self.invoke_event(PeerEvent::RemoteAvailablePieces(piece_indexes));

            if !self.is_client_interested().await {
                self.send_command_event(PeerCommandEvent::DetermineClientInterestState);
            }
        }
    }

    async fn update_remote_fast_have(&self, have_all: bool) {
        // if the fast protocol is disabled, we should close the connection
        if !self
            .protocol_extensions
            .contains(ProtocolExtensionFlags::Fast)
        {
            warn!(
                "Fast protocol is disabled, closing connection with peer {}",
                self
            );
            self.close(CloseReason::FastProtocol).await;
            return;
        }

        let bitfield_len = self.torrent.total_pieces().await;
        self.update_remote_pieces(BitVec::from_elem(bitfield_len, have_all))
            .await;
        self.send_command_event(PeerCommandEvent::DetermineClientInterestState);
    }

    /// Add a pending request which is being requested by the remote peer.
    /// This request can however still be rejected on several conditions.
    async fn add_remote_pending_request(&self, request: Request) {
        let mut reject_request = false;
        let mut mutex = self.remote_pending_requests.write().await;
        // check if the request is a duplicate
        if mutex.contains(&request) {
            warn!("Peer {} requested duplicate request {:?}", self, request);
            if self.is_protocol_enabled(ProtocolExtensionFlags::Fast).await {
                self.close(CloseReason::FastProtocol).await;
            }

            return;
        }
        // check if the client peer is choked, if so, reject the request
        if *self.client_choke_state.read().await == ChokeState::Choked {
            debug!(
                "Peer {} received request for piece {} data while being choked",
                self, request.index
            );
            reject_request = true;
        }
        // check if the request chunk is larger than the allowed chunk size, if so, reject the request
        if request.length > MAX_PIECE_PART_SIZE {
            debug!(
                "Peer {} requested too large piece {} part, max length {}, requested length {}",
                self, request.index, MAX_PIECE_PART_SIZE, request.length
            );
            reject_request = true;
        }

        if reject_request {
            self.send_reject_request(request).await;
            return;
        }

        mutex.push(request.clone());
        self.send_command_event(PeerCommandEvent::RemoteRequest(request));
    }

    /// Try to cancel a remote pending request.
    /// This will remove the pending request from the queue if found.
    async fn cancel_remote_pending_request(&self, request: Request) {
        let mut mutex = self.remote_pending_requests.write().await;

        if let Some(position) = mutex.iter().position(|e| e == &request) {
            let request = mutex.remove(position);
            debug!("Cancelled remote pending {:?} for {}", request, self);
        } else {
            debug!(
                "Unable to cancel remote pending {:?} for {}, pending request not found",
                request, self
            );
        }
    }

    /// Reject any remaining pending requests of the remote peer.
    /// This should be called when our client peer enters the [ChokeState::Choked].
    async fn reject_remote_pending_requests(&self) {
        if self.is_protocol_enabled(ProtocolExtensionFlags::Fast).await {
            // reject any remaining pending requests as specified in BEP6 when entering the choked state
            // this should prevent race conditions in which case we're still sending some piece data while
            // the client is entering the choked state
            for request in self.remote_pending_requests.write().await.drain(..) {
                // offload the rejection to the event loop
                self.send_command_event(PeerCommandEvent::Send(Message::RejectRequest(request)));
            }
        } else {
            // clear any remaining pending requests as specified in BEP3 when entering the choked state
            self.remote_pending_requests.write().await.clear();
        }
    }

    /// Add the given piece to be executed as fast request.
    async fn remote_fast_piece(&self, piece: PieceIndex) {
        // When the fast extension is disabled, if a peer receives an Allowed Fast message then the peer MUST close the connection.
        if !self.is_protocol_enabled(ProtocolExtensionFlags::Fast).await {
            self.close(CloseReason::FastProtocol).await;
            return;
        }

        {
            let mut mutex = self.remote_fast_pieces.write().await;
            if mutex.len() < piece {
                // extend the bitfield
                let additional_len = piece as usize + 1 - mutex.len();
                mutex.extend(vec![false; additional_len]);
            }

            mutex.set(piece, true);
        }

        self.send_command_event(PeerCommandEvent::RequestFastPieces);
    }

    /// Handle a piece suggestion from the remote peer.
    /// This will request the given piece if the fast protocol is enabled, downloading is allowed and the piece is wanted by the torrent.
    async fn handle_piece_suggestion(&self, piece: PieceIndex) {
        // When the fast extension is disabled, if a peer receives a Suggest Piece message, the peer MUST close the connection.
        if !self.is_protocol_enabled(ProtocolExtensionFlags::Fast).await {
            self.close(CloseReason::FastProtocol).await;
            return;
        }

        // check if we're allowed to download pieces and that the given piece is wanted by the torrent
        let is_download_allowed = self.torrent.is_download_allowed().await;
        let is_piece_wanted = self.torrent.is_piece_wanted(&piece).await;
        if is_download_allowed && is_piece_wanted {
            if let Some(permit) = self.torrent.request_download_permit(&piece).await {
                self.send_command_event(PeerCommandEvent::RequestPieceData(RequestPieceData {
                    piece,
                    permit,
                }));
            }
        }
    }

    async fn update_extended_handshake(&self, handshake: ExtendedHandshake) {
        let mut mutex = self.remote.write().await;
        if let Some(remote) = mutex.as_mut() {
            remote.extensions = handshake.m;
            remote.client_name = handshake.client;
            let remote_info = format!("{:?}", remote);

            debug!(
                "Peer {} updated extended handshake information, {}",
                self, remote_info
            );
            self.invoke_event(PeerEvent::ExtendedHandshakeCompleted);
        } else {
            warn!(
                "Peer {} received extended handshake before the initial handshake was completed",
                self
            );
            self.close(CloseReason::Client).await;
        }
    }

    async fn update_read_data_transfer_stats(
        &self,
        message: &Message,
        data_transfer: DataTransferStats,
    ) {
        let mut mutex = self.incoming_data_stats.write().await;
        mutex.transferred_bytes += data_transfer.transferred_bytes;
        mutex.total_transferred_bytes += data_transfer.transferred_bytes as u64;

        if let Message::Piece(piece) = message {
            let data_size = piece.data.len();
            mutex.transferred_bytes_useful += data_size;
            mutex.total_transferred_bytes_useful += data_size as u64;
        }
    }

    async fn update_write_data_transfer_stats(
        &self,
        data_transfer: DataTransferStats,
        piece_data_size: Option<usize>,
    ) {
        let mut mutex = self.outgoing_data_stats.write().await;
        mutex.transferred_bytes += data_transfer.transferred_bytes;
        mutex.total_transferred_bytes += data_transfer.transferred_bytes as u64;

        if let Some(piece_data_size) = piece_data_size {
            mutex.transferred_bytes_useful += piece_data_size;
            mutex.total_transferred_bytes_useful += piece_data_size as u64;
        }
    }

    /// Try to request the piece data from the remote peer.
    ///
    /// ## Request permits
    ///
    /// A request permit is obtained from the torrent before trying to request the data from the remote peer.
    /// If no permit is available, the piece data won't be requested from the remote peer.
    ///
    /// ## Fast
    ///
    /// If the piece is allowed through the fast protocol, the request will be sent to the remote peer even if it's choked.
    /// This doesn't bypass the request permits, if no permit is available, then no request will be made to the remote peer.
    async fn request_piece_data(&self, request_data: RequestPieceData) {
        if !self.torrent.is_piece_wanted(&request_data.piece).await {
            trace!(
                "Piece {} is no longer wanted for {}",
                request_data.piece,
                self
            );
            return;
        }
        if self.is_piece_already_requested(&request_data.piece).await {
            trace!(
                "Piece {} is already being requested for {}",
                request_data.piece,
                self
            );
            return;
        }

        let permit = request_data.permit;
        let piece = self
            .torrent
            .pieces_lock()
            .read()
            .await
            .get(request_data.piece)
            .cloned();
        if let Some(piece) = piece {
            let mut sent_requests = 0;
            let requests: Vec<Request> = piece
                .parts_to_request()
                .into_iter()
                .map(|part| Request {
                    index: piece.index,
                    begin: part.begin,
                    length: part.length,
                })
                .collect();

            trace!(
                "Trying to request piece {} data for {}, {:?}",
                piece.index,
                self,
                requests
            );
            for request in requests {
                if self.send_pending_request(request).await {
                    sent_requests += 1;
                }
            }

            if sent_requests > 0 {
                // keep the permit as the data requested is in flight until completion, cancellation or rejection
                self.client_pending_request_permits
                    .lock()
                    .await
                    .insert(piece.index, permit);
                debug!(
                    "Peer {} requested piece {} data ({} pending requests)",
                    self, piece.index, sent_requests
                );
            } // otherwise, we'll drop the permit automatically as no data could be requested
        }
    }

    /// Try to request pieces that are wanted by the torrent and are available.
    /// Pieces that have already been requested will not be retried.
    async fn request_wanted_pieces(&self) {
        // this can happen if the torrent options have changed while the command was being queued
        if !self.torrent.is_download_allowed().await {
            trace!("Peer {} is no longer allowed to download data", self);
            return;
        }

        let client_pending_requests = self.client_pending_requests.read().await;
        let is_pending_requests_full = client_pending_requests.len() >= MAX_PENDING_PIECES;
        if is_pending_requests_full {
            trace!(
                "Peer {} no additional requests can be made, pending requests queue is full",
                self
            );
            return;
        }

        let remote_has_all_pieces = self.remote_has_all_pieces().await;
        let remote_pieces = self.remote_pieces.read().await;
        let wanted_pieces: Vec<PieceIndex> = self
            .torrent
            .wanted_request_pieces()
            .await
            .into_iter()
            // filter out the pieces the remote doesn't have
            .filter(|e| remote_has_all_pieces || remote_pieces.get(*e).unwrap_or(false))
            // filter out any pending requests which have already been sent
            .filter(|e| !client_pending_requests.contains_key(e))
            // take a max of X pieces
            .take(MAX_PENDING_PIECES.saturating_sub(client_pending_requests.len()))
            .collect();

        if !wanted_pieces.is_empty() {
            // try to request an upload permit if we're currently choked and want pieces from the remote peer
            self.request_upload_permit_if_needed(true).await;

            for piece in wanted_pieces {
                if let Some(permit) = self.torrent.request_download_permit(&piece).await {
                    self.send_command_event(PeerCommandEvent::RequestPieceData(RequestPieceData {
                        piece,
                        permit,
                    }))
                }
            }
        }
    }

    /// Try to request any fast pieces which have not yet been requested
    async fn request_fast_pieces(&self) {
        if !self.torrent.is_download_allowed().await {
            return;
        }

        let wanted_pieces = self.torrent.wanted_request_pieces().await;
        let client_pending_requests = self.client_pending_requests.read().await;
        let wanted_fast_pieces: Vec<PieceIndex> = self
            .remote_fast_pieces
            .read()
            .await
            .iter()
            .enumerate()
            // filter out the non-fast pieces
            .filter(|(_, value)| *value)
            // filter out any unwanted pieces
            .filter(|(piece, _)| wanted_pieces.contains(piece))
            // filter out any pending requests which have already been sent
            .filter(|(piece, _)| !client_pending_requests.contains_key(piece))
            // take a max of X pieces
            .take(MAX_PENDING_PIECES.saturating_sub(client_pending_requests.len()))
            .map(|(piece, _)| piece as PieceIndex)
            .collect();

        for piece in wanted_fast_pieces {
            if let Some(permit) = self.torrent.request_download_permit(&piece).await {
                self.send_command_event(PeerCommandEvent::RequestPieceData(RequestPieceData {
                    piece,
                    permit,
                }))
            }
        }
    }

    /// Check if the given piece is already being requested from the remote peer.
    /// It returns true if at least one pending piece part is pending for the given piece index.
    async fn is_piece_already_requested(&self, piece: &PieceIndex) -> bool {
        self.client_pending_requests
            .read()
            .await
            .get(piece)
            .map(|e| e.len() > 0)
            .unwrap_or(false)
    }

    /// Try to send the handshake information of our client peer to the remote peer.
    async fn send_handshake(&self) -> Result<()> {
        self.update_state(PeerState::Handshake).await;
        let info_hash = self.torrent.metadata_lock().read().await.info_hash.clone();
        let protocol_extensions = self.protocol_extensions;

        let handshake = Handshake::new(info_hash, self.client.id, protocol_extensions);
        debug!("Peer {} is sending handshake {:?}", self, handshake);
        match self
            .send_raw_bytes(TryInto::<Vec<u8>>::try_into(handshake)?, None)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                self.update_state(PeerState::Error).await;
                Err(e)
            }
        }
    }

    async fn send_extended_handshake(&self) -> Result<()> {
        let extension_registry = self.extension_registry.clone();
        let is_partial_seed = self.torrent.is_partial_seed().await;
        let message = Message::ExtendedHandshake(ExtendedHandshake {
            m: extension_registry,
            upload_only: is_partial_seed,
            client: Some(self.torrent.config_lock().read().await.client_name())
                .filter(|e| !e.is_empty())
                .map(|e| e.to_string()),
            regg: None,
            encryption: false,
            metadata_size: None,
            port: self.torrent.peer_port().map(|e| e as u32),
            your_ip: Some(CompactIp::from(&self.client.addr)),
            ipv4: None,
            ipv6: None,
        });

        debug!("Peer {} is sending extended handshake {:?}", self, message);
        self.send(message).await
    }

    /// Send the reject request to the remote peer.
    /// This is only executed if the fast protocol is enabled.
    async fn send_reject_request(&self, request: Request) {
        // if the fast protocol is disabled, then we don't send a reject
        if !self.is_protocol_enabled(ProtocolExtensionFlags::Fast).await {
            return;
        }

        let piece = request.index;
        match self.send(Message::RejectRequest(request)).await {
            Ok(_) => debug!("Peer {} rejected remote request {}", self, piece),
            Err(e) => warn!(
                "Peer {} failed to reject remote request {}, {}",
                self, piece, e
            ),
        }
    }

    /// Try to send the given message to the remote peer.
    pub async fn send(&self, message: Message) -> Result<()> {
        trace!("Peer {} trying to send message {:?}", self, message);
        let mut piece_data_size: Option<usize> = None;
        if let Message::Piece(piece) = &message {
            piece_data_size = Some(piece.data.len());
        }

        let message_bytes = TryInto::<Vec<u8>>::try_into(message)?;
        self.send_bytes(message_bytes, piece_data_size).await
    }

    /// Send the given message to the remote peer.
    /// This method will prefix the message bytes with the BigEndian length bytes of the given message.
    pub async fn send_bytes<T: AsRef<[u8]>>(
        &self,
        message: T,
        piece_data_size: Option<usize>,
    ) -> Result<()> {
        let msg_length = message.as_ref().len();
        let mut buffer = vec![0u8; 4];

        // write the length of the given message as BigEndian in the first 4 bytes
        BigEndian::write_u32(&mut buffer[..4], msg_length as u32);
        // append the given message bytes to the buffer
        buffer.extend_from_slice(message.as_ref());

        self.send_raw_bytes(buffer, piece_data_size).await
    }

    /// Send the given message bytes AS-IS to the remote peer.
    /// The given bytes should be a valid BitTorrent protocol message.
    async fn send_raw_bytes<T: AsRef<[u8]>>(
        &self,
        bytes: T,
        piece_data_size: Option<usize>,
    ) -> Result<()> {
        let msg_length = bytes.as_ref().len();

        let start_time = Instant::now();
        timeout(self.timeout, self.connection.write(bytes.as_ref())).await??;
        let elapsed = start_time.elapsed();

        // update the connection stats
        self.update_write_data_transfer_stats(
            DataTransferStats {
                transferred_bytes: msg_length,
                elapsed_micro: elapsed.as_micros(),
            },
            piece_data_size,
        )
        .await;

        Ok(())
    }

    /// Request piece data which is available from the remote peer.
    /// This will only be executed if:
    ///  - remote peer is not choked or fast is allowed
    ///  - request has not been requested before
    ///
    /// It returns `true` when the request has been sent to the remote peer, else `false`.
    async fn send_pending_request(&self, request: Request) -> bool {
        // if the remote peer is choked, but the fast protocol allows this request, the request will be allowed
        let is_remote_choked = *self.remote_choke_state.read().await == ChokeState::Choked;
        let is_fast_allowed = self.is_fast_allowed(&request.index).await;
        if !is_fast_allowed && is_remote_choked {
            trace!(
                "Peer {} tried to request piece {} (offset: {}) data from choked remote peer",
                self,
                request.index,
                request.begin
            );
            return false;
        }

        // check if the request is a duplicate
        let mut mutex = self.client_pending_requests.write().await;
        if mutex
            .get(&request.index)
            .map(|requests| requests.contains(&request))
            .unwrap_or(false)
        {
            debug!(
                "Peer {} tried to request duplicate piece {} (offset: {}) data",
                self, request.index, request.begin
            );
            return false;
        }

        if *self.state.read().await != PeerState::Downloading {
            self.send_command_event(PeerCommandEvent::State(PeerState::Downloading));
        }

        match self.send(Message::Request(request.clone())).await {
            Ok(_) => trace!("Peer {} sent request {:?}", self, request),
            Err(e) => {
                warn!(
                    "Peer {} failed request piece {} (offset: {}) data (fast: {:?}), {}",
                    self, request.index, request.begin, is_fast_allowed, e
                );
                return false;
            }
        }

        // store the pending request
        let requests = mutex.entry(request.index).or_insert(Vec::new());
        requests.push(request.clone());
        true
    }

    /// Send the keep alive message to the remote peer.
    pub async fn send_keep_alive(&self) {
        let message = Message::KeepAlive;

        match TryInto::<Vec<u8>>::try_into(message) {
            Ok(bytes) => {
                if let Err(e) = self.send_bytes(bytes, None).await {
                    warn!("Failed to send keep alive to peer {}, {}", self, e);
                }
            }
            Err(e) => warn!("Failed to parse keep alive message, {}", e),
        }
    }

    /// Try to remove the given request from the pending requests.
    /// This function should be called when piece data has been received or rejected for the given request.
    async fn remove_client_pending_request(&self, request: &Request) -> Option<Request> {
        let mut is_piece_completed = false;
        let piece_index = &request.index;
        let mut result: Option<Request> = None;

        let mut mutex = self.client_pending_requests.write().await;
        if let Some(requests) = mutex.get_mut(piece_index) {
            result = requests
                .iter()
                .position(|e| e == request)
                .map(|e| requests.remove(e));
            is_piece_completed = requests.is_empty();
        }
        drop(mutex);

        if !is_piece_completed {
            return result;
        }

        self.release_client_pending_request_permit(piece_index)
            .await;
        result
    }

    /// Release the permit behind the client pending piece data request.
    /// This will remove any remaining pending requests for the piece and release the permit unlock.
    async fn release_client_pending_request_permit(&self, piece: &PieceIndex) {
        let mut client_pending_requests = self.client_pending_requests.write().await;

        // remove the permit lock and entry
        client_pending_requests.remove(piece);
        self.client_pending_request_permits
            .lock()
            .await
            .remove(piece);
        let request_additional_pieces = client_pending_requests.len() < MAX_PENDING_PIECES;
        drop(client_pending_requests); // drop the mutex as `has_wanted_piece` needs it for calculation
        trace!("Peer {} released piece {} request permit", self, piece);

        if request_additional_pieces && self.has_wanted_piece().await {
            trace!("Peer {} has more wanted pieces from the remote peer", self);
            self.check_for_wanted_pieces().await;
        }
    }

    /// Find the supported extension from our own client extensions through the extensions number.
    /// This should be used when we've received an extended message from the remote peer.
    ///
    /// # Arguments
    ///
    /// * `extension_number` - The extensions number send by the remote peer.
    ///
    /// # Returns
    ///
    /// Returns a reference to the found client extension.
    pub async fn find_extension_by_number(
        &self,
        extension_number: ExtensionNumber,
    ) -> Option<&Box<dyn Extension>> {
        // search for the given extension, by extensions number, in our own supported extensions
        let extension_registry = self.client_extension_registry();
        if let Some(extension_name) = extension_registry
            .iter()
            .find(|(_, number)| extension_number == **number)
            .map(|(name, _)| name.clone())
        {
            return self.find_extension_by_name(extension_name.as_str());
        } else {
            let extensions = self.remote_extension_registry().await;
            debug!(
                "Extension number {} is not support by {}, supported remote {:?}",
                extension_number, self, extensions
            )
        }

        None
    }

    /// Find the supported extension from our own client extensions through the unique extension's name.
    /// This should be used when trying to send an extended message to the remote peer.
    ///
    /// # Arguments
    ///
    /// * `extension_name` - The name of the extension.
    ///
    /// # Returns
    ///
    /// Returns a reference to the found client extension.
    pub fn find_extension_by_name(&self, extension_name: &str) -> Option<&Box<dyn Extension>> {
        let extension_registry = self.client_extension_registry();
        if let Some(extension) = self.extensions.iter().find(|e| e.name() == extension_name) {
            return Some(extension);
        }

        warn!(
            "Extension name {} not found back for peer {}, supported {:?}",
            extension_name, self, extension_registry
        );
        None
    }

    /// Find the extensions number from our own client extensions through the unique extension's name.
    ///
    /// # Arguments
    ///
    /// * `extension_name` - The name of the extension.
    pub fn find_client_extension_number(&self, extension_name: &str) -> Option<ExtensionNumber> {
        let extension_registry = self.client_extension_registry();
        if let Some((_, number)) = extension_registry
            .iter()
            .find(|(name, _)| name.as_str() == extension_name)
        {
            return Some(number.clone());
        }

        None
    }

    /// Invoke an event on the peer instance.
    /// This will trigger the event for all enabled extensions.
    pub fn invoke_event(&self, event: PeerEvent) {
        self.send_command_event(PeerCommandEvent::Event(event.clone()));
        self.callbacks.invoke(event);
    }

    /// Close the connection of the peer.
    /// This cancels the main loop of the peer and notifies the parent torrent of the closure.
    pub(crate) async fn close(&self, reason: CloseReason) {
        debug!("Peer {} is closing, {}", self, reason);
        // cancel the main loop of the peer to stop any ongoing operation
        self.cancellation_token.cancel();
        // clear any permits as they cannot be completed from now on
        self.client_pending_requests.write().await.clear();
        self.client_pending_request_permits.lock().await.clear();
        // close underlying connection
        let _ = self.connection.close().await;
        // notify any subscribers
        self.update_state(PeerState::Closed).await;
        // notify the torrent that this peer is being closed
        self.torrent.notify_peer_closed(self.client.handle);
    }

    /// Publish a command event to the peer that will be processed by the main loop.
    fn send_command_event(&self, event: PeerCommandEvent) {
        if let Err(e) = self.event_sender.send(event) {
            debug!(
                "Failed to send internal peer command event for {}, {}",
                self, e
            );
        }
    }
}

impl Callback<PeerEvent> for PeerContext {
    fn subscribe(&self) -> Subscription<PeerEvent> {
        self.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<PeerEvent>) {
        self.callbacks.subscribe_with(subscriber)
    }
}

impl Drop for PeerContext {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
        trace!("Peer {} is being dropped", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::torrent::operation::{TorrentCreateFilesOperation, TorrentCreatePiecesOperation};
    use crate::torrent::peer::extension::metadata::MetadataExtension;
    use crate::torrent::peer::protocol::tests::UtpPacketCaptureExtension;
    use crate::torrent::peer::tests::create_utp_peer_pair;
    use crate::torrent::{
        TorrentConfig, TorrentFlags, TorrentOperation, TorrentOperationResult, TorrentState,
        DEFAULT_TORRENT_PROTOCOL_EXTENSIONS,
    };
    use crate::{create_peer_pair, create_torrent, create_utp_socket_pair};

    use popcorn_fx_core::{assert_timeout, init_logger};
    use tempfile::tempdir;
    use tokio::sync::mpsc::channel;

    #[tokio::test]
    async fn test_peer_new_tcp() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![]
        );
        let (outgoing, incoming) = create_peer_pair!(&torrent);

        let result = incoming.state().await;
        assert_ne!(PeerState::Error, result);
        assert_ne!(PeerState::Closed, result);

        incoming.close().await;
        let result = incoming.state().await;
        assert_eq!(PeerState::Closed, result);
        assert_timeout!(
            Duration::from_secs(1),
            PeerState::Closed == outgoing.state().await,
            "expected the outgoing connection to be closed"
        );
    }

    #[tokio::test]
    async fn test_peer_new_utp() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![]
        );
        let incoming_capture = UtpPacketCaptureExtension::new();
        let outgoing_capture = UtpPacketCaptureExtension::new();
        let (incoming_socket, outgoing_socket) = create_utp_socket_pair!(
            vec![Box::new(incoming_capture.clone())],
            vec![Box::new(outgoing_capture.clone())]
        );
        let (outgoing, incoming) = create_utp_peer_pair(
            &incoming_socket,
            &outgoing_socket,
            &torrent,
            &torrent,
            DEFAULT_TORRENT_PROTOCOL_EXTENSIONS(),
        )
        .await;

        let result = incoming.state().await;
        assert_ne!(PeerState::Error, result);
        assert_ne!(PeerState::Closed, result);

        incoming.close().await;
        let result = incoming.state().await;
        assert_eq!(PeerState::Closed, result);
        assert_timeout!(
            Duration::from_secs(1),
            PeerState::Closed == outgoing.state().await,
            "expected the outgoing connection to be closed"
        );
    }

    #[tokio::test]
    async fn test_peer_retrieve_metadata() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let source_torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![]
        );
        let target_torrent = create_torrent!(
            uri,
            temp_path,
            TorrentFlags::Metadata,
            TorrentConfig::default(),
            vec![]
        );
        let context = target_torrent.instance().unwrap();

        let (tx, mut rx) = channel(1);
        let mut receiver = target_torrent.subscribe();
        tokio::spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let TorrentEvent::MetadataChanged(_) = *event {
                        tx.send(()).await.unwrap();
                    }
                } else {
                    break;
                }
            }
        });

        // create a connection to the source torrent which has the metadata
        let peer_id = PeerId::new();
        let peer_addr = SocketAddr::from(([127, 0, 0, 1], source_torrent.peer_port().unwrap()));
        let stream = TcpStream::connect(peer_addr).await.unwrap();
        let peer = BitTorrentPeer::new_outbound(
            peer_id,
            peer_addr,
            PeerStream::Tcp(stream),
            context.clone(),
            context.protocol_extensions(),
            vec![Box::new(MetadataExtension::new())],
            Duration::from_secs(5),
        )
        .await
        .expect("expected the outbound connection to have been created");
        assert_timeout!(
            Duration::from_secs(1),
            PeerState::Handshake != peer.state().await,
            "expected the peer handshake to have been completed"
        );

        select! {
            _ = time::sleep(Duration::from_secs(5)) => assert!(false, "expected the metadata to have been retrieved"),
            result = rx.recv() => assert!(result.is_some(), "expected some metadata to have been retrieved"),
        }
    }

    #[tokio::test]
    async fn test_peer_has_wanted_piece() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let expected_pieces = vec![0, 1, 2];
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![]
        );
        let context = torrent.instance().unwrap();
        let (outgoing, incoming) = create_peer_pair!(&torrent);
        let incoming_context = &incoming.inner;

        // create the pieces for the torrent
        let operation = TorrentCreatePiecesOperation::new();
        let result = operation.execute(&context).await;
        assert_eq!(TorrentOperationResult::Continue, result);

        let (tx, mut rx) = channel(1);
        let mut receiver = incoming.subscribe();
        tokio::spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let PeerEvent::RemoteAvailablePieces(pieces) = (*event).clone() {
                        tx.send(pieces).await.unwrap();
                        break;
                    }
                } else {
                    break;
                }
            }
        });

        // notify the other peer we have "fake" pieces
        outgoing.notify_piece_availability(expected_pieces.clone());

        let result = select! {
            _ = time::sleep(Duration::from_secs(2)) => panic!("expected to have received the RemoteAvailablePieces event"),
            result = rx.recv() => result.unwrap(),
        };
        assert_eq!(vec![0], result);

        let result = incoming_context.has_wanted_piece().await;
        assert_eq!(true, result, "expected the remote to have wanted pieces");
    }

    #[tokio::test]
    async fn test_peer_torrent_pieces_changed() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![]
        );
        let context = torrent.instance().unwrap();
        let (outgoing, _incoming) = create_peer_pair!(&torrent);

        // create the pieces for the torrent
        let operation = TorrentCreatePiecesOperation::new();
        let result = operation.execute(&context).await;
        assert_eq!(TorrentOperationResult::Continue, result);

        // check if both the client & remote piece bitfield have been updated
        let torrent_bitfield = context.piece_bitfield().await;
        let peer_context = &outgoing.inner;
        assert_timeout!(
            Duration::from_secs(1),
            torrent_bitfield == *peer_context.client_pieces.read().await,
            "expected the peer client bitfield to match the torrent bitfield"
        );
        let remote_bitfield = peer_context.remote_piece_bitfield().await;
        assert_eq!(
            torrent_bitfield.len(),
            remote_bitfield.len(),
            "expected the remote bitfield to match the torrent bitfield length"
        );
    }

    #[tokio::test]
    async fn test_peer_torrent_validating_files() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let source_torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::UploadMode,
            TorrentConfig::default(),
            vec![|| Box::new(TorrentCreatePiecesOperation::new()), || {
                Box::new(TorrentCreateFilesOperation::new())
            }]
        );
        let target_torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::DownloadMode,
            TorrentConfig::default(),
            vec![|| Box::new(TorrentCreatePiecesOperation::new()), || {
                Box::new(TorrentCreateFilesOperation::new())
            }]
        );
        let target_torrent_context = target_torrent.instance().unwrap();

        // wait for the pieces/files to have been created
        let (tx, mut rx) = channel(1);
        let mut receiver = target_torrent.subscribe();
        tokio::spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let TorrentEvent::FilesChanged = *event {
                        tx.send(()).await.unwrap();
                    }
                } else {
                    break;
                }
            }
        });
        select! {
            _ = time::sleep(Duration::from_secs(2)) => panic!("expected to have received the FilesChanged event"),
            _ = rx.recv() => (),
        }

        // set the state of the downloading torrent to checking files
        target_torrent_context
            .update_state(TorrentState::CheckingFiles)
            .await;
        // create the peer connections
        let (incoming_peer, outgoing_peer) = create_peer_pair!(
            &source_torrent,
            &target_torrent,
            DEFAULT_TORRENT_PROTOCOL_EXTENSIONS()
        );

        // notify that pieces have become available
        incoming_peer.notify_piece_availability(vec![0, 1, 2, 3]);
        // wait for the availability to have been processed
        loop {
            if outgoing_peer.inner.remote_pieces.read().await.count_ones() > 0 {
                break;
            }
            time::sleep(Duration::from_millis(50)).await
        }

        // check that the outgoing peer is not trying to download any pieces
        let pending_requests = outgoing_peer
            .inner
            .client_pending_requests
            .read()
            .await
            .len();
        assert_eq!(
            0, pending_requests,
            "expected the downloading peer to not have sent out any requests"
        );

        // set the state of the downloading torrent to downloading
        target_torrent_context
            .update_state(TorrentState::Downloading)
            .await;
        // notify that pieces have become available
        incoming_peer.notify_piece_availability(vec![4]);
        // check that the outgoing peer is trying to download pieces
        let result = async {
            let mut attempts = 0;

            loop {
                if outgoing_peer
                    .inner
                    .client_pending_requests
                    .read()
                    .await
                    .len()
                    > 0
                {
                    return true;
                }
                attempts += 1;
                if attempts > 10 {
                    return false;
                }
                time::sleep(Duration::from_millis(50)).await;
            }
        }
        .await;
        assert!(
            result,
            "expected the downloading peer to have sent requests"
        );
    }

    #[test]
    fn test_data_transfer_stats_rate() {
        let stats = DataTransferStats {
            transferred_bytes: 1024,
            elapsed_micro: 1_000_000,
        };
        let result = stats.rate();
        assert_eq!(1024, result);

        let stats = DataTransferStats {
            transferred_bytes: 1024,
            elapsed_micro: 500_000,
        };
        let result = stats.rate();
        assert_eq!(1024, result);

        let stats = DataTransferStats {
            transferred_bytes: 16384,
            elapsed_micro: 500_000,
        };
        let result = stats.rate();
        assert_eq!(16384, result);

        let stats = DataTransferStats {
            transferred_bytes: 1024,
            elapsed_micro: 1_250_000,
        };
        let result = stats.rate();
        assert_eq!(819, result);
    }

    #[test]
    fn test_interest_state_ordering() {
        let result = InterestState::NotInterested.cmp(&InterestState::NotInterested);
        assert_eq!(Ordering::Equal, result);

        let result = InterestState::Interested.cmp(&InterestState::NotInterested);
        assert_eq!(Ordering::Greater, result);

        let result = InterestState::NotInterested.cmp(&InterestState::Interested);
        assert_eq!(Ordering::Less, result);

        let result = InterestState::Interested.cmp(&InterestState::Interested);
        assert_eq!(Ordering::Equal, result);
    }

    #[cfg(test)]
    mod peer_client_info {
        use super::*;

        #[test]
        fn test_peer_priority() {
            let peer1 = create_info_from_addr(([230, 12, 123, 1], 1234).into());
            let peer2 = create_info_from_addr(([230, 12, 123, 3], 300).into());

            assert_eq!(Some(2579844473), peer1.peer_priority(&peer2));
        }

        fn create_info_from_addr(addr: SocketAddr) -> PeerClientInfo {
            PeerClientInfo {
                handle: Default::default(),
                id: PeerId::new(),
                addr,
                connection_type: ConnectionDirection::Inbound,
                connection_protocol: ConnectionProtocol::Tcp,
            }
        }
    }
}
