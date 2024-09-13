use byteorder::BigEndian;
use byteorder::ByteOrder;
use derive_more::Display;
use log::{debug, error, trace, warn};
use std::fmt::{Display, Formatter};
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{split, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;
use tokio::time::timeout;
use tokio::{select, time};
use tokio_util::sync::CancellationToken;

use crate::torrents::channel::{new_command_channel, ChannelError, CommandSender};
use crate::torrents::peers::bt_connection::{ExtendedHandshake, ExtensionFlag, Handshake, Message};
use crate::torrents::peers::extensions::metadata::EXTENSION_NAME_METADATA;
use crate::torrents::peers::extensions::Extensions;
use crate::torrents::peers::peer_commands::{
    PeerCommand, PeerCommandInstruction, PeerCommandReceiver, PeerCommandResponse,
    PeerCommandSender,
};
use crate::torrents::peers::{PeerError, PeerId, Result};
use crate::torrents::torrent_commands::TorrentCommandSender;
use crate::torrents::{InfoHash, TorrentHandle};

const DEFAULT_CONNECTION_TIMEOUT_SECONDS: u64 = 10;

#[derive(Debug, Display, Clone, PartialEq)]
pub enum ChokeState {
    #[display(fmt = "choked")]
    Choked,
    #[display(fmt = "un-choked")]
    UnChoked,
}

#[derive(Debug, Display, Clone, PartialEq)]
pub enum InterestedState {
    #[display(fmt = "interested")]
    Interested,
    #[display(fmt = "not interested")]
    NotInterested,
}

#[derive(Debug, Clone, PartialEq)]
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
    pub extensions: Extensions,
    pub client_name: Option<String>,
}

#[derive(Debug)]
pub struct Peer {
    command_sender: PeerCommandSender,
    cancellation_token: CancellationToken,
}

impl Peer {
    pub async fn new_outbound(
        addr: SocketAddr,
        torrent_handle: TorrentHandle,
        torrent_command_sender: TorrentCommandSender,
        info_hash: InfoHash,
        client_id: PeerId,
        runtime: Arc<Runtime>,
    ) -> Result<Self> {
        trace!("Trying to connect to peer {}", addr);
        select! {
            _ = time::sleep(Duration::from_secs(DEFAULT_CONNECTION_TIMEOUT_SECONDS)) => {
                Err(PeerError::Io(format!("failed to connect to {}, connection timed out", addr)))
            },
            stream = TcpStream::connect(&addr) => Self::create_outgoing_stream(stream, client_id, addr, torrent_handle, torrent_command_sender, info_hash, runtime).await
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
    /// Returns the remote peer id as reference if the handshake has been completed, else `None`.
    pub async fn id(&self) -> Option<PeerId> {
        match self.command_sender.send(PeerCommand::Remote).await {
            Ok(response) => {
                if let PeerCommandResponse::Remote(remote) = response {
                    return remote.map(|e| e.peer_id);
                }

                warn!(
                    "Expected PeerCommandResponse::RemoteId, but got {:?} instead",
                    response
                )
            }
            Err(e) => error!("Failed to retrieve remote peer id, {}", e),
        }

        None
    }

    /// Retrieve the current state of this peer.
    ///
    /// # Returns
    ///
    /// Returns the current state of this peer.
    pub async fn state(&self) -> PeerState {
        match self.command_sender.send(PeerCommand::State).await {
            Ok(response) => {
                if let PeerCommandResponse::State(state) = response {
                    return state;
                }

                warn!(
                    "Expected PeerCommandResponse::State, but got {:?} instead",
                    response
                )
            }
            Err(e) => error!("Failed to retrieve peer state, {}", e),
        }

        PeerState::Error
    }

    async fn create_outgoing_stream(
        stream: io::Result<TcpStream>,
        client_id: PeerId,
        addr: SocketAddr,
        torrent_handle: TorrentHandle,
        torrent_command_sender: TorrentCommandSender,
        info_hash: InfoHash,
        runtime: Arc<Runtime>,
    ) -> Result<Self> {
        let (reader, writer) = stream
            .map(|e| split(e))
            .map_err(|e| PeerError::Io(format!("failed to connect to {}, {}", addr, e)))?;
        let (command_sender, command_receiver) = new_command_channel();
        let cancellation_token = CancellationToken::new();
        let mut inner = InnerPeer {
            client_id,
            remote: None,
            torrent_handle,
            torrent_command_sender,
            info_hash,
            // connections should always start in the choked state
            choke_state: ChokeState::Choked,
            peer_choke_state: ChokeState::Choked,
            // connections should always start in the not interested state
            interested_state: InterestedState::NotInterested,
            addr,
            connection_type: ConnectionType::Outbound,
            state: RwLock::new(PeerState::Handshake),
            writer,
            cancellation_token: cancellation_token.clone(),
        };
        let mut peer_reader = PeerReader {
            addr,
            reader,
            sender: command_sender.clone(),
            cancellation_token: cancellation_token.clone(),
        };

        // as this is an outgoing connection, we're the once who initiate the handshake
        inner.send_handshake().await?;
        // retrieve the incoming handshake from the reader
        // as the handshake is always 68 bytes long, we request a buffer of 68 bytes from the reader
        trace!("Waiting for peer handshake from {}", inner.addr);
        let bytes = peer_reader.read(68).await?;
        inner.validate_handshake(bytes).await?;

        // start the peer command loop in a new thread
        // this moves the ownership of InnerPeer to a new thread
        runtime.spawn(async move {
            inner.start_command_loop(command_receiver).await;
        });

        // start the peer read loop in a new thread
        // this moves the ownership of PeerReader to a new thread
        runtime.spawn(async move {
            peer_reader.start_read_loop().await;
        });

        let instance = Self {
            command_sender,
            cancellation_token,
        };
        instance.send_initial_messages().await?;

        Ok(instance)
    }

    async fn send_initial_messages(&self) -> Result<()> {
        // retrieve the supported extensions from the remote peer
        let extensions = self
            .command_sender
            .send(PeerCommand::SupportedExtensions)
            .await
            .map(|response| {
                if let PeerCommandResponse::SupportedExtensions(e) = response {
                    return Ok(e);
                }

                Err(ChannelError::UnexpectedCommandResponse(
                    "PeerCommandResponse::SupportedExtensions".to_string(),
                    format!("{:?}", response),
                ))
            })
            .map_err(|e| PeerError::Io(e.to_string()))??;
        if extensions.contains(ExtensionFlag::Extensions) {
            // send the extended handshake
            self.command_sender
                .send(PeerCommand::SendExtendedHandshake)
                .await
                .map(|response| {
                    if let PeerCommandResponse::SendExtendedHandshake(e) = response {
                        return e;
                    }

                    Err(ChannelError::UnexpectedCommandResponse(
                        "PeerCommandResponse::SendExtendedHandshake".to_string(),
                        format!("{:?}", response),
                    ))?
                })
                .map_err(|e| PeerError::Io(e.to_string()))??;
        }

        if extensions.contains(ExtensionFlag::Fast) {
            // TODO check if we have any pieces available
            self.command_sender.send_void(PeerCommand::SendHaveNone)?;
        }

        Ok(())
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        let _ = self.command_sender.send_void(PeerCommand::Close);
    }
}

#[derive(Debug)]
struct PeerReader {
    addr: SocketAddr,
    reader: ReadHalf<TcpStream>,
    sender: PeerCommandSender,
    cancellation_token: CancellationToken,
}

impl PeerReader {
    async fn start_read_loop(&mut self) {
        loop {
            let mut buffer = vec![0u8; 4];

            select! {
                _ = self.cancellation_token.cancelled() => break,
                read_result = self.reader.read(&mut buffer) => {
                    match read_result {
                        Ok(0) => break,
                        Ok(buffer_size) => {
                            if let Err(e) = self.read_next(&buffer, buffer_size).await {
                                error!("{}", e);
                                break
                            }
                        },
                        Err(e) => {
                            error!("{}", PeerError::from(e));
                            break
                        }
                    }
                }
            }
        }

        trace!("Main read loop of peer {} ended", self.addr);
        if let Err(e) = self.sender.send_void(PeerCommand::Close) {
            warn!("Failed to send close peer command for {}, {}", self.addr, e);
        }
    }

    async fn read(&mut self, len: usize) -> Result<Vec<u8>> {
        let mut buffer = vec![0u8; len];
        let read_result = self.reader.read_exact(&mut buffer).await;

        match read_result {
            Ok(0) => Err(PeerError::Closed),
            Ok(_) => Ok(buffer),
            Err(e) => Err(PeerError::Io(e.to_string())),
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

        match Message::try_from(bytes.as_ref()) {
            Ok(msg) => self.sender.send_void(PeerCommand::Message(msg))?,
            Err(e) => warn!("Received invalid message payload, {}", e),
        }

        Ok(())
    }
}

#[derive(Debug)]
struct InnerPeer {
    /// Our unique client peer id
    client_id: PeerId,
    /// The remote peer data
    remote: Option<RemotePeer>,
    /// The owning torrent handle of this peer
    torrent_handle: TorrentHandle,
    /// The command sender for communicating with the owning torrent
    torrent_command_sender: TorrentCommandSender,
    /// The torrent metadata
    info_hash: InfoHash,
    /// Our own current choke state with the remote peer
    choke_state: ChokeState,
    /// The choke state of the remote peer
    peer_choke_state: ChokeState,
    /// The current interest state into the remote peer
    interested_state: InterestedState,
    /// The remote peer address
    addr: SocketAddr,
    /// Identifies the connection direction (_incoming or outgoing_) of this peer
    connection_type: ConnectionType,
    /// The state of our peer connection with the remote peer
    state: RwLock<PeerState>,

    /// The TCP write connection to the remote peer
    writer: WriteHalf<TcpStream>,

    /// The cancellation token to cancel any async task within this peer on closure
    cancellation_token: CancellationToken,
}

impl InnerPeer {
    /// Start the command loop for this peer.
    /// This is the main logic loop of a peer which delegates all incoming and outgoing actions.
    async fn start_command_loop(&mut self, mut command_receiver: PeerCommandReceiver) {
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                _ = time::sleep(Duration::from_secs(180)) => self.send_keep_alive().await,
                command = command_receiver.recv() => {
                    if let Some(command) = command {
                        self.handle_command_instruction(command).await;
                    } else {
                        break
                    }
                }
            }
        }

        debug!("Closing peer {:?}", self);
    }

    /// Retrieve the remote peer id.
    ///
    /// # Returns
    ///
    /// Returns the remote peer id when known, else `None`.
    fn remote_id(&self) -> Option<PeerId> {
        self.remote.as_ref().map(|e| e.peer_id.clone())
    }

    async fn handle_command_instruction(&mut self, mut instruction: PeerCommandInstruction) {
        let instruction_info = format!("{:?}", instruction);
        trace!("Received peer {:?}", instruction);
        let command_result = match instruction.command {
            PeerCommand::ClientId => {
                instruction.respond(PeerCommandResponse::ClientId(self.client_id))
            }
            PeerCommand::Remote => {
                instruction.respond(PeerCommandResponse::Remote(self.remote.clone()))
            }
            PeerCommand::State => {
                let mutex = self.state.read().await;
                instruction.respond(PeerCommandResponse::State(mutex.clone()))
            }
            PeerCommand::SupportedExtensions => {
                instruction.respond(PeerCommandResponse::SupportedExtensions(
                    self.remote
                        .as_ref()
                        .map(|e| e.supported_extensions.clone())
                        .unwrap_or(ExtensionFlag::None),
                ))
            }
            PeerCommand::SendExtendedHandshake => instruction.respond(
                PeerCommandResponse::SendExtendedHandshake(self.send_extended_handshake().await),
            ),
            PeerCommand::SendHaveNone => {
                if let Err(e) = self.send(Message::HaveNone).await {
                    warn!("Failed to send have none to peer, {}", e);
                }
                Ok(())
            }
            PeerCommand::Message(message) => {
                self.process_message(message).await;
                Ok(())
            }
            PeerCommand::Close => {
                self.cancellation_token.cancel();
                Ok(())
            }
        };

        if let Err(e) = command_result {
            error!("Failed to process peer {:?}, {}", instruction_info, e);
        }
    }

    async fn process_message(&mut self, message: Message) {
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
            Message::ExtendedHandshake(handshake) => self.update_extended_handshake(handshake),
            _ => warn!("Not yet implemented"),
        }
    }

    async fn validate_handshake(&mut self, buffer: Vec<u8>) -> Result<()> {
        let handshake = Handshake::from_bytes(buffer.as_ref())?;
        debug!("Received handshake {:?} from {}", handshake, self.addr);

        // verify that the peer sent the correct info hash which we expect
        if self.info_hash != handshake.info_hash {
            self.update_state(PeerState::Error).await;
            return Err(PeerError::Handshake(
                "received incorrect info hash from peer".to_string(),
            ));
        }

        // store the remote peer information
        trace!("Updating remote peer information for {}", handshake.peer_id);
        self.remote = Some(RemotePeer {
            peer_id: handshake.peer_id,
            supported_extensions: handshake.supported_extensions,
            extensions: Extensions::default(),
            client_name: None,
        });

        debug!("Handshake of peer {} has been validated", self);
        Ok(())
    }

    async fn update_remote_peer_choke_state(&mut self, state: ChokeState) {
        self.peer_choke_state = state;
        trace!(
            "Remote peer {} entered {} state",
            self.remote_id().unwrap(),
            self.peer_choke_state
        );
    }

    async fn update_state(&self, state: PeerState) {
        let mut mutex = self.state.write().await;
        *mutex = state;
        debug!("Updated peer {} state to {:?}", self, state);
    }

    fn update_extended_handshake(&mut self, handshake: ExtendedHandshake) {
        if let Some(remote) = self.remote.as_mut() {
            remote.extensions = handshake.m;
            remote.client_name = handshake.client;
            debug!("Updated peer {} with extended handshake information", self);
        } else {
            warn!("Received extended handshake before the initial handshake was completed");
        }
    }

    async fn send_handshake(&mut self) -> Result<()> {
        self.update_state(PeerState::Handshake).await;

        let handshake = Handshake::new(
            self.info_hash.clone(),
            self.client_id,
            ExtensionFlag::Extensions,
        );
        trace!("Trying to send handshake {:?}", handshake);
        match self
            .send_bytes(TryInto::<Vec<u8>>::try_into(handshake)?)
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

    async fn send_extended_handshake(&mut self) -> Result<()> {
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

    async fn send(&mut self, message: Message) -> Result<()> {
        let message_bytes = TryInto::<Vec<u8>>::try_into(message)?;
        self.send_bytes(message_bytes).await
    }

    async fn send_bytes<T: AsRef<[u8]>>(&mut self, message: T) -> Result<()> {
        timeout(
            Duration::from_secs(DEFAULT_CONNECTION_TIMEOUT_SECONDS),
            async {
                debug!(
                    "Sending a total of {} bytes to peer {}",
                    message.as_ref().len(),
                    self.addr
                );
                self.writer.write_all(message.as_ref()).await?;
                Ok::<(), PeerError>(())
            },
        )
        .await??;
        Ok(())
    }

    async fn send_keep_alive(&mut self) {
        let message = Message::KeepAlive;

        match TryInto::<Vec<u8>>::try_into(message) {
            Ok(bytes) => {
                if let Err(e) = self.send_bytes(bytes).await {
                    warn!(
                        "Failed to send keep alive to peer {}, {}",
                        self.remote.as_ref().map(|e| e.peer_id).unwrap(),
                        e
                    );
                }
            }
            Err(e) => warn!("Failed to parse keep alive message, {}", e),
        }
    }

    fn close(&mut self) {
        self.cancellation_token.cancel();
    }
}

impl Display for InnerPeer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.remote {
            Some(remote) => write!(f, "{}:{}", self.client_id, remote.peer_id),
            None => write!(f, "{}", self.client_id),
        }
    }
}

impl Drop for InnerPeer {
    fn drop(&mut self) {
        self.close();
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

    use crate::torrents::{Torrent, TorrentInfo, TorrentRequest};

    use super::*;

    #[test]
    fn test_peer_new_outbound() {
        init_logger();
        let torrent = read_test_file_to_bytes("debian-udp.torrent");
        let torrent_info = TorrentInfo::try_from(torrent.as_slice()).unwrap();
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
                torrent.handle(),
                torrent.command_sender(),
                torrent_info.info_hash.clone(),
                torrent.peer_id(),
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
                torrent.handle(),
                torrent.command_sender(),
                torrent_info.info_hash.clone(),
                torrent.peer_id(),
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
