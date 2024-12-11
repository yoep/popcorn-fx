use crate::torrent::peer::extension::{
    Extension, ExtensionName, ExtensionNumber, ExtensionRegistry, Extensions,
};
use crate::torrent::peer::peer_reader::{PeerReader, PeerReaderEvent};
use crate::torrent::peer::protocol::{ExtendedHandshake, Handshake, Message, Piece, Request};
use crate::torrent::peer::{Error, PeerId, Result};
use crate::torrent::{
    InfoHash, PieceIndex, TorrentContext, TorrentEvent, TorrentInfo, TorrentMetadata,
};
use bit_vec::BitVec;
use bitmask_enum::bitmask;
use byteorder::BigEndian;
use byteorder::ByteOrder;
use derive_more::Display;
use log::{debug, error, trace, warn};
use popcorn_fx_core::core::callback::{Callback, MultiCallback, Subscriber, Subscription};
use popcorn_fx_core::core::Handle;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{split, AsyncRead, AsyncWriteExt, BufWriter, WriteHalf};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{channel, unbounded_channel, Receiver, UnboundedReceiver, UnboundedSender};
use tokio::sync::{Mutex, OwnedSemaphorePermit, RwLock};
use tokio::time::timeout;
use tokio::{select, time};
use tokio_util::sync::CancellationToken;

const KEEP_ALIVE_SECONDS: u64 = 90;
const HANDSHAKE_MESSAGE_LEN: usize = 68;

/// The peer's unique identifier handle.
pub type PeerHandle = Handle;

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

/// The connection direction type of a peer.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionType {
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum PeerEvent {
    /// Indicates that the handshake with the remote has been completed
    HandshakeCompleted,
    /// Indicates that the extended handshake with the remote peer has been completed
    ExtendedHandshakeCompleted,
    /// Indicates that the state of this peer has changed
    StateChanged(PeerState),
    /// Indicates that remote pieces have become available to be downloaded
    RemoteAvailablePieces(Vec<PieceIndex>),
    /// Indicates that one or more peers has been discovered by the swarm
    PeersDiscovered(Vec<SocketAddr>),
    /// Indicates that one or more peers are dropped from the swarm
    PeersDropped(Vec<SocketAddr>),
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
pub struct PeerDataTransferStats {
    /// The bytes that have been transferred to the peer.
    pub upload: usize,
    /// The bytes per second that have been transferred to the peer.
    pub upload_rate: u64,
    /// The bytes that contain actual piece data transferred to the peer.
    pub upload_useful: usize,
    /// The bytes per seconds that contain actual piece data transferred to the peer.
    pub upload_useful_rate: u64,
    /// The bytes that have been transferred from the peer.
    pub download: usize,
    /// The bytes per second that the downloaded from the peer.
    pub download_rate: u64,
    /// The bytes that contain actual piece data transferred from the peer.
    pub download_useful: usize,
    /// The bytes per seconds that contain actual piece data transferred from the peer.
    pub download_useful_rate: u64,
}

#[derive(Debug)]
pub struct Peer {
    handle: PeerHandle,
    inner: Arc<PeerContext>,
}

impl Peer {
    pub async fn new_outbound(
        peer_id: PeerId,
        addr: SocketAddr,
        torrent: Arc<TorrentContext>,
        protocol_extensions: ProtocolExtensionFlags,
        extensions: Extensions,
        timeout: Duration,
        runtime: Arc<Runtime>,
    ) -> Result<Self> {
        trace!("Trying outgoing peer connection to {}", addr);
        select! {
            _ = time::sleep(timeout) => {
                Err(Error::Io(format!("failed to connect to {}, connection timed out", addr)))
            },
            stream = TcpStream::connect(&addr) => Self::process_connection_stream(peer_id, addr, stream?, ConnectionType::Outbound, torrent, protocol_extensions, extensions, timeout, runtime).await
        }
    }

    pub async fn new_inbound(
        peer_id: PeerId,
        addr: SocketAddr,
        stream: TcpStream,
        torrent: Arc<TorrentContext>,
        protocol_extensions: ProtocolExtensionFlags,
        extensions: Extensions,
        timeout: Duration,
        runtime: Arc<Runtime>,
    ) -> Result<Self> {
        trace!(
            "Trying to receive incoming peer connection from {}",
            stream.peer_addr()?
        );
        select! {
            _ = time::sleep(timeout) => {
                Err(Error::Io(format!("failed to accept connection from {}, connection timed out", addr)))
            },
            result = Self::process_connection_stream(peer_id, addr, stream, ConnectionType::Inbound, torrent, protocol_extensions, extensions, timeout, runtime) => result
        }
    }

    /// Get the unique identifier handle of the peer.
    ///
    /// # Returns
    ///
    /// Returns the unique identifier handle of the peer.
    pub fn handle(&self) -> PeerHandle {
        self.handle
    }

    /// Get the unique identifier handle of the peer as reference.
    ///
    /// # Returns
    ///
    /// Returns the unique identifier reference handle of the peer.
    pub fn handle_as_ref(&self) -> &PeerHandle {
        &self.handle
    }

    /// Get the address of the remote peer.
    pub fn addr(&self) -> SocketAddr {
        self.inner.addr()
    }

    /// Get the address reference of the remote peer.
    pub fn addr_as_ref(&self) -> &SocketAddr {
        self.inner.addr_as_ref()
    }

    /// Get the connection type of the peer.
    ///
    /// # Returns
    ///
    /// Returns the connection type of the peer.
    pub fn connection_type(&self) -> ConnectionType {
        self.inner.connection_type
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

    /// Get the available pieces of the remote peer as a bit vector.
    /// It might return an empty bit vector when the handshake has not been completed yet.
    pub async fn remote_piece_bitfield(&self) -> BitVec {
        self.inner.remote_pieces.read().await.clone()
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

    /// Notify the peer we have new piece(s) available for download.
    /// This is offloaded to the main loop of the peer, so this will not block unless the channel is full.
    ///
    /// # Arguments
    ///
    /// * `piece` - The piece index that we have available for download
    pub fn notify_has_pieces(&self, pieces: Vec<PieceIndex>) {
        self.inner
            .send_command_event(PeerCommandEvent::ClientHasPieces(pieces))
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

    /// Resume the exchanging of data with the peer.
    pub fn resume(&self) {
        self.inner
            .send_command_event(PeerCommandEvent::UpdateClientChokeState(
                ChokeState::UnChoked,
            ));
    }

    /// Pause the peer client.
    /// This will put the data exchange with the peer on hold.
    pub fn pause(&self) {
        self.inner
            .send_command_event(PeerCommandEvent::UpdateClientChokeState(ChokeState::Choked));
        self.inner
            .send_command_event(PeerCommandEvent::UpdateState(PeerState::Paused));
    }

    /// Close this peer connection.
    /// The connection with the remote peer will be closed and this peer can no longer be used.
    pub async fn close(&self) {
        self.inner.close(CloseReason::Client).await
    }

    /// Retrieve the connection stats from this peer and reset the stats.
    ///
    /// # Returns
    ///
    /// Returns the peer connection stats.
    pub(crate) async fn stats_and_reset(&self) -> PeerDataTransferStats {
        let mut stats = PeerDataTransferStats::default();

        {
            let mut mutex = self.inner.outgoing_data_stats.write().await;
            stats.upload = mutex.transferred_bytes;
            stats.upload_rate = mutex.transferred_bytes_rate;
            stats.upload_useful = mutex.transferred_bytes_useful;
            stats.upload_useful_rate = mutex.transferred_bytes_useful_rate;
            mutex.reset();
        }
        {
            let mut mutex = self.inner.incoming_data_stats.write().await;
            stats.download = mutex.transferred_bytes;
            stats.download_rate = mutex.transferred_bytes_rate;
            stats.download_useful = mutex.transferred_bytes_useful;
            stats.download_useful_rate = mutex.transferred_bytes_useful_rate;
            mutex.reset();
        }

        stats
    }

    async fn send_initial_messages(&self) -> Result<()> {
        let bitfield = self.inner.torrent.piece_bitfield().await;
        let mut is_fast_have_sent = false;

        // the extended handshake should be sent immediately after the standard bittorrent handshake to any peer that supports the extension protocol
        if self
            .inner
            .is_protocol_enabled(ProtocolExtensionFlags::LTEP)
            .await
        {
            trace!("Exchanging extended handshake with peer {}", self);
            self.inner
                .send_command_event(PeerCommandEvent::UpdateState(PeerState::Handshake));
            if let Err(e) = self.inner.send_extended_handshake().await {
                warn!("Failed to send extended handshake to peer {}, {}", self, e);
                // remove the LTEP extension flag from the remote peer
                // as the extended handshake has failed to complete
                if let Some(mutex) = self.inner.remote.write().await.as_mut() {
                    mutex.protocol_extensions &= !ProtocolExtensionFlags::LTEP;
                }
            }
        }

        // check if the fast protocol is enabled
        // if so, we send the initial fast messages to the remote peer
        if self
            .inner
            .is_protocol_enabled(ProtocolExtensionFlags::Fast)
            .await
        {
            let is_metadata_known = self
                .inner
                .torrent
                .metadata_lock()
                .read()
                .await
                .info
                .is_some();

            if is_metadata_known && bitfield.all() {
                is_fast_have_sent = true;
                if let Err(e) = self.inner.send(Message::HaveAll).await {
                    warn!("Failed to send have all to peer {}, {}", self, e);
                    self.inner
                        .send_command_event(PeerCommandEvent::UpdateState(PeerState::Error));
                }
            } else if !is_metadata_known || bitfield.none() {
                is_fast_have_sent = true;
                if let Err(e) = self.inner.send(Message::HaveNone).await {
                    warn!("Failed to send have none to peer {}, {}", self, e);
                    self.inner
                        .send_command_event(PeerCommandEvent::UpdateState(PeerState::Error));
                }
            }
        }

        // we try to send the bitfield with completed pieces if none of the initial fast messages have been sent
        // this is only done if at least one piece is completed
        if !is_fast_have_sent && bitfield.any() {
            if let Err(e) = self.inner.send(Message::Bitfield(bitfield.clone())).await {
                warn!("Failed to send bitfield to peer {}, {}", self, e);
                self.inner
                    .send_command_event(PeerCommandEvent::UpdateState(PeerState::Error));
            }
        }

        // store the bitfield of the torrent as initial state
        *self.inner.client_pieces.write().await = bitfield;

        self.inner
            .send_command_event(PeerCommandEvent::UpdateState(PeerState::Idle));
        Ok(())
    }

    /// Create a new clone of this instance, which is only allowed by the internal processes
    /// of this library.
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
            inner: self.inner.clone(),
        }
    }

    async fn process_connection_stream(
        peer_id: PeerId,
        addr: SocketAddr,
        stream: TcpStream,
        connection_type: ConnectionType,
        torrent: Arc<TorrentContext>,
        protocol_extensions: ProtocolExtensionFlags,
        extensions: Extensions,
        timeout: Duration,
        runtime: Arc<Runtime>,
    ) -> Result<Self> {
        let (reader, writer) = split(stream);
        let (reader_sender, peer_reader_receiver) = channel(20);
        let (event_sender, event_receiver) = unbounded_channel();
        let extension_registry = Self::create_extension_registry(&extensions);
        let peer_handle = PeerHandle::new();
        let total_pieces = torrent.total_pieces().await;
        let inner = Arc::new(PeerContext {
            handle: peer_handle,
            client_id: peer_id,
            // the remote information is unknown until the handshake has been completed
            remote: RwLock::new(None),
            torrent,
            addr,
            state: RwLock::new(PeerState::Handshake),
            connection_type,
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
            remote_fast_pieces: RwLock::new(Vec::with_capacity(0)),
            // create new peer request buffers which are not running as the peer connection starts in the state choked
            client_pending_requests: RwLock::new(HashMap::with_capacity(0)),
            client_pending_request_permits: Mutex::new(HashMap::with_capacity(0)),
            remote_pending_requests: RwLock::new(Vec::with_capacity(0)),
            remote_pending_request_permit: Mutex::new(None),
            writer: Mutex::new(BufWriter::new(writer)),
            incoming_data_stats: RwLock::new(PeerTransferStats::default()),
            outgoing_data_stats: RwLock::new(PeerTransferStats::default()),
            event_sender,
            callbacks: MultiCallback::new(runtime.clone()),
            cancellation_token: CancellationToken::new(),
            timeout,
            runtime,
        });
        let peer = Self {
            handle: peer_handle,
            inner,
        };
        let mut peer_reader = PeerReader::new(
            peer.handle,
            reader,
            reader_sender,
            peer.inner.cancellation_token.clone(),
        );

        if connection_type == ConnectionType::Outbound {
            // as this is an outgoing connection, we're the once who initiate the handshake
            peer.inner.send_handshake().await?;
        }

        // retrieve the incoming handshake from the reader
        // as the handshake is always 68 bytes long, we request a buffer of 68 bytes from the reader
        trace!("Waiting for peer handshake from {}", peer.inner.addr);
        let bytes =
            Self::try_receive_handshake(&peer.inner.addr, &mut peer_reader, timeout).await?;
        peer.inner.validate_handshake(bytes).await?;

        if connection_type == ConnectionType::Inbound {
            // as this is an incoming connection, we need to send our own handshake after receiving the peer handshake
            peer.inner.send_handshake().await?;
        }

        // start the peer read loop in a new thread
        // this moves the ownership of PeerReader to a new thread
        peer.inner.runtime.spawn(async move {
            peer_reader.start_read_loop().await;
        });

        // start the main loop of the inner peer
        let main_loop = peer.inner.clone();
        let torrent_receiver = peer.inner.torrent.subscribe();
        peer.inner.runtime.spawn(async move {
            main_loop
                .start(peer_reader_receiver, event_receiver, torrent_receiver)
                .await
        });

        peer.send_initial_messages().await?;
        Ok(peer)
    }

    /// Try to receive/read the incoming handshake from the remote peer.
    async fn try_receive_handshake<R: AsyncRead + Unpin>(
        addr: &SocketAddr,
        reader: &mut PeerReader<R>,
        timeout: Duration,
    ) -> Result<Vec<u8>> {
        select! {
            _ = time::sleep(timeout) => Err(Error::Handshake(
                addr.clone(),
                format!("handshake has timed out after {}.{:03} seconds", timeout.as_secs(), timeout.subsec_millis())
            )),
            bytes = reader.read(HANDSHAKE_MESSAGE_LEN) => bytes,
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

impl Callback<PeerEvent> for Peer {
    fn subscribe(&self) -> Subscription<PeerEvent> {
        self.inner.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<PeerEvent>) {
        self.inner.subscribe_with(subscriber)
    }
}

impl Display for Peer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl PartialEq for Peer {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

/// Information about transferred data over the peer connection.
#[derive(Debug, Clone)]
pub(crate) struct DataTransferStats {
    /// The total amount of bytes that have been transferred
    pub transferred_bytes: usize,
    /// The time it took in micro seconds to transfer the bytes
    pub elapsed: u128,
}

impl DataTransferStats {
    /// Get the rate of bytes transferred per second.
    pub fn rate(&self) -> u64 {
        PeerContext::calculate_rate(self.transferred_bytes, self.elapsed)
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

#[derive(Debug, Default, Clone)]
struct PeerTransferStats {
    /// The amount of bytes that have been transferred.
    pub transferred_bytes: usize,
    /// The actual useful bytes that have been transferred.
    /// This only counts the actual piece payload data that has been transferred and excludes everything of the Bittorrent message protocol.
    pub transferred_bytes_useful: usize,
    /// The total amount of bytes that have been transferred per second.
    pub transferred_bytes_rate: u64,
    /// The actual useful bytes transferred per second.
    /// This only counts the actual piece payload data that has been transferred and excludes everything of the Bittorrent message protocol.
    pub transferred_bytes_useful_rate: u64,
    /// The total amount of bytes that have been transferred during the lifetime of the connection.
    pub total_transferred_bytes: u64,
    /// The actual useful total bytes that have been transferred during the lifetime of the connection.
    /// This only counts the actual piece payload data that has been transferred and excludes everything of the Bittorrent message protocol.
    pub total_transferred_bytes_useful: u64,
}

impl PeerTransferStats {
    /// Reset all the data transfer stats, except for the lifetime stats.
    fn reset(&mut self) {
        self.transferred_bytes = 0;
        self.transferred_bytes_useful = 0;
        self.transferred_bytes_rate = 0;
        self.transferred_bytes_useful_rate = 0;
    }
}

/// The piece that should be requested from the remote peer.
#[derive(Debug, Clone, PartialEq)]
pub struct RequestPieceData {
    /// The piece index to request
    pub piece: PieceIndex,
    /// The indication if the piece is allowed by the fast protocol.
    /// If it's allowed, then it will be requested even if the remote peer is choked.
    pub is_fast_allowed: bool,
}

/// The internal peer command events which are executed on the main loop of the peer.
/// These can be used to offload async operations to the main loop.
#[derive(Debug, PartialEq)]
pub enum PeerCommandEvent {
    /// Indicates that the torrent has completed one or more pieces and the remote peer needs to be notified
    ClientHasPieces(Vec<PieceIndex>),
    /// Indicates that the choke state of the peer needs to be changed
    UpdateClientChokeState(ChokeState),
    /// Indicates that the state if the peer needs to be changed
    UpdateState(PeerState),
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
}

#[derive(Debug)]
pub struct PeerContext {
    /// The peer's unique identifier handle
    handle: PeerHandle,
    /// Our unique client peer id
    client_id: PeerId,
    /// The remote peer information, known after the initial handshake.
    remote: RwLock<Option<RemotePeer>>,
    /// The immutable torrent this peer connection belongs to.
    /// This is a weak reference to the [Torrent] and might be invalid if the peer is kept alive for invalid reasons.
    torrent: Arc<TorrentContext>,
    /// The immutable address of the remote peer
    addr: SocketAddr,
    /// Identifies the connection direction (_incoming or outgoing_) of this peer
    connection_type: ConnectionType,
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
    remote_fast_pieces: RwLock<Vec<PieceIndex>>,

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

    /// The TCP write connection to the remote peer
    writer: Mutex<BufWriter<WriteHalf<TcpStream>>>,

    /// The data transfer info of the incoming channel (from the remote peer)
    incoming_data_stats: RwLock<PeerTransferStats>,
    /// The data transfer info of the outgoing channel (to the remote peer)
    outgoing_data_stats: RwLock<PeerTransferStats>,

    /// The sender for internal events
    event_sender: UnboundedSender<PeerCommandEvent>,
    /// The callbacks which are triggered by this peer when an event is raised
    callbacks: MultiCallback<PeerEvent>,
    /// The timeout of the connection
    timeout: Duration,
    /// The cancellation token to cancel any async task within this peer on closure
    cancellation_token: CancellationToken,
    /// The shared runtime instance
    runtime: Arc<Runtime>,
}

impl PeerContext {
    /// Start the main loop of this peer.
    /// It handles the peer reader events and processing of the pending requests.
    async fn start(
        &self,
        mut peer_reader: Receiver<PeerReaderEvent>,
        mut event_receiver: UnboundedReceiver<PeerCommandEvent>,
        mut torrent_receiver: Subscription<TorrentEvent>,
    ) {
        let mut interval = time::interval(Duration::from_secs(2));

        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                _ = time::sleep(Duration::from_secs(KEEP_ALIVE_SECONDS)) => self.send_keep_alive().await,
                Some(event) = peer_reader.recv() => self.handle_reader_event(event).await,
                Some(event) = event_receiver.recv() => self.handle_command_event(event).await,
                Some(event) = torrent_receiver.recv() => self.handle_torrent_event(&*event).await,
                _ = interval.tick() => self.check_wanted_pieces().await,
            }
        }

        self.update_state(PeerState::Closed).await;
        trace!("Peer {} main loop ended", self);
    }

    /// Get the address of the remote peer.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Get the address reference of the remote peer.
    pub fn addr_as_ref(&self) -> &SocketAddr {
        &self.addr
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

    /// Check if the remote peer is a seed.
    /// This means that the remote peer has all pieces available and is seeding the torrent.
    pub async fn is_seed(&self) -> bool {
        self.remote_pieces.read().await.all()
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

    /// Get the known metadata from the torrent.
    /// This info is requested from the torrent that created this peer.
    pub async fn metadata(&self) -> TorrentInfo {
        self.torrent.metadata_lock().read().await.clone()
    }

    /// Update the underlying torrent metadata.
    /// This method can be used by extensions to update the torrent metadata when the current
    /// connection is based on a magnet link.
    ///
    /// # Arguments
    ///
    /// * `metadata` - The new torrent metadata
    pub async fn update_metadata(&self, metadata: TorrentMetadata) {
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
        let wanted_pieces = self.torrent.wanted_pieces().await;

        wanted_pieces.into_iter().any(|e| !mutex.contains_key(&e))
    }

    /// Handle events that are sent from the peer reader.
    async fn handle_reader_event(&self, event: PeerReaderEvent) {
        match event {
            PeerReaderEvent::Closed => self.close(CloseReason::Remote).await,
            PeerReaderEvent::Message(message, data_transfer) => {
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
        }
    }

    async fn handle_received_message(&self, message: Message) {
        debug!("Processing remote peer {} message {:?}", self, message);
        match message {
            Message::KeepAlive => {
                trace!("Received keep alive for peer {}", self);
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
            Message::Have(piece) => self.update_remote_piece(piece as PieceIndex).await,
            Message::HaveAll => self.update_remote_fast_have(true).await,
            Message::HaveNone => self.update_remote_fast_have(false).await,
            Message::Bitfield(pieces) => self.update_remote_pieces(pieces).await,
            Message::Request(request) => self.add_remote_pending_request(request).await,
            Message::RejectRequest(request) => self.handle_rejected_client_request(request).await,
            Message::Cancel(request) => self.cancel_remote_pending_request(request).await,
            Message::AllowedFast(piece) => self.add_remote_fast_piece(piece as PieceIndex).await,
            Message::Piece(piece) => self.handle_received_piece(piece).await,
            Message::ExtendedHandshake(handshake) => {
                self.update_extended_handshake(handshake).await
            }
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

        if *self.state.read().await != PeerState::Uploading {
            self.send_command_event(PeerCommandEvent::UpdateState(PeerState::Uploading));
        }

        if self.torrent.has_piece(request.index).await {
            let request_end = request.begin + request.length;
            match self
                .torrent
                .read_piece_bytes(request.index, request.begin..request_end)
                .await
            {
                Ok(data) => {
                    match self
                        .send(Message::Piece(Piece {
                            index: request.index,
                            begin: request.begin,
                            data,
                        }))
                        .await
                    {
                        Ok(_) => {
                            debug!("Sent {:?} data to remote peer {}", request, self)
                        }
                        Err(e) => warn!("Failed to send piece data to peer {}, {}", self, e),
                    }
                }
                Err(e) => {
                    warn!("Failed to read the piece data for {}, {}", self, e);
                    self.send_reject_request(request).await;
                }
            }
        } else {
            debug!(
                "Unable to provide {:?} data to peer {}, piece data is not available",
                request, self
            );
            self.send_reject_request(request).await;
        }
    }

    /// Handle an event that has been triggered by the [Torrent].
    async fn handle_torrent_event(&self, event: &TorrentEvent) {
        match event {
            TorrentEvent::PiecesChanged => {
                trace!("Updating peer {} client piece bitfield", self);
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
                self.request_upload_permit_if_needed().await;
            }
            _ => {}
        }
    }

    /// Process a request which has been rejected by the remote peer.
    /// This can be the case when we've request piece data that is no longer available, or the remote peer cannot serve it at the moment.
    async fn handle_rejected_client_request(&self, request: Request) {
        self.remove_client_pending_request(&request).await;
        self.torrent
            .pending_request_rejected(request.index, request.begin, self.handle)
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
                    "Received invalid piece part {:?} data from {}, received data len {} and expected {}",
                        part,
                        self,
                        piece.data.len(),
                        data_size
                    );

                    self.release_client_pending_request_permit(piece_index)
                        .await;
                    self.torrent.invalid_piece_data_received(part, self.handle);
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

    /// Handle an internal peer command event.
    async fn handle_command_event(&self, event: PeerCommandEvent) {
        trace!("Handling peer {} command event {:?}", self, event);
        match event {
            PeerCommandEvent::ClientHasPieces(pieces) => {
                self.update_client_piece_availability(pieces).await
            }
            PeerCommandEvent::UpdateClientChokeState(state) => {
                self.update_client_choke_state(state).await
            }
            PeerCommandEvent::UpdateState(state) => self.update_state(state).await,
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
        }
    }

    /// Check if the remote peer has at least one wanted piece available.
    /// If so, trigger the necessary commands to retrieve this piece.
    async fn check_wanted_pieces(&self) {
        if !self.torrent.is_download_allowed().await {
            return;
        }

        let pending_requests = self.client_pending_requests.read().await.len();
        if pending_requests == 0 && self.has_wanted_piece().await {
            self.send_command_event(PeerCommandEvent::DetermineClientInterestState);
            self.send_command_event(PeerCommandEvent::RequestWantedPieces);
        }
    }

    /// Informs the enabled extensions of the peer event.
    async fn inform_extensions_of_event(&self, event: PeerEvent) {
        let extensions = self.remote_extension_registry().await;

        if let Some(extensions) = extensions {
            for extension in self
                .extensions
                .iter()
                .filter(|e| extensions.contains_key(&e.name().to_string()))
            {
                extension.on(&event, &self).await;
            }
        }
    }

    /// Determine if our peer client is interested in pieces from the remote peer.
    async fn determine_client_interest_state(&self) {
        let state: InterestState;
        let wanted_pieces = self.torrent.wanted_pieces().await;
        if self
            .remote_pieces
            .read()
            .await
            .iter()
            .enumerate()
            .filter(|(_, e)| *e)
            .any(|(piece, _)| wanted_pieces.contains(&piece))
        {
            state = InterestState::Interested;
        } else {
            state = InterestState::NotInterested;
        }

        self.update_client_interest_state(state).await;
    }

    /// Try to retrieve an upload permit from the torrent.
    /// If the peer already has an upload permit, this call will be a no-op.
    async fn request_upload_permit(&self) {
        if !self.torrent.is_upload_allowed().await
            || self.remote_pending_request_permit.lock().await.is_some()
        {
            return;
        }

        if let Some(permit) = self.torrent.request_upload_permit().await {
            // store the permit
            *self.remote_pending_request_permit.lock().await = Some(permit);
            debug!("Got upload permit from {} for {}", self.torrent, self);
            // unchoke the client peer
            self.send_command_event(PeerCommandEvent::UpdateClientChokeState(
                ChokeState::UnChoked,
            ));
        } else {
            trace!(
                "No upload permit available from {} for {}",
                self.torrent,
                self
            )
        }
    }

    /// Request an upload permit from the torrent if needed.
    ///
    /// If the remote peer is [InterestState::Interested] and the torrent allows uploads,
    /// then we queue the command to try to obtain an upload permit.
    async fn request_upload_permit_if_needed(&self) {
        let state = self.remote_interest_state.read().await.clone();
        let has_permit = self.remote_pending_request_permit.lock().await.is_some();
        if state == InterestState::Interested
            && !has_permit
            && self.torrent.is_upload_allowed().await
        {
            self.send_command_event(PeerCommandEvent::RequestUploadPermit);
        }
    }

    async fn validate_handshake(&self, buffer: Vec<u8>) -> Result<()> {
        let handshake = Handshake::from_bytes(&self.addr, buffer.as_ref())?;
        let info_hash = self.torrent.metadata_lock().read().await.info_hash.clone();
        debug!("Received {:?} for {}", handshake, self);

        // verify that the peer sent the correct info hash which we expect
        if info_hash != handshake.info_hash {
            self.update_state(PeerState::Error).await;
            return Err(Error::Handshake(
                self.addr.clone(),
                "received incorrect info hash from peer".to_string(),
            ));
        }

        // store the remote peer information
        trace!("Updating remote peer information for {}", handshake.peer_id);
        {
            let mut mutex = self.remote.write().await;
            *mutex = Some(RemotePeer {
                peer_id: handshake.peer_id,
                protocol_extensions: handshake.supported_extensions,
                extensions: ExtensionRegistry::default(),
                client_name: None,
            });
        }

        debug!("Handshake of peer {} has been validated", self);
        self.invoke_event(PeerEvent::HandshakeCompleted);
        Ok(())
    }

    /// Updates the choke state of the client peer.
    pub async fn update_client_choke_state(&self, state: ChokeState) {
        // check if we're already in the expected state
        if *self.client_choke_state.read().await == state {
            trace!("Peer {} is already in the client state {}", self, state);
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
            debug!("Failed to send choke state to peer {}, {}", self, e);
            // remove the upload permit
            let _ = self.remote_pending_request_permit.lock().await.take();
        }

        debug!("Client peer {} entered {} state", self, state);
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
        } else {
            if self.torrent.is_download_allowed().await {
                self.send_command_event(PeerCommandEvent::RequestWantedPieces);
            }
        }

        trace!("Remote peer {} entered {} state", self, state);
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
            trace!("Client peer {} entered {} state", self, state);
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
        trace!("Remote peer {} entered {} state", self, state);

        self.request_upload_permit_if_needed().await;
    }

    /// Updates the state of the peer.
    pub async fn update_state(&self, state: PeerState) {
        let mut mutex = self.state.write().await;
        if *mutex == state {
            return;
        }

        *mutex = state;
        debug!("Updated peer {} state to {:?}", self, state);

        self.invoke_event(PeerEvent::StateChanged(state));
    }

    /// Set the client peer as having the given piece.
    pub async fn update_client_piece_availability(&self, pieces: Vec<PieceIndex>) {
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
            if let Err(e) = self.send(Message::Have(*piece as u32)).await {
                warn!(
                    "Failed to notify remote peer about piece {} availability, {}",
                    piece, e
                );
            }
        }

        self.request_upload_permit_if_needed().await;
    }

    /// Set the remote peer as having the given piece.
    async fn update_remote_piece(&self, piece: PieceIndex) {
        {
            let mut mutex = self.remote_pieces.write().await;
            // ensure the BitVec is large enough to accommodate the piece index
            if piece >= mutex.len() {
                if self.torrent.is_metadata_known().await {
                    warn!(
                        "Received piece index {} out of range ({}) for {}",
                        piece,
                        mutex.len(),
                        self
                    );
                    return;
                }

                // increase the size of the BitVec if metadata is still being retrieved
                let additional_len = piece as usize + 1 - mutex.len();
                mutex.extend(vec![false; additional_len]);
            }

            mutex.set(piece, true);
        }

        self.send_command_event(PeerCommandEvent::DetermineClientInterestState);
        self.invoke_event(PeerEvent::RemoteAvailablePieces(vec![piece]));
    }

    /// Update the remote piece availability based on the supplied [BitVec].
    async fn update_remote_pieces(&self, pieces: BitVec) {
        {
            let mut mutex = self.remote_pieces.write().await;
            *mutex = pieces.clone();
            if pieces.all() {
                debug!("Updated peer {} with all pieces available", self);
            } else {
                debug!(
                    "Updated peer {} with a total of {} available pieces",
                    self,
                    pieces.count_ones()
                );
            }
        }

        // notify subscribers about each available piece
        let piece_indexes: Vec<_> = pieces
            .into_iter()
            .enumerate()
            .filter(|(_, v)| *v)
            .map(|(piece, _)| piece as PieceIndex)
            .collect();

        if !piece_indexes.is_empty() {
            self.send_command_event(PeerCommandEvent::DetermineClientInterestState);
            self.invoke_event(PeerEvent::RemoteAvailablePieces(piece_indexes));
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

    async fn add_remote_pending_request(&self, request: Request) {
        let mut mutex = self.remote_pending_requests.write().await;
        if mutex.contains(&request) {
            debug!("Received duplicate {:?} for {}", request, self);
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

    /// Add the received "fast" piece to the list of allowed fast pieces.
    async fn add_remote_fast_piece(&self, piece: PieceIndex) {
        // When the fast extension is disabled, if a peer receives an Allowed Fast message then the peer MUST close the connection.
        if !self.is_protocol_enabled(ProtocolExtensionFlags::Fast).await {
            self.close(CloseReason::FastProtocol).await;
            return;
        }

        {
            let mut mutex = self.remote_fast_pieces.write().await;
            if !mutex.contains(&piece) {
                mutex.push(piece);
            } else {
                trace!(
                    "Received duplicate allowed fast piece {} for {}",
                    piece,
                    self
                );
            }
        }

        self.send_command_event(PeerCommandEvent::RequestFastPieces);
    }

    async fn update_extended_handshake(&self, handshake: ExtendedHandshake) {
        {
            let mut mutex = self.remote.write().await;
            if let Some(remote) = mutex.as_mut() {
                remote.extensions = handshake.m;
                remote.client_name = handshake.client;
                let remote_info = format!("{:?}", remote);
                // drop the mutex as the Display impl requires it to print the info of the remote peer
                drop(mutex);
                debug!(
                    "Updated peer {} with extended handshake information, {}",
                    self, remote_info
                );
            } else {
                warn!("Received extended handshake before the initial handshake was completed");
            }
        }

        self.invoke_event(PeerEvent::ExtendedHandshakeCompleted);
    }

    async fn update_read_data_transfer_stats(
        &self,
        message: &Message,
        data_transfer: DataTransferStats,
    ) {
        let mut mutex = self.incoming_data_stats.write().await;
        mutex.transferred_bytes += data_transfer.transferred_bytes;
        mutex.transferred_bytes_rate += data_transfer.rate();
        mutex.total_transferred_bytes += data_transfer.transferred_bytes as u64;

        if let Message::Piece(piece) = message {
            let data_size = piece.data.len();
            mutex.transferred_bytes_useful += data_size;
            mutex.total_transferred_bytes_useful += data_size as u64;
            mutex.transferred_bytes_useful_rate =
                Self::calculate_rate(data_size, data_transfer.elapsed);
        }
    }

    async fn update_write_data_transfer_stats(
        &self,
        data_transfer: DataTransferStats,
        piece_data_size: Option<usize>,
    ) {
        let mut mutex = self.outgoing_data_stats.write().await;
        mutex.transferred_bytes += data_transfer.transferred_bytes;
        mutex.transferred_bytes_rate += data_transfer.rate();
        mutex.total_transferred_bytes += data_transfer.transferred_bytes as u64;

        if let Some(piece_data_size) = piece_data_size {
            mutex.transferred_bytes_useful += piece_data_size;
            mutex.total_transferred_bytes_useful += piece_data_size as u64;
            mutex.transferred_bytes_useful_rate =
                Self::calculate_rate(piece_data_size, data_transfer.elapsed);
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
        if !self.torrent.is_piece_wanted(request_data.piece).await {
            trace!(
                "Piece {} is no longer wanted for {}",
                request_data.piece,
                self
            );
            return;
        }

        let piece = self
            .torrent
            .pieces_lock()
            .read()
            .await
            .get(request_data.piece)
            .cloned();
        if let Some(piece) = piece {
            // try to obtain a permit before requesting the piece data
            if let Some(permit) = self.torrent.request_download_permit().await {
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
                    if self
                        .send_pending_request(request, request_data.is_fast_allowed)
                        .await
                    {
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
                        "Requested a total of {} requests for piece {} data from {}",
                        sent_requests, piece.index, self
                    );
                } // otherwise, we'll drop the permit automatically as no data could be requested
            }
        }
    }

    /// Try to request pieces that are wanted by the torrent and are available.
    /// Pieces that have already been requested will not be retried.
    async fn request_wanted_pieces(&self) {
        if !self.torrent.is_download_allowed().await {
            return;
        }

        let mutex = self.client_pending_requests.read().await;
        let wanted_pieces: Vec<PieceIndex> = self
            .torrent
            .wanted_pieces()
            .await
            .into_iter()
            .filter(|e| !mutex.contains_key(e))
            .collect();

        for piece in wanted_pieces {
            self.send_command_event(PeerCommandEvent::RequestPieceData(RequestPieceData {
                piece,
                is_fast_allowed: false,
            }))
        }
    }

    /// Try to request any fast pieces which have not yet been requested
    async fn request_fast_pieces(&self) {
        if !self.torrent.is_download_allowed().await {
            return;
        }

        let wanted_pieces = self.torrent.wanted_pieces().await;
        let mutex = self.client_pending_requests.read().await;
        let wanted_fast_pieces: Vec<PieceIndex> = self
            .remote_fast_pieces
            .read()
            .await
            .iter()
            .filter(|e| wanted_pieces.contains(e))
            .filter(|e| !mutex.contains_key(e))
            .cloned()
            .collect();

        for piece in wanted_fast_pieces {
            self.send_command_event(PeerCommandEvent::RequestPieceData(RequestPieceData {
                piece,
                is_fast_allowed: true,
            }))
        }
    }

    async fn send_handshake(&self) -> Result<()> {
        self.update_state(PeerState::Handshake).await;
        let info_hash = self.torrent.metadata_lock().read().await.info_hash.clone();

        let handshake = Handshake::new(info_hash, self.client_id, self.protocol_extensions);
        trace!("Trying to send handshake {:?}", handshake);
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
        let message = Message::ExtendedHandshake(ExtendedHandshake {
            m: extension_registry,
            client: Some("PopcornFX".to_string()),
            regg: None,
            encryption: false,
            metadata_size: None,
            port: Some(self.torrent.peer_port() as u32),
            your_ip: None,
            ipv4: None,
            ipv6: None,
        });

        self.send(message).await
    }

    /// Send the reject request to the remote peer.
    /// This is only executed if the fast protocol is enabled.
    async fn send_reject_request(&self, request: Request) {
        // check if the fast protocol is enabled
        // if so, we should send the reject request
        if self.is_protocol_enabled(ProtocolExtensionFlags::Fast).await {
            if let Err(e) = self.send(Message::RejectRequest(request)).await {
                debug!("Failed to send reject request to peer {}, {}", self, e);
            }
        }
    }

    /// Try to send the given message to the remote peer.
    pub async fn send(&self, message: Message) -> Result<()> {
        trace!("Sending peer {} message {:?}", self, message);
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
        let mut mutex = self.writer.lock().await;
        let msg_length = bytes.as_ref().len();

        let start_time = Instant::now();
        timeout(self.timeout, async {
            trace!("Sending a total of {} bytes to peer {}", msg_length, self);
            mutex.write_all(bytes.as_ref()).await?;
            mutex.flush().await?;
            Ok::<(), Error>(())
        })
        .await??;
        drop(mutex);
        let elapsed = start_time.elapsed().as_micros();

        // update the connection stats
        self.update_write_data_transfer_stats(
            DataTransferStats {
                transferred_bytes: msg_length,
                elapsed,
            },
            piece_data_size,
        )
        .await;

        Ok(())
    }

    /// Request the given piece data from the remote peer.
    /// This will only be executed if the remote peer is not choked and the request has not been made before.
    ///
    /// It returns `true` when the request has been sent to the remote peer, else `false`.
    async fn send_pending_request(&self, request: Request, is_fast_allowed: bool) -> bool {
        // if the remote peer is choked, but the fast protocol allows this request, the request will be allowed
        if !is_fast_allowed && *self.remote_choke_state.read().await == ChokeState::Choked {
            trace!(
                "Trying to request piece {} data from choked peer {}",
                request.index,
                self
            );
            return false;
        }

        if *self.state.read().await != PeerState::Downloading {
            self.send_command_event(PeerCommandEvent::UpdateState(PeerState::Downloading));
        }

        // check if the request is a duplicate
        let mut mutex = self.client_pending_requests.write().await;
        if mutex
            .get(&request.index)
            .map(|requests| requests.contains(&request))
            .unwrap_or(false)
        {
            debug!(
                "Trying to request duplicate piece {} data from peer {}",
                request.index, self
            );
            return false;
        }

        if let Err(e) = self.send(Message::Request(request.clone())).await {
            warn!(
                "Failed request piece {} data from peer {}, {}",
                request.index, self, e
            );
            return false;
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
        let mut mutex = self.client_pending_requests.write().await;

        // remove the permit lock and entry
        mutex.remove(piece);
        self.client_pending_request_permits
            .lock()
            .await
            .remove(piece);
        trace!("Released piece {} data request permit for {}", piece, self);

        if mutex.len() == 0 {
            self.send_command_event(PeerCommandEvent::UpdateState(PeerState::Idle));
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
        debug!("Closing peer {}, {}", self, reason);
        // cancel the main loop of the peer to stop any ongoing operation
        self.cancellation_token.cancel();
        // clear any permits as they cannot be completed from now on
        self.client_pending_requests.write().await.clear();
        self.client_pending_request_permits.lock().await.clear();
        // notify any subscribers
        self.update_state(PeerState::Closed).await;
        // notify the torrent that this peer is being closed
        self.torrent.notify_peer_closed(self.handle);
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

    /// Calculate the data transfer rate in bytes/second.
    fn calculate_rate(data_size: usize, elapsed_micro_secs: u128) -> u64 {
        if elapsed_micro_secs == 0 {
            return 0;
        }

        ((data_size as u128 * 1_000_000) / elapsed_micro_secs) as u64
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

impl Display for PeerContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{}]", self.client_id, self.addr)
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
    use std::str::FromStr;
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio::runtime::Runtime;

    use popcorn_fx_core::core::torrents::magnet::Magnet;
    use popcorn_fx_core::testing::read_test_file_to_bytes;
    use popcorn_fx_core::{available_port, init_logger};

    use super::*;
    use crate::torrent::fs::DefaultTorrentFileStorage;
    use crate::torrent::operation::{TorrentCreateFilesOperation, TorrentCreatePiecesOperation};
    use crate::torrent::peer::extension::metadata::MetadataExtension;
    use crate::torrent::{
        Torrent, TorrentConfig, TorrentFlags, TorrentInfo, DEFAULT_TORRENT_EXTENSIONS,
        DEFAULT_TORRENT_PROTOCOL_EXTENSIONS,
    };

    #[test]
    fn test_peer_new_outbound() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let magnet = Magnet::from_str("magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce").unwrap();
        let torrent_info = TorrentInfo::try_from(magnet).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let port = available_port!(6881, 31000).unwrap();
        let torrent = Torrent::request()
            .metadata(torrent_info.clone())
            .options(TorrentFlags::None)
            .config(
                TorrentConfig::builder()
                    .peers_lower_limit(0)
                    .peers_upper_limit(0)
                    .peer_connection_timeout(Duration::from_secs(2))
                    .tracker_connection_timeout(Duration::from_secs(1))
                    .build(),
            )
            .peer_listener_port(port)
            .extensions(DEFAULT_TORRENT_EXTENSIONS().iter().map(|e| e()).collect())
            .operations(vec![])
            .storage(Box::new(DefaultTorrentFileStorage::new(temp_path)))
            .runtime(runtime.clone())
            .build()
            .unwrap();
        let torrent = Torrent::try_from(torrent).unwrap();
        let inner = torrent.instance().unwrap();

        let peer_addr = SocketAddr::from(([127, 0, 0, 1], port));
        let peer = runtime
            .block_on(Peer::new_outbound(
                PeerId::new(),
                peer_addr,
                inner.clone(),
                DEFAULT_TORRENT_PROTOCOL_EXTENSIONS(),
                vec![],
                Duration::from_secs(5),
                runtime.clone(),
            ))
            .expect("expected the outbound connection to have been created");

        let (tx, rx) = std::sync::mpsc::channel();
        let mut receiver = peer.subscribe();
        runtime.spawn(async move {
            if let PeerEvent::StateChanged(state) = *receiver.recv().await.unwrap() {
                tx.send(state).unwrap()
            }
        });

        let state = rx
            .recv_timeout(Duration::from_secs(5))
            .expect("expected to receive a state update");
        assert_ne!(PeerState::Handshake, state);
    }

    #[test]
    fn test_peer_new_inbound() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent_info_data = read_test_file_to_bytes("debian-udp.torrent");
        let torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let peer_listener_port =
            available_port!(9000, 32000).expect("expected to find an available port");
        let torrent = Torrent::request()
            .metadata(torrent_info.clone())
            .options(TorrentFlags::None)
            .config(
                TorrentConfig::builder()
                    .peer_connection_timeout(Duration::from_secs(1))
                    .tracker_connection_timeout(Duration::from_secs(1))
                    .build(),
            )
            .peer_listener_port(peer_listener_port)
            .extensions(vec![])
            .operations(vec![
                Box::new(TorrentCreatePiecesOperation::new()),
                Box::new(TorrentCreateFilesOperation::new()),
            ])
            .storage(Box::new(DefaultTorrentFileStorage::new(temp_path)))
            .runtime(runtime.clone())
            .build()
            .unwrap();
        let inner = torrent.instance().unwrap();

        let _outbound_peer = runtime
            .block_on(Peer::new_outbound(
                PeerId::new(),
                SocketAddr::from(([127, 0, 0, 1], peer_listener_port)),
                inner,
                ProtocolExtensionFlags::None,
                vec![],
                Duration::from_secs(5),
                runtime.clone(),
            ))
            .unwrap();

        let result = runtime.block_on(torrent.active_peer_connections());
        assert_eq!(
            1, result,
            "expected the incoming peer connection to have been added to the peer pool"
        );
    }

    #[test]
    fn test_peer_close() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent_info_data = read_test_file_to_bytes("debian-udp.torrent");
        let torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let port = available_port!(6881, 31000).unwrap();
        let request = Torrent::request()
            .metadata(torrent_info.clone())
            .options(TorrentFlags::None)
            .config(
                TorrentConfig::builder()
                    .peer_connection_timeout(Duration::from_secs(2))
                    .tracker_connection_timeout(Duration::from_secs(1))
                    .build(),
            )
            .peer_listener_port(port)
            .extensions(vec![Box::new(MetadataExtension::new())])
            .storage(Box::new(DefaultTorrentFileStorage::new(temp_path)))
            .runtime(runtime.clone())
            .build()
            .unwrap();
        let torrent = Torrent::try_from(request).unwrap();
        let torrent_context = torrent.instance().unwrap();
        let context = torrent.instance().unwrap();

        let peer = runtime
            .block_on(Peer::new_outbound(
                PeerId::new(),
                ([127, 0, 0, 1], port).into(),
                torrent_context,
                ProtocolExtensionFlags::LTEP,
                vec![Box::new(MetadataExtension::new())],
                Duration::from_secs(5),
                runtime.clone(),
            ))
            .expect("expected the outbound peer to have been created");
        assert_eq!(
            1,
            runtime.block_on(context.peer_pool().peers.read()).len(),
            "expected the inbound peer to have been added to the peer pool"
        );

        let (tx, rx) = std::sync::mpsc::channel();
        let receiver_runtime = runtime.clone();
        runtime.block_on(async move {
            let mut receiver = context
                .peer_pool()
                .peers
                .read()
                .await
                .get(0)
                .as_ref()
                .unwrap()
                .subscribe();
            receiver_runtime.spawn(async move {
                loop {
                    if let PeerEvent::StateChanged(state) = *receiver.recv().await.unwrap() {
                        if state == PeerState::Closed {
                            tx.send(()).unwrap();
                            break;
                        }
                    }
                }
            });
        });

        runtime.block_on(peer.close());
        drop(peer);

        rx.recv_timeout(Duration::from_millis(500))
            .expect("expected the peer to have been closed");
    }

    #[test]
    fn test_data_transfer_stats_rate() {
        let stats = DataTransferStats {
            transferred_bytes: 1024,
            elapsed: 1000,
        };
        let result = stats.rate();
        assert_eq!(1024, result);

        let stats = DataTransferStats {
            transferred_bytes: 1024,
            elapsed: 500,
        };
        let result = stats.rate();
        assert_eq!(2048, result);

        let stats = DataTransferStats {
            transferred_bytes: 16384,
            elapsed: 50,
        };
        let result = stats.rate();
        assert_eq!(327680, result);

        let stats = DataTransferStats {
            transferred_bytes: 1024,
            elapsed: 1250,
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
}
