use crate::torrents::peers::bt_protocol::{
    ExtendedHandshake, ExtensionFlags, Handshake, Message, Request,
};
use crate::torrents::peers::extensions::{
    Extension, ExtensionName, ExtensionNumber, ExtensionRegistry, Extensions,
};

use crate::torrents::peers::peer_reader::{PeerReader, PeerReaderEvent};
use crate::torrents::peers::peer_request_buffer::PeerRequestBuffer;
use crate::torrents::peers::{Error, PeerId, Result};
use crate::torrents::{InfoHash, PieceIndex, Torrent, TorrentInfo, TorrentMetadata};
use bit_vec::BitVec;
use byteorder::BigEndian;
use byteorder::ByteOrder;
use derive_more::Display;
use log::{debug, error, trace, warn};
use popcorn_fx_core::core::{
    block_in_place, CallbackHandle, Callbacks, CoreCallback, CoreCallbacks, Handle,
};
use std::fmt::{Debug, Display, Formatter};
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{split, AsyncRead, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter, WriteHalf};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{channel, unbounded_channel, Receiver, Sender};
use tokio::sync::{Mutex, RwLock};
use tokio::time::timeout;
use tokio::{select, time};
use tokio_util::sync::CancellationToken;

const DEFAULT_CONNECTION_TIMEOUT_SECONDS: u64 = 10;
const KEEP_ALIVE_SECONDS: u64 = 120;
const HANDSHAKE_MESSAGE_LEN: usize = 68;
const REQUEST_MAX_LENGTH: usize = 16 * 1024; // 16 KiB

/// The peer's unique identifier handle.
pub type PeerHandle = Handle;

/// The peer specific event callbacks.
pub type PeerCallback = CoreCallback<PeerEvent>;

/// The choke states of a peer.
#[repr(u8)]
#[derive(Debug, Display, Clone, Copy, PartialEq)]
pub enum ChokeState {
    #[display(fmt = "choked")]
    Choked = 0,
    #[display(fmt = "un-choked")]
    UnChoked = 1,
}

/// The interest states of a peer.
#[repr(u8)]
#[derive(Debug, Display, Clone, Copy, PartialEq)]
pub enum InterestState {
    #[display(fmt = "not interested")]
    NotInterested = 0,
    #[display(fmt = "interested")]
    Interested = 1,
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
    #[display(fmt = "performing peer handshake")]
    Handshake,
    #[display(fmt = "retrieving metadata")]
    RetrievingMetadata,
    #[display(fmt = "downloading")]
    Downloading,
    #[display(fmt = "uploading")]
    Uploading,
    #[display(fmt = "error")]
    Error,
    #[display(fmt = "closed")]
    Closed,
}

/// The remote peer information
#[derive(Debug, Clone, PartialEq)]
pub struct RemotePeer {
    pub peer_id: PeerId,
    pub supported_extensions: ExtensionFlags,
    pub extensions: ExtensionRegistry,
    pub client_name: Option<String>,
}

#[derive(Debug, Display, Clone, PartialEq)]
pub enum PeerEvent {
    #[display(fmt = "handshake completed")]
    HandshakeCompleted,
    #[display(fmt = "extended handshake completed")]
    ExtendedHandshakeCompleted,
    #[display(fmt = "peer state changed to {}", _0)]
    StateChanged(PeerState),
}

#[derive(Debug, Default, Clone)]
pub struct PeerDataTransferStats {
    /// The bytes that have been transferred from the peer.
    pub upload: usize,
    /// The bytes per second that have been transferred from the peer.
    pub upload_rate: u64,
    /// The bytes that have been transferred to the peer.
    pub download: usize,
    /// The bytes per second that the downloaded from the peer.
    pub download_rate: u64,
}

#[derive(Debug)]
pub struct Peer {
    handle: PeerHandle,
    inner: Arc<InnerPeer>,
    runtime: Arc<Runtime>,
}

impl Peer {
    pub async fn new_outbound(
        addr: SocketAddr,
        torrent: Torrent,
        extensions: Extensions,
        runtime: Arc<Runtime>,
    ) -> Result<Self> {
        trace!("Trying to connect to peer {}", addr);
        select! {
            _ = time::sleep(Duration::from_secs(DEFAULT_CONNECTION_TIMEOUT_SECONDS)) => {
                Err(Error::Io(format!("failed to connect to {}, connection timed out", addr)))
            },
            stream = TcpStream::connect(&addr) => Self::create_outgoing_stream(stream, addr, torrent, extensions, runtime).await
        }
    }

    pub async fn new_inbound(stream: TcpStream) -> Result<Self> {
        trace!(
            "Trying to receive incoming peer connection from {}",
            stream.peer_addr()?
        );
        let (reader, writer) = split(stream);
        let cancellation_token = CancellationToken::new();

        todo!()
    }

    /// Get the unique identifier handle of the peer.
    ///
    /// # Returns
    ///
    /// Returns the unique identifier handle of the peer.
    pub fn handle(&self) -> PeerHandle {
        self.handle
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

    /// Retrieve the remote peer information.
    /// This is only available after the handshake with the peer has been completed.
    ///
    /// # Returns
    ///
    /// Returns the remote peer information when the handshake has been completed, else `None`.
    pub async fn remote_peer(&self) -> Option<RemotePeer> {
        self.inner.remote.read().await.as_ref().map(|e| e.clone())
    }

    /// Retrieve the known supported extensions of the remote peer.
    /// This might still be `None` when the handshake with the peer has not been completed yet.
    ///
    /// # Returns
    ///
    /// Returns the supported extensions of the remote peer.
    pub async fn remote_supported_extensions(&self) -> ExtensionFlags {
        let mutex = self.inner.remote.read().await;
        mutex
            .as_ref()
            .map(|e| e.supported_extensions.clone())
            .unwrap_or(ExtensionFlags::None)
    }

    /// Retrieve the known extension registry of the remote peer.
    /// This might still be `None` when the handshake with the peer has not been completed yet.
    ///
    /// # Returns
    ///
    /// Returns the extension registry of the remote peer.
    pub async fn remote_extension_registry(&self) -> Option<ExtensionRegistry> {
        let mutex = self.inner.remote.read().await;
        mutex.as_ref().map(|e| e.extensions.clone())
    }

    /// Retrieve the torrent info hash.
    /// This info hash is used during the handshake with the peer and is immutable for the
    /// lifetime of the peer connection.
    pub async fn info_hash(&self) -> Result<InfoHash> {
        let metadata = self.inner.torrent.metadata().await?;
        Ok(metadata.info_hash)
    }

    /// Retrieve the torrent metadata.
    /// This info is requested from the torrent that created this peer.
    pub async fn metadata(&self) -> Option<TorrentInfo> {
        self.inner
            .torrent
            .metadata()
            .await
            .map(|e| Some(e))
            .unwrap_or(None)
    }

    /// Retrieve the current state of this peer.
    ///
    /// # Returns
    ///
    /// Returns the current state of this peer.
    pub async fn state(&self) -> PeerState {
        let mutex = self.inner.state.read().await;
        mutex.clone()
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

    /// Send the given message to the remote peer.
    ///
    /// # Arguments
    ///
    /// * `message` - The protocol message to send
    ///
    /// # Returns
    ///
    /// Returns an error when the message failed to send successfully.
    pub async fn send(&self, message: Message) -> Result<()> {
        self.inner.send(message).await
    }

    /// Send the given Bittorrent Protocol message bytes to the remote peer.
    /// The BigEndian length of the given message bytes is automatically prefixed to the outgoing stream.
    ///
    /// Be aware that if you're sending an invalid protocol message to the remote, it might close the connection.
    ///
    /// # Arguments
    ///
    /// * `message` - The bytes to send
    ///
    /// # Returns
    ///
    /// Returns an error when the message failed to send successfully.
    pub async fn send_bytes<T: AsRef<[u8]>>(&self, message: T) -> Result<()> {
        self.inner.send_bytes(message).await
    }

    /// Update the underlying torrent metadata.
    /// This method can be used by extensions to update the torrent metadata when the current
    /// connection is based on a magnet link.
    ///
    /// # Arguments
    ///
    /// * `metadata` - The new torrent metadata
    pub async fn update_torrent_metadata(&self, metadata: TorrentMetadata) {
        self.inner.torrent.add_metadata(metadata).await;
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

    /// Request a piece from the remote peer.
    /// This creates one or more requests for the given piece in a queue buffer.
    ///
    /// # Arguments
    ///
    /// * `piece` - The piece index to request
    pub(crate) async fn request_piece(&self, piece: PieceIndex) {
        if let Some(piece) = self.inner.torrent.piece_info(piece).await {
            let num_of_requests = (piece.length + REQUEST_MAX_LENGTH - 1) / REQUEST_MAX_LENGTH;
            let mut offset = 0;

            for _ in 0..num_of_requests {
                let length = if piece.length < REQUEST_MAX_LENGTH {
                    piece.length
                } else {
                    REQUEST_MAX_LENGTH
                };

                self.inner
                    .client_pending_requests
                    .push(Request {
                        index: piece.index,
                        begin: offset,
                        length,
                    })
                    .await;

                offset += length;
            }
        }
    }

    /// Retrieve the connection stats from this peer and reset the stats.
    ///
    /// # Returns
    ///
    /// Returns the peer connection stats.
    pub(crate) async fn stats_and_reset(&self) -> PeerDataTransferStats {
        let mut stats = PeerDataTransferStats::default();

        {
            let mut mutex = self.inner.incoming_data_stats.write().await;
            stats.upload = mutex.transferred_bytes;
            stats.upload_rate = mutex.transferred_bytes_rate;
            mutex.reset();
        }
        {
            let mut mutex = self.inner.outgoing_data_stats.write().await;
            stats.download = mutex.transferred_bytes;
            stats.download_rate = mutex.transferred_bytes_rate;
            mutex.reset();
        }

        stats
    }

    /// Close this peer connection.
    /// The connection with the remote peer will be closed and this peer can no longer be used.
    pub async fn close(&self) {
        self.inner.close().await
    }

    async fn create_outgoing_stream(
        stream: io::Result<TcpStream>,
        addr: SocketAddr,
        torrent: Torrent,
        extensions: Extensions,
        runtime: Arc<Runtime>,
    ) -> Result<Self> {
        let (reader, writer) = stream
            .map(|e| split(e))
            .map_err(|e| Error::Io(format!("failed to connect to {}, {}", addr, e)))?;
        let (extension_event_sender, extension_event_receiver) = channel(10);
        let (reader_sender, peer_reader_receiver) = channel(10);
        let extension_registry = Self::create_extension_registry(&extensions);
        let peer_handle = PeerHandle::new();
        let inner = Arc::new(InnerPeer {
            handle: peer_handle,
            client_id: torrent.peer_id(),
            // the remote information is unknown until the handshake has been completed
            remote: RwLock::new(None),
            torrent,
            addr,
            state: RwLock::new(PeerState::Handshake),
            connection_type: ConnectionType::Outbound,
            // connections should always start in the choked state
            client_choke_state: RwLock::new(ChokeState::Choked),
            remote_choke_state: RwLock::new(ChokeState::Choked),
            // connections should always start in the not interested state
            client_interest_state: RwLock::new(InterestState::NotInterested),
            remote_interest_state: RwLock::new(InterestState::NotInterested),
            extension_event_sender,
            extensions,
            extension_registry,
            client_pieces: RwLock::new(BitVec::with_capacity(0)),
            remote_pieces: RwLock::new(BitVec::with_capacity(0)),
            // create new peer request buffers which are not running as the peer connection starts in the state choked
            client_pending_requests: PeerRequestBuffer::new(false),
            remote_pending_requests: PeerRequestBuffer::new(false),
            writer: Mutex::new(BufWriter::new(writer)),
            incoming_data_stats: RwLock::new(PeerTransferStats::default()),
            outgoing_data_stats: RwLock::new(PeerTransferStats::default()),
            callbacks: Default::default(),
            cancellation_token: CancellationToken::new(),
        });
        let peer = Self {
            handle: peer_handle,
            inner,
            runtime,
        };
        let mut peer_reader = PeerReader::new(
            peer.handle,
            reader,
            reader_sender,
            peer.inner.cancellation_token.clone(),
        );
        let mut peer_extension_events = PeerExtensionEvents {
            peer: peer.clone(),
            receiver: extension_event_receiver,
        };

        // as this is an outgoing connection, we're the once who initiate the handshake
        peer.inner.send_handshake().await?;
        // retrieve the incoming handshake from the reader
        // as the handshake is always 68 bytes long, we request a buffer of 68 bytes from the reader
        trace!("Waiting for peer handshake from {}", peer.inner.addr);
        let bytes = peer_reader.read(HANDSHAKE_MESSAGE_LEN).await?;
        peer.inner.validate_handshake(bytes).await?;

        // start the peer extension event loop
        // this moves the ownership of PeerExtensionEvents to a new thread
        peer.runtime
            .spawn(async move { peer_extension_events.start_events_loop().await });

        // start the peer read loop in a new thread
        // this moves the ownership of PeerReader to a new thread
        peer.runtime.spawn(async move {
            peer_reader.start_read_loop().await;
        });

        // start the main loop of the inner peer
        let main_loop = peer.clone();
        peer.runtime
            .spawn(async move { main_loop.start(peer_reader_receiver).await });

        peer.send_initial_messages().await?;

        Ok(peer)
    }

    /// Handle events that are sent from the peer reader.
    async fn handle_reader_event(&self, event: PeerReaderEvent) {
        match event {
            PeerReaderEvent::Closed => self.inner.cancellation_token.cancel(),
            PeerReaderEvent::Message(message, data_transfer) => {
                self.inner
                    .update_read_data_transfer_stats(data_transfer)
                    .await;

                if let Message::ExtendedPayload(extension_number, payload) = message {
                    trace!("Handling extended payload number {}", extension_number);
                    if let Some(extension) = self.find_supported_extension(extension_number).await {
                        if let Err(e) = extension.handle(payload.as_ref(), self).await {
                            error!(
                                "Failed to process extension message of peer {}, {}",
                                self, e
                            );
                        }
                    } else {
                        warn!(
                    "Received unsupported extension message of peer {} for extension number {}",
                    self, extension_number
                );
                    }
                } else {
                    self.inner.handle_received_message(message).await
                }
            }
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
    /// Returns the found client extension.
    async fn find_supported_extension(
        &self,
        extension_number: ExtensionNumber,
    ) -> Option<&Box<dyn Extension>> {
        // search for the given extension, by extensions number, in our own supported extensions
        let extension_registry = self.inner.client_extension_registry();
        if let Some(extension_name) = extension_registry
            .iter()
            .find(|(_, number)| extension_number == **number)
            .map(|(name, _)| name.clone())
        {
            trace!(
                "Looking up peer extensions {} for peer {}, {:?}",
                extension_name,
                self,
                self.inner.extensions
            );
            if let Some(extension) = self
                .inner
                .extensions
                .iter()
                .find(|e| e.name() == extension_name)
            {
                return Some(extension);
            } else {
                warn!(
                    "Extension name {} not found back for peer {}, supported {:?}",
                    extension_name, self, extension_registry
                )
            }
        } else {
            let extensions = self.inner.remote_extension_registry().await;
            debug!(
                "Extension number {} is not support by {}, supported remote {:?}",
                extension_number, self, extensions
            )
        }

        None
    }

    async fn send_initial_messages(&self) -> Result<()> {
        let extensions = self.remote_supported_extensions().await;

        // TODO: send the bitfield if we know it
        if extensions.contains(ExtensionFlags::LTEP) {
            self.inner.send_extended_handshake().await?;
        }
        if extensions.contains(ExtensionFlags::Fast) {
            // TODO: fix the fast extension
            // this is being sent even when the peer does not support the fast extension
            // peers are closing the connection if this happens
            // self.inner.send(Message::HaveNone).await?;
        }

        Ok(())
    }

    /// Create a new clone of this instance, which is only allowed by the internal processes
    /// of this library.
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
            inner: self.inner.clone(),
            runtime: self.runtime.clone(),
        }
    }

    /// Start the main loop of this peer.
    /// It handles the peer reader events and processing of the pending requests.
    async fn start(&self, mut peer_reader: Receiver<PeerReaderEvent>) {
        loop {
            select! {
                _ = self.inner.cancellation_token.cancelled() => break,
                _ = time::sleep(Duration::from_secs(KEEP_ALIVE_SECONDS)) => self.inner.send_keep_alive().await,
                event = peer_reader.recv() => {
                    if let Some(event) = event {
                        self.handle_reader_event(event).await;
                    }
                },
                request = self.inner.client_pending_requests.next() => self.inner.send_pending_request(request).await,
                request = self.inner.remote_pending_requests.next() => self.inner.handle_pending_request(request).await
            }
        }

        trace!("Peer {} main loop ended", self);
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
                (e.name(), extension_index)
            })
            .collect()
    }
}

impl Callbacks<PeerEvent> for Peer {
    fn add_callback(&self, callback: CoreCallback<PeerEvent>) -> CallbackHandle {
        self.inner.add_callback(callback)
    }

    fn remove_callback(&self, handle: CallbackHandle) {
        self.inner.remove_callback(handle)
    }
}

impl Display for Peer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

/// Information about transferred data over the peer connection.
#[derive(Debug, Clone)]
pub(crate) struct DataTransferStats {
    /// The total amount of bytes that have been transferred
    pub transferred_bytes: usize,
    /// The time it took in milliseconds to transfer the bytes
    pub elapsed: u128,
}

impl DataTransferStats {
    /// Get the rate of bytes transferred per second.
    pub fn rate(&self) -> u64 {
        // if a connection channel is closed
        // it can cause the elapsed time to be 0
        if self.elapsed == 0 {
            return 0;
        }

        ((self.transferred_bytes as u128 * 1000) / self.elapsed) as u64
    }
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

struct PeerExtensionEvents {
    peer: Peer,
    receiver: Receiver<PeerEvent>,
}

impl PeerExtensionEvents {
    async fn start_events_loop(&mut self) {
        loop {
            select! {
                _ = self.peer.inner.cancellation_token.cancelled() => break,
                event = self.receiver.recv() => {
                    if let Some(event) = event {
                        self.handle_event(event).await;
                    } else {
                      break;
                    }
                }
            }
        }

        trace!("Peer {} extensions loop ended", self.peer);
    }

    async fn handle_event(&mut self, event: PeerEvent) {
        let extensions = self.peer.remote_extension_registry().await;

        if let Some(extensions) = extensions {
            for extension in self
                .peer
                .inner
                .extensions
                .iter()
                .filter(|e| extensions.contains_key(&e.name()))
            {
                extension.on(event.clone(), &self.peer).await;
            }
        }
    }
}

#[derive(Debug)]
struct InnerPeer {
    /// The peer's unique identifier handle
    handle: PeerHandle,
    /// Our unique client peer id
    client_id: PeerId,
    /// The remote peer information, known after the initial handshake.
    remote: RwLock<Option<RemotePeer>>,
    /// The immutable torrent this peer connection belongs to.
    /// This is a weak reference to the [Torrent] and might be invalid if the peer is kept alive for invalid reasons.
    torrent: Torrent,
    /// The immutable address of the remote peer
    addr: SocketAddr,
    /// Identifies the connection direction (_incoming or outgoing_) of this peer
    connection_type: ConnectionType,
    /// The state of the client peer connection with the remote peer
    state: RwLock<PeerState>,

    /// The client choke state
    client_choke_state: RwLock<ChokeState>,
    /// The choke state of the remote peer
    remote_choke_state: RwLock<ChokeState>,

    /// The client interest state for the pieces of the remote peer
    client_interest_state: RwLock<InterestState>,
    /// The interest state of the remote peer for our available pieces
    remote_interest_state: RwLock<InterestState>,

    /// The event sender for extensions
    extension_event_sender: Sender<PeerEvent>,
    /// The extensions which are support by the application
    /// These are immutable once the peer has been created
    extensions: Extensions,
    extension_registry: ExtensionRegistry,

    /// The torrent pieces
    client_pieces: RwLock<BitVec>,
    /// The pieces of the remote peer
    remote_pieces: RwLock<BitVec>,

    /// The clients pending requests to the remote peer
    client_pending_requests: PeerRequestBuffer,
    /// The remote pending requests for this client
    remote_pending_requests: PeerRequestBuffer,

    /// The TCP write connection to the remote peer
    writer: Mutex<BufWriter<WriteHalf<TcpStream>>>,

    /// The data transfer info of the incoming channel (from the remote peer)
    incoming_data_stats: RwLock<PeerTransferStats>,
    /// The data transfer info of the outgoing channel (to the remote peer)
    outgoing_data_stats: RwLock<PeerTransferStats>,

    /// The callbacks which are triggered by this peer when an event is raised
    callbacks: CoreCallbacks<PeerEvent>,
    /// The cancellation token to cancel any async task within this peer on closure
    cancellation_token: CancellationToken,
}

impl InnerPeer {
    /// Retrieve the remote peer id.
    ///
    /// # Returns
    ///
    /// Returns the remote peer id when known, else `None`.
    async fn remote_id(&self) -> Option<PeerId> {
        let mutex = self.remote.read().await;
        mutex.as_ref().map(|e| e.peer_id.clone())
    }

    /// Retrieve the remote peer information.
    ///
    /// # Returns
    ///
    /// Returns the remote peer information when the handshake has been completed, else `None`.
    async fn remote_peer(&self) -> Option<RemotePeer> {
        let mutex = self.remote.read().await;
        mutex.as_ref().map(|e| e.clone())
    }

    /// Retrieve the supported extension registry of the remote peer.
    ///
    /// # Returns
    ///
    /// Returns the extension registry of the remote peer if known, else `None`.
    async fn remote_extension_registry(&self) -> Option<ExtensionRegistry> {
        self.remote
            .read()
            .await
            .as_ref()
            .map(|e| e.extensions.clone())
    }

    /// Retrieve the client peer extensions registry.
    /// This is the registry of our own client.
    ///
    /// # Returns
    ///
    /// Returns a reference to the client extension registry.
    fn client_extension_registry(&self) -> &ExtensionRegistry {
        &self.extension_registry
    }

    async fn handle_received_message(&self, message: Message) {
        debug!("Processing received peer {} message {:?}", self, message);
        match message {
            Message::KeepAlive => {
                trace!("Received keep alive from peer {}", self.client_id);
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
            Message::Have(piece) => self.update_remote_piece(piece as usize, true).await,
            Message::Bitfield(pieces) => self.update_remote_pieces(pieces).await,
            Message::Request(request) => self.remote_pending_requests.push(request).await,
            Message::ExtendedHandshake(handshake) => {
                self.update_extended_handshake(handshake).await
            }
            _ => warn!("Message handling not yet implemented for {:?}", message),
        }
    }

    async fn handle_pending_request(&self, request: Request) {
        todo!()
    }

    async fn validate_handshake(&self, buffer: Vec<u8>) -> Result<()> {
        let handshake = Handshake::from_bytes(buffer.as_ref())?;
        debug!("Received handshake {:?} from {}", handshake, self.addr);
        let torrent_info = self.torrent.metadata().await?;

        // verify that the peer sent the correct info hash which we expect
        if torrent_info.info_hash != handshake.info_hash {
            self.update_state(PeerState::Error).await;
            return Err(Error::Handshake(
                "received incorrect info hash from peer".to_string(),
            ));
        }

        // store the remote peer information
        trace!("Updating remote peer information for {}", handshake.peer_id);
        {
            let mut mutex = self.remote.write().await;
            *mutex = Some(RemotePeer {
                peer_id: handshake.peer_id,
                supported_extensions: handshake.supported_extensions,
                extensions: ExtensionRegistry::default(),
                client_name: None,
            });
        }

        debug!("Handshake of peer {} has been validated", self);
        self.invoke_event(PeerEvent::HandshakeCompleted).await;
        Ok(())
    }

    async fn update_remote_peer_choke_state(&self, state: ChokeState) {
        // update the choke state of the remote peer
        {
            let mut mutex = self.remote_choke_state.write().await;
            *mutex = state;
        }

        // update the pending requests buffer state
        if state == ChokeState::Choked {
            self.client_pending_requests.pause().await;
        } else {
            self.client_pending_requests.resume().await;
        }

        trace!("Remote peer {} entered {} state", self, state);
    }

    async fn update_remote_peer_interest_state(&self, state: InterestState) {
        let mut mutex = self.remote_interest_state.write().await;
        *mutex = state;
        trace!("Remote peer {} entered {} state", self, state);
    }

    async fn update_state(&self, state: PeerState) {
        let mut mutex = self.state.write().await;
        *mutex = state;
        debug!("Updated peer {} state to {:?}", self, state);

        self.invoke_event(PeerEvent::StateChanged(state)).await;
    }

    async fn update_pieces(&self, pieces: BitVec) {
        let mut mutex = self.client_pieces.write().await;
        *mutex = pieces;
        debug!("Updated peer {} with pieces information", self);
    }

    async fn update_remote_pieces(&self, pieces: BitVec) {
        let mut mutex = self.remote_pieces.write().await;
        *mutex = pieces;
        debug!("Updated peer {} with remote pieces information", self);
    }

    async fn update_remote_piece(&self, piece: usize, value: bool) {
        {
            let mut mutex = self.remote_pieces.write().await;
            mutex.set(piece, value);
        }

        // notify the torrent about the piece availability from this peer
        if value {
            self.torrent
                .notify_peer_has_piece(piece as PieceIndex)
                .await;
        }
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

        self.invoke_event(PeerEvent::ExtendedHandshakeCompleted)
            .await;
    }

    async fn update_read_data_transfer_stats(&self, data_transfer: DataTransferStats) {
        let mut mutex = self.incoming_data_stats.write().await;
        mutex.transferred_bytes += data_transfer.transferred_bytes;
        mutex.transferred_bytes_rate += data_transfer.rate();
        mutex.total_transferred_bytes += data_transfer.transferred_bytes as u64;
    }

    async fn update_write_data_transfer_stats(&self, data_transfer: DataTransferStats) {
        let mut mutex = self.outgoing_data_stats.write().await;
        mutex.transferred_bytes += data_transfer.transferred_bytes;
        mutex.transferred_bytes_rate += data_transfer.rate();
        mutex.total_transferred_bytes += data_transfer.transferred_bytes as u64;
    }

    async fn send_handshake(&self) -> Result<()> {
        self.update_state(PeerState::Handshake).await;
        let torrent_info = self.torrent.metadata().await?;

        let handshake =
            Handshake::new(torrent_info.info_hash, self.client_id, ExtensionFlags::LTEP);
        trace!("Trying to send handshake {:?}", handshake);
        match self
            .send_raw_bytes(TryInto::<Vec<u8>>::try_into(handshake)?)
            .await
        {
            Ok(_) => {
                debug!("Handshake has been successfully sent to {}", self.addr);
                Ok(())
            }
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
            port: None,
            your_ip: None,
            ipv4: None,
            ipv6: None,
        });

        self.send(message).await
    }

    async fn send(&self, message: Message) -> Result<()> {
        let message_bytes = TryInto::<Vec<u8>>::try_into(message)?;
        self.send_bytes(message_bytes).await
    }

    /// Send the given message to the remote peer.
    /// This method will prefix the message bytes with the BigEndian length bytes of the given message.
    async fn send_bytes<T: AsRef<[u8]>>(&self, message: T) -> Result<()> {
        let msg_length = message.as_ref().len();
        let mut buffer = vec![0u8; 4];

        // write the length of the given message as BigEndian in the first 4 bytes
        BigEndian::write_u32(&mut buffer[..4], msg_length as u32);
        // append the given message bytes to the buffer
        buffer.extend_from_slice(message.as_ref());

        self.send_raw_bytes(buffer).await
    }

    /// Send the given message bytes AS-IS to the remote peer.
    /// The given bytes should be a valid BitTorrent protocol message.
    async fn send_raw_bytes<T: AsRef<[u8]>>(&self, bytes: T) -> Result<()> {
        let mut mutex = self.writer.lock().await;
        let msg_length = bytes.as_ref().len();

        let start_time = Instant::now();
        timeout(
            Duration::from_secs(DEFAULT_CONNECTION_TIMEOUT_SECONDS),
            async {
                trace!("Sending a total of {} bytes to peer {}", msg_length, self);
                mutex.write_all(bytes.as_ref()).await?;
                mutex.flush().await?;
                debug!("Successfully sent {} bytes to peer {}", msg_length, self);
                Ok::<(), Error>(())
            },
        )
        .await??;
        drop(mutex);
        let elapsed = start_time.elapsed().as_millis();

        // update the connection stats
        self.update_write_data_transfer_stats(DataTransferStats {
            transferred_bytes: msg_length,
            elapsed,
        })
        .await;

        Ok(())
    }

    async fn send_pending_request(&self, request: Request) {
        // we normally shouldn't receive a request when the remote peer is choked
        // in case it does happen, we put the request back on the queue
        if *self.remote_choke_state.read().await == ChokeState::Choked {
            debug!("Received a request when the remote peer is choked, putting the request back on the queue");
            self.client_pending_requests.push(request).await;
            return;
        }

        if let Err(e) = self.send(Message::Request(request)).await {
            warn!("Failed to send pending request to peer {}, {}", self, e);
        }
    }

    async fn send_keep_alive(&self) {
        let message = Message::KeepAlive;

        match TryInto::<Vec<u8>>::try_into(message) {
            Ok(bytes) => {
                if let Err(e) = self.send_bytes(bytes).await {
                    warn!("Failed to send keep alive to peer {}, {}", self, e);
                }
            }
            Err(e) => warn!("Failed to parse keep alive message, {}", e),
        }
    }

    async fn invoke_event(&self, event: PeerEvent) {
        if let Err(e) = self.extension_event_sender.send(event.clone()).await {
            error!("Failed to send extensions event, {}", e)
        }

        self.callbacks.invoke(event);
    }

    async fn close(&self) {
        self.cancellation_token.cancel();
        self.update_state(PeerState::Closed).await;
    }
}

impl Callbacks<PeerEvent> for InnerPeer {
    fn add_callback(&self, callback: CoreCallback<PeerEvent>) -> CallbackHandle {
        self.callbacks.add_callback(callback)
    }

    fn remove_callback(&self, handle: CallbackHandle) {
        self.callbacks.remove_callback(handle)
    }
}

impl Display for InnerPeer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match block_in_place(self.remote.read()).as_ref() {
            Some(remote) => write!(f, "{}:{}", self.client_id, remote.peer_id),
            None => write!(f, "{}", self.client_id),
        }
    }
}

impl Drop for InnerPeer {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
        trace!("Peer {} is being dropped", self)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::sync::Arc;

    use log::info;
    use tokio::net::TcpListener;
    use tokio::runtime::Runtime;

    use popcorn_fx_core::assert_timeout;
    use popcorn_fx_core::core::torrents::magnet::Magnet;
    use popcorn_fx_core::core::utils::network::available_socket;
    use popcorn_fx_core::testing::{init_logger, read_test_file_to_bytes};

    use super::*;
    use crate::torrents::peers::extensions::metadata::MetadataExtension;
    use crate::torrents::{Torrent, TorrentFlags, TorrentInfo, TorrentRequest};

    #[test]
    fn test_peer_new_outbound() {
        init_logger();
        let magnet = Magnet::from_str("magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce").unwrap();
        let torrent_info = TorrentInfo::try_from(magnet).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let request = TorrentRequest {
            metadata: torrent_info.clone(),
            options: TorrentFlags::None,
            peer_listener_port: 6881,
            extensions: vec![Box::new(MetadataExtension::new())],
            peer_timeout: Some(Duration::from_secs(2)),
            tracker_timeout: Some(Duration::from_secs(2)),
            runtime: Some(runtime.clone()),
        };
        let torrent = Torrent::try_from(request).unwrap();

        let announcement = runtime.block_on(torrent.announce()).unwrap();
        let mut peer: Option<Peer> = None;

        for peer_addr in announcement.peers {
            match runtime.block_on(Peer::new_outbound(
                peer_addr,
                torrent.clone(),
                vec![Box::new(MetadataExtension::new())],
                runtime.clone(),
            )) {
                Ok(e) => {
                    peer = Some(e);
                    break;
                }
                Err(e) => warn!(
                    "Failed to establish connection with peer {}, {}",
                    peer_addr, e
                ),
            }
        }

        let state = runtime.block_on(peer.as_ref().unwrap().state());

        loop {}
        assert_timeout!(Duration::from_secs(10), state != PeerState::Handshake);
    }

    #[test]
    fn test_peer_new_outbound_mock() {
        init_logger();
        let magnet = Magnet::from_str("magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce").unwrap();
        let torrent_info = TorrentInfo::try_from(magnet).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let request = TorrentRequest {
            metadata: torrent_info.clone(),
            options: Default::default(),
            peer_listener_port: 6881,
            extensions: vec![],
            peer_timeout: Some(Duration::from_secs(1)),
            tracker_timeout: None,
            runtime: Some(runtime.clone()),
        };
        let mock_addr = available_socket();
        let mut listener = runtime.block_on(TcpListener::bind(&mock_addr)).unwrap();
        let torrent = Torrent::try_from(request).unwrap();

        let stream_info_hash = torrent_info.info_hash.clone();
        runtime.spawn(async move {
            loop {
                let (stream, addr) = listener.accept().await.unwrap();
                debug!("[Mock] Accepted connection from {}", addr);

                tokio::spawn(handle_tcp_stream(stream, stream_info_hash.clone()));
            }
        });

        let mut peer = runtime
            .block_on(Peer::new_outbound(
                mock_addr,
                torrent,
                vec![Box::new(MetadataExtension::new())],
                runtime.clone(),
            ))
            .unwrap();
    }

    #[test]
    fn test_peer_close() {
        init_logger();
        let torrent_info_data = read_test_file_to_bytes("debian-udp.torrent");
        let torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let request = TorrentRequest {
            metadata: torrent_info.clone(),
            options: Default::default(),
            peer_listener_port: 6881,
            extensions: vec![],
            peer_timeout: Some(Duration::from_secs(1)),
            tracker_timeout: None,
            runtime: Some(runtime.clone()),
        };
        let mock_addr = available_socket();
        let mut listener = runtime.block_on(TcpListener::bind(&mock_addr)).unwrap();
        let torrent = Torrent::try_from(request).unwrap();

        let stream_info_hash = torrent_info.info_hash.clone();
        let stream_runtime = runtime.clone();
        runtime.spawn(async move {
            loop {
                let (stream, addr) = listener.accept().await.unwrap();
                debug!("[Mock] Accepted connection from {}", addr);

                stream_runtime.spawn(handle_tcp_stream(stream, stream_info_hash.clone()));
            }
        });

        let peer = runtime
            .block_on(Peer::new_outbound(
                mock_addr,
                torrent,
                vec![Box::new(MetadataExtension::new())],
                runtime.clone(),
            ))
            .unwrap();

        runtime.block_on(peer.close());
        drop(peer);

        // subtract the mock thread from the running tasks
        let alive_tasks = runtime.metrics().num_alive_tasks() - 1;
        assert_eq!(0, alive_tasks, "expected all tasks to have been ended");
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

    async fn handle_tcp_stream(mut stream: TcpStream, info_hash: InfoHash) {
        let mut buffer = [0; 68];
        let read = stream
            .read_exact(&mut buffer)
            .await
            .map_err(|e| {
                error!("[Mock] Failed to read handshake, {}", e);
                e
            })
            .unwrap();
        debug!("[Mock] Read a total of {} bytes", read);

        let handshake = Handshake::from_bytes(&buffer[0..])
            .map_err(|e| {
                error!("[Mock] Failed to parse handshake, {}", e);
                e
            })
            .unwrap();
        info!("[Mock] Received handshake: {:?}", handshake);

        if info_hash != handshake.info_hash {
            handle_stream_error("[Mock] Handshake info hash does not match", stream).await;
            return;
        }

        let peer_id = PeerId::new();
        let handshake = Handshake::new(info_hash, peer_id, ExtensionFlags::None);

        stream
            .write_all(TryInto::<Vec<u8>>::try_into(handshake).unwrap().as_ref())
            .await
            .unwrap();
        debug!("[Mock] Written handshake");
    }

    async fn handle_stream_error(msg: &str, mut stream: TcpStream) {
        error!("[Mock] {}", msg);
        stream.shutdown().await.unwrap();
    }
}
