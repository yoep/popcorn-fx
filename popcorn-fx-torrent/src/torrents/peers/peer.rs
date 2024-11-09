use crate::torrents::peers::bt_protocol::{ExtendedHandshake, ExtensionFlag, Handshake, Message};
use crate::torrents::peers::extensions::metadata::EXTENSION_NAME_METADATA;
use crate::torrents::peers::extensions::{
    Extension, ExtensionNumber, ExtensionRegistry, Extensions,
};

use crate::torrents::peers::{Error, PeerId, Result};
use crate::torrents::torrent_commands::{
    TorrentCommand, TorrentCommandResponse, TorrentCommandSender,
};
use crate::torrents::{channel, InfoHash, TorrentHandle, TorrentInfo, TorrentMetadata};
use byteorder::BigEndian;
use byteorder::ByteOrder;
use derive_more::Display;
use log::{debug, error, trace, warn};
use popcorn_fx_core::core::{
    block_in_place, CallbackHandle, Callbacks, CoreCallback, CoreCallbacks,
};
use std::fmt::{Debug, Display, Formatter};
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{
    split, AsyncRead, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter, ReadHalf, WriteHalf,
};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::{Mutex, RwLock};
use tokio::time::timeout;
use tokio::{select, time};
use tokio_util::sync::CancellationToken;

const DEFAULT_CONNECTION_TIMEOUT_SECONDS: u64 = 10;
const KEEP_ALIVE_SECONDS: u64 = 120;
const HANDSHAKE_MESSAGE_LEN: usize = 68;

/// The peer specific event callbacks.
pub type PeerCallback = CoreCallback<PeerEvent>;

/// The choke states of a peer.
#[derive(Debug, Display, Clone, Copy, PartialEq)]
pub enum ChokeState {
    #[display(fmt = "choked")]
    Choked,
    #[display(fmt = "un-choked")]
    UnChoked,
}

#[derive(Debug, Display, Clone, Copy, PartialEq)]
pub enum InterestedState {
    #[display(fmt = "interested")]
    Interested,
    #[display(fmt = "not interested")]
    NotInterested,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionType {
    Inbound,
    Outbound,
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
    pub supported_extensions: ExtensionFlag,
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

#[derive(Debug)]
pub struct Peer {
    inner: Arc<InnerPeer>,
    runtime: Arc<Runtime>,
}

impl Peer {
    pub async fn new_outbound(
        addr: SocketAddr,
        torrent_info_hash: InfoHash,
        torrent_handle: TorrentHandle,
        torrent_command_sender: TorrentCommandSender,
        client_id: PeerId,
        extensions: Extensions,
        runtime: Arc<Runtime>,
    ) -> Result<Self> {
        trace!("Trying to connect to peer {}", addr);
        select! {
            _ = time::sleep(Duration::from_secs(DEFAULT_CONNECTION_TIMEOUT_SECONDS)) => {
                Err(Error::Io(format!("failed to connect to {}, connection timed out", addr)))
            },
            stream = TcpStream::connect(&addr) => Self::create_outgoing_stream(stream, client_id, addr, torrent_info_hash, torrent_handle, torrent_command_sender, extensions, runtime).await
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
    pub async fn remote_supported_extensions(&self) -> ExtensionFlag {
        let mutex = self.inner.remote.read().await;
        mutex
            .as_ref()
            .map(|e| e.supported_extensions.clone())
            .unwrap_or(ExtensionFlag::None)
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
    pub fn info_hash(&self) -> InfoHash {
        self.inner.torrent_info_hash.clone()
    }
    /// Retrieve the torrent metadata.
    /// This info is requested from the torrent that created this peer.
    pub async fn metadata(&self) -> Option<TorrentInfo> {
        match self
            .inner
            .torrent_command_sender
            .send(TorrentCommand::Metadata)
            .await
            .and_then(|response| {
                if let TorrentCommandResponse::Metadata(metadata) = response {
                    return Ok(metadata);
                }

                Err(channel::Error::UnexpectedCommandResponse(
                    "TorrentCommandResponse::Metadata".to_string(),
                    format!("{:?}", response),
                ))?
            }) {
            Ok(metadata) => Some(metadata),
            Err(e) => {
                warn!(
                    "Failed to retrieve torrent metadata for peer {}, {}",
                    self, e
                );
                None
            }
        }
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
    pub fn update_torrent_metadata(&self, metadata: TorrentMetadata) {
        self.inner
            .torrent_command_sender
            .send_void(TorrentCommand::AddMetadata(metadata))
            .unwrap()
    }

    /// Close this peer connection.
    /// The connection with the remote peer will be closed and this peer can no longer be used.
    pub async fn close(&self) {
        self.inner.close().await
    }

    async fn create_outgoing_stream(
        stream: io::Result<TcpStream>,
        client_id: PeerId,
        addr: SocketAddr,
        torrent_info_hash: InfoHash,
        torrent_handle: TorrentHandle,
        torrent_command_sender: TorrentCommandSender,
        extensions: Extensions,
        runtime: Arc<Runtime>,
    ) -> Result<Self> {
        let (reader, writer) = stream
            .map(|e| split(e))
            .map_err(|e| Error::Io(format!("failed to connect to {}, {}", addr, e)))?;
        let (extension_event_sender, extension_event_receiver) = channel(10);
        let inner = Arc::new(InnerPeer {
            client_id,
            remote: RwLock::new(None),
            torrent_info_hash,
            torrent_handle,
            torrent_command_sender,
            // connections should always start in the choked state
            choke_state: ChokeState::Choked,
            peer_choke_state: RwLock::new(ChokeState::Choked),
            // connections should always start in the not interested state
            interested_state: InterestedState::NotInterested,
            addr,
            connection_type: ConnectionType::Outbound,
            state: RwLock::new(PeerState::Handshake),
            extension_event_sender,
            extensions,
            writer: Mutex::new(BufWriter::new(writer)),
            callbacks: Default::default(),
            cancellation_token: CancellationToken::new(),
        });
        let peer = Self { inner, runtime };
        let mut peer_reader = PeerReader {
            addr,
            reader: BufReader::new(reader),
            peer: peer.clone(),
        };
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

        peer.start_keep_alive_loop();
        peer.send_initial_messages().await?;

        Ok(peer)
    }

    async fn handle_received_message(&self, message: Message) {
        if let Message::ExtendedPayload(extension_number, payload) = message {
            trace!(
                "Handling extended payload (type {}): {}",
                extension_number,
                String::from_utf8_lossy(payload.as_ref())
            );
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

    async fn find_supported_extension(
        &self,
        extension_number: ExtensionNumber,
    ) -> Option<&Box<dyn Extension>> {
        let extension_registry = self.inner.extension_registry().await;
        if let Some(extension_name) = extension_registry.and_then(|registry| {
            registry
                .iter()
                .find(|(_, number)| extension_number == **number)
                .map(|(name, _)| name.clone())
        }) {
            if let Some(extension) = self
                .inner
                .extensions
                .iter()
                .find(|e| e.name() == extension_name)
            {
                return Some(extension);
            } else {
                warn!(
                    "Extension number {} is not support by {}",
                    extension_number, self
                )
            }
        }

        None
    }

    async fn send_initial_messages(&self) -> Result<()> {
        let extensions = self.remote_supported_extensions().await;

        if extensions.contains(ExtensionFlag::Extensions) {
            self.inner.send_extended_handshake().await?;
        }
        if extensions.contains(ExtensionFlag::Fast) {
            // TODO: fix the fast extension
            // this is being sent even when the peer does not support the fast extension
            // peers are closing the connection if this happens
            // self.inner.send(Message::HaveNone).await?;
        }

        Ok(())
    }

    /// Create a new thread which has a shared reference to the [InnerPeer].
    /// This thread will send every [KEEP_ALIVE_SECONDS] seconds a [Message::KeepAlive] to the remote peer.
    ///
    /// The loop is automatically cancelled when the [InnerPeer] is dropped or [InnerPeer::close] is called.
    fn start_keep_alive_loop(&self) {
        let inner = self.inner.clone();
        self.runtime.spawn(async move {
            loop {
                select! {
                    _ = inner.cancellation_token.cancelled() => break,
                    _ = time::sleep(Duration::from_secs(KEEP_ALIVE_SECONDS)) => inner.send_keep_alive().await,
                }
            }
        });
    }

    /// Create a new clone of this instance, which is only allowed by the internal processes
    /// of this library.
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            runtime: self.runtime.clone(),
        }
    }
}

impl Callbacks<PeerEvent> for Peer {
    fn add(&self, callback: CoreCallback<PeerEvent>) -> CallbackHandle {
        self.inner.add(callback)
    }

    fn remove(&self, handle: CallbackHandle) {
        self.inner.remove(handle)
    }
}

impl Display for Peer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[derive(Debug)]
struct PeerReader<R>
where
    R: AsyncRead + Unpin,
{
    addr: SocketAddr,
    reader: BufReader<R>,
    peer: Peer,
}

impl<R> PeerReader<R>
where
    R: AsyncRead + Unpin,
{
    async fn start_read_loop(&mut self) {
        loop {
            let mut buffer = vec![0u8; 4];

            select! {
                _ = self.peer.inner.cancellation_token.cancelled() => break,
                read_result = self.reader.read(&mut buffer) => {
                    match read_result {
                        Ok(0) => {
                            trace!("Peer reader EOF for {}", self.addr);
                            break
                        },
                        Ok(buffer_size) => {
                            if let Err(e) = self.read_next(&buffer, buffer_size).await {
                                error!("{}", e);
                                break
                            }
                        },
                        Err(e) => {
                            error!("{}", Error::from(e));
                            break
                        }
                    }
                }
            }
        }

        trace!("Main read loop of peer {} ended", self.addr);
        self.peer.close().await;
    }

    async fn read(&mut self, len: usize) -> Result<Vec<u8>> {
        let mut buffer = vec![0u8; len];
        let read_result = self.reader.read_exact(&mut buffer).await;

        match read_result {
            Ok(0) => Err(Error::Closed),
            Ok(_) => Ok(buffer),
            Err(e) => Err(Error::Io(e.to_string())),
        }
    }

    async fn read_next(&mut self, buffer: &[u8], buffer_size: usize) -> Result<()> {
        // we expect to receive the incoming message length as a BigEndian
        if buffer_size != 4 {
            warn!("Invalid message length {}", buffer_size);
            return Ok(());
        }

        let length = BigEndian::read_u32(buffer);
        let bytes = self.read(length as usize).await?;

        // we want to unblock the reader thread as soon as possible
        // so we're going to move this whole process into a new separate thread
        let message_peer = self.peer.clone();
        self.peer.runtime.spawn(async move {
            match Message::try_from(bytes.as_ref()) {
                Ok(msg) => message_peer.handle_received_message(msg).await,
                Err(e) => warn!("Received invalid message payload, {}", e),
            }
        });

        Ok(())
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

        trace!("Extensions events loop of peer {} ended", self.peer);
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

struct InnerPeer {
    /// Our unique client peer id
    client_id: PeerId,
    /// The remote peer data
    remote: RwLock<Option<RemotePeer>>,
    /// The info hash information of the torrent
    torrent_info_hash: InfoHash,
    /// The owning torrent handle of this peer
    torrent_handle: TorrentHandle,
    /// The command sender for communicating with the owning torrent
    torrent_command_sender: TorrentCommandSender,
    /// Our own current choke state with the remote peer
    choke_state: ChokeState,
    /// The choke state of the remote peer
    peer_choke_state: RwLock<ChokeState>,
    /// The current interest state into the remote peer
    interested_state: InterestedState,
    /// The remote peer address
    addr: SocketAddr,
    /// Identifies the connection direction (_incoming or outgoing_) of this peer
    connection_type: ConnectionType,
    /// The state of our peer connection with the remote peer
    state: RwLock<PeerState>,

    /// The event sender for extensions
    extension_event_sender: Sender<PeerEvent>,
    /// The extensions which are support by the application
    /// These are immutable once the peer has been created
    extensions: Extensions,

    /// The TCP write connection to the remote peer
    writer: Mutex<BufWriter<WriteHalf<TcpStream>>>,

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
    async fn extension_registry(&self) -> Option<ExtensionRegistry> {
        self.remote
            .read()
            .await
            .as_ref()
            .map(|e| e.extensions.clone())
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
            Message::ExtendedHandshake(handshake) => {
                self.update_extended_handshake(handshake).await
            }
            _ => warn!("Message handling not yet implemented for {:?}", message),
        }
    }

    async fn validate_handshake(&self, buffer: Vec<u8>) -> Result<()> {
        let handshake = Handshake::from_bytes(buffer.as_ref())?;
        debug!("Received handshake {:?} from {}", handshake, self.addr);

        // verify that the peer sent the correct info hash which we expect
        if self.torrent_info_hash != handshake.info_hash {
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
        let mut mutex = self.peer_choke_state.write().await;
        *mutex = state;
        trace!("Remote peer {} entered {} state", self, state);
    }

    async fn update_state(&self, state: PeerState) {
        let mut mutex = self.state.write().await;
        *mutex = state;
        debug!("Updated peer {} state to {:?}", self, state);

        self.invoke_event(PeerEvent::StateChanged(state)).await;
    }

    async fn update_extended_handshake(&self, handshake: ExtendedHandshake) {
        {
            let mut mutex = self.remote.write().await;
            if let Some(remote) = mutex.as_mut() {
                remote.extensions = handshake.m;
                remote.client_name = handshake.client;
                // drop the mutex as the Display impl requires it to print the info of the remote peer
                drop(mutex);
                debug!("Updated peer {} with extended handshake information", self);
            } else {
                warn!("Received extended handshake before the initial handshake was completed");
            }
        }

        self.invoke_event(PeerEvent::ExtendedHandshakeCompleted)
            .await;
    }

    async fn send_handshake(&self) -> Result<()> {
        self.update_state(PeerState::Handshake).await;

        let handshake = Handshake::new(
            self.torrent_info_hash.clone(),
            self.client_id,
            ExtensionFlag::Extensions,
        );
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
        let message = Message::ExtendedHandshake(ExtendedHandshake {
            m: vec![(EXTENSION_NAME_METADATA.to_string(), 1)]
                .into_iter()
                .collect(),
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
        Ok(())
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
    fn add(&self, callback: CoreCallback<PeerEvent>) -> CallbackHandle {
        self.callbacks.add(callback)
    }

    fn remove(&self, handle: CallbackHandle) {
        self.callbacks.remove(handle)
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

impl Debug for InnerPeer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InnerPeer")
            .field("client_id", &self.client_id)
            .field("remote", &self.remote)
            .field("torrent_info_hash", &self.torrent_info_hash)
            .field("torrent_handle", &self.torrent_handle)
            .field("choke_state", &self.choke_state)
            .field("peer_choke_state", &self.peer_choke_state)
            .field("addr", &self.addr)
            .field("interested_state", &self.interested_state)
            .field("connection_type", &self.connection_type)
            .field("state", &self.state)
            .field("extensions", &self.extensions)
            .field("writer", &self.writer)
            .field("cancellation_token", &self.cancellation_token)
            .finish()
    }
}

impl Drop for InnerPeer {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;
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
    use crate::torrents::{Torrent, TorrentInfo, TorrentRequest};

    #[test]
    fn test_peer_new_outbound() {
        init_logger();
        let magnet = Magnet::from_str("magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce").unwrap();
        let torrent_info = TorrentInfo::try_from(magnet).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let request = TorrentRequest {
            metadata: torrent_info.clone(),
            options: Default::default(),
            peer_listener_port: 6881,
            timeout: Some(Duration::from_secs(1)),
            runtime: Some(runtime.clone()),
        };
        let torrent = Torrent::try_from(request).unwrap();

        let announcement = runtime.block_on(torrent.announce()).unwrap();
        let mut peer: Option<Peer> = None;

        for peer_addr in announcement.peers {
            match runtime.block_on(Peer::new_outbound(
                peer_addr,
                torrent_info.info_hash.clone(),
                torrent.handle(),
                torrent.command_sender(),
                torrent.peer_id(),
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
            timeout: Some(Duration::from_secs(1)),
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
                torrent_info.info_hash.clone(),
                torrent.handle(),
                torrent.command_sender(),
                torrent.peer_id(),
                vec![Box::new(MetadataExtension::new())],
                runtime.clone(),
            ))
            .unwrap();
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
        let handshake = Handshake::new(info_hash, peer_id, ExtensionFlag::None);

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
