use crate::torrent::peer::{Error, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use derive_more::Display;
use log::{debug, trace, warn};
use popcorn_fx_core::core::Handle;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::io::Cursor;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::RwLock;
use tokio::{select, time};
use tokio_util::sync::CancellationToken;

const MAX_PACKET_SIZE: usize = 8192;

/// The UTP socket identifier.
pub type UtpHandle = Handle;

/// The uTorrent transport protocol socket which can receive and send packets to multiple peers simultaneously.
#[derive(Debug, Display)]
#[display(fmt = "{}", inner)]
pub struct UtpSocket {
    inner: Arc<InnerUtpSocket>,
}

impl UtpSocket {
    /// Create a new UTP socket on the given address.
    ///
    /// The address should either be a [std::net::Ipv4Addr] or [std::net::Ipv6Addr] local machine address, and never an external address.
    /// If the socket port is no longer available, the UTP socket cannot be created and an error will be returned.
    pub async fn new(addr: SocketAddr, timeout: Duration, runtime: Arc<Runtime>) -> Result<Self> {
        let port = addr.port();
        let socket = UdpSocket::bind(addr).await?;
        let inner = Arc::new(InnerUtpSocket {
            handle: Default::default(),
            port,
            socket,
            addr,
            connections: Default::default(),
            pending_packets: Default::default(),
            timeout,
            cancellation_token: Default::default(),
            runtime,
        });

        let inner_main_loop = inner.clone();
        inner
            .runtime
            .spawn(async move { inner_main_loop.start().await });

        Ok(Self { inner })
    }

    /// Get the local socket address of the uTP connection.
    /// It returns the socket address on which the uTP socket is bound.
    pub fn addr(&self) -> SocketAddr {
        self.inner.addr
    }

    /// Try to connect to the given utp target address.
    /// This will establish a new stream with the given address that can be used to send and receive data.
    ///
    /// It returns an error if the connection for the utp socket couldn't be established.
    pub async fn connect(&self, addr: SocketAddr) -> Result<UtpStream> {
        let mut connections = self.inner.connections.write().await;
        let connection_id: UtpConnId;

        // generate a unique connection id
        loop {
            let id = UtpConnId::new();
            if !connections.contains_key(&id) {
                connection_id = id;
                break;
            }
        }

        let (message_sender, message_receiver) = unbounded_channel();
        let stream = UtpStream::new(
            connection_id,
            addr,
            self.inner.clone(),
            message_receiver,
            self.inner.runtime.clone(),
        )
        .await?;

        // store the connection
        debug!("Utp socket {} connected with {}", self, addr);
        connections.insert(connection_id, message_sender);

        Ok(stream)
    }
}

impl Drop for UtpSocket {
    fn drop(&mut self) {
        self.inner.cancellation_token.cancel();
    }
}

/// A uTorrent transport protocol connection stream.
/// This stream allows to read and write to a specific uTP connection.
#[derive(Debug, Display)]
#[display(fmt = "{}", inner)]
pub struct UtpStream {
    inner: Arc<InnerUtpStream>,
}

impl UtpStream {
    /// Try to create a new uTP stream for the given address.
    /// This will initiate the SYN process with the remote socket address.
    async fn new(
        key: UtpConnId,
        addr: SocketAddr,
        socket: Arc<InnerUtpSocket>,
        message_receiver: UnboundedReceiver<Message>,
        runtime: Arc<Runtime>,
    ) -> Result<Self> {
        let inner = Arc::new(InnerUtpStream {
            key,
            socket,
            addr,
            state: RwLock::new(UtpStreamState::Initializing),
            sequence_number: RwLock::new(1),
            cancellation_token: Default::default(),
        });

        let inner_main_loop = inner.clone();
        runtime.spawn(async move {
            inner_main_loop.start(message_receiver).await;
        });

        inner.send_syn().await?;
        Ok(Self { inner })
    }

    /// Close the uTP stream.
    pub async fn close(&self) {
        self.inner.close().await
    }
}

impl Drop for UtpStream {
    fn drop(&mut self) {
        self.inner.cancellation_token.cancel();
    }
}

#[derive(Debug, Display)]
#[display(fmt = "{} ({})", socket, addr)]
struct InnerUtpStream {
    /// The unique key of the utp stream
    key: UtpConnId,
    /// The underlying shared uTP socket
    socket: Arc<InnerUtpSocket>,
    /// The remote connected address
    addr: SocketAddr,
    /// The state of the stream connection
    state: RwLock<UtpStreamState>,
    /// The packet sequence number
    sequence_number: RwLock<u16>,
    cancellation_token: CancellationToken,
}

impl InnerUtpStream {
    /// Starts the main loop of the utp stream for processing messages.
    async fn start(&self, mut message_receiver: UnboundedReceiver<Message>) {
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                message = message_receiver.recv() => {
                    if let Some(message) = message {
                        self.handle_received_message(message).await;
                    } else {
                        break;
                    }
                }
            }
        }

        debug!("Utp stream {} main loop ended", self);
    }

    async fn handle_received_message(&self, message: Message) {}

    async fn close(&self) {
        self.socket.close_connection(self.key).await;
        self.cancellation_token.cancel();
    }

    async fn send_syn(&self) -> Result<()> {
        let syn_message = Message::Connect(self.key.recv_id);
        self.send(syn_message).await?;
        self.update_state(UtpStreamState::SynSent).await;
        Ok(())
    }

    async fn send(&self, message: Message) -> Result<()> {
        trace!("Utp stream {} is sending {:?}", self, message);
        self.socket.send_message(message, self.addr).await?;
        self.increase_sequence().await;
        Ok(())
    }

    async fn update_state(&self, state: UtpStreamState) {
        let mut mutex = self.state.write().await;
        if *mutex == state {
            return;
        }

        *mutex = state;
        debug!("Utp stream {} state changed to {:?}", self, state);
    }

    async fn increase_sequence(&self) {
        *self.sequence_number.write().await += 1;
    }
}

/// The state of an uTP stream connection
#[derive(Debug, Copy, Clone, PartialEq)]
enum UtpStreamState {
    Initializing,
    SynSent,
    SynRecv,
    Connected,
}

#[derive(Debug, Display)]
#[display(fmt = "{}:{}", handle, port)]
struct InnerUtpSocket {
    /// The unique identifier of the utp socket
    handle: UtpHandle,
    /// The port on which the socket has been bound
    port: u16,
    /// The underlying socket used by the utp socket
    socket: UdpSocket,
    /// The underlying socket address of the utp socket
    addr: SocketAddr,
    /// The established connections for the utp socket
    connections: RwLock<HashMap<UtpConnId, UnboundedSender<Message>>>,
    /// The packets which are still pending and didn't receive an ACK yet
    pending_packets: RwLock<Vec<Packet>>,
    /// The connection timeout
    timeout: Duration,
    /// The termination cancellation token
    cancellation_token: CancellationToken,
    /// The shared runtime of the socket
    runtime: Arc<Runtime>,
}

impl InnerUtpSocket {
    /// Start the main loop of the utp socket.
    async fn start(&self) {
        loop {
            // a packet header should always exist out of at least 20 bytes
            let mut packet_header_bytes = vec![0u8; MAX_PACKET_SIZE];

            select! {
                _ = self.cancellation_token.cancelled() => break,
                result = self.socket.recv(&mut packet_header_bytes) => {
                    match result {
                        Ok(bytes_read) => {
                            if bytes_read > 0 {
                                self.handle_packet_bytes(&packet_header_bytes[..bytes_read]).await;
                            } else {
                                debug!("Utp socket {} reader received EOF", self);
                                break;
                            }
                        },
                        Err(e) => {
                            warn!("Utp socket {} reader failed to receive packet, {}", self, e);
                            break;
                        }
                    }
                }
            }
        }

        debug!("Utp socket {} main loop ended", self);
    }

    async fn handle_packet_bytes(&self, packet_bytes: &[u8]) {
        trace!("Utp socket {} received {} bytes", self, packet_bytes.len());
        match Packet::try_from(packet_bytes) {
            Ok(packet) => {
                debug!("Utp socket {} received packet {:?}", self, packet);
                let connections = self.connections.read().await;

                // try to find the relevant connection of the received packet
                if let Some((_, connection_message_sender)) = connections
                    .iter()
                    .find(|(key, _)| key.recv_id == packet.connection_id)
                {}
            }
            Err(e) => warn!("Utp socket {} failed to parse packet, {}", self, e),
        }
    }

    /// Try to send the given message to the remote peer.
    async fn send_message(&self, message: Message, addr: SocketAddr) -> Result<()> {
        let timestamp_microseconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Error::Io(format!("invalid system time, {}", e)))?
            .as_micros() as u32;
        let mut packet: Packet = message.into();

        packet.timestamp_microseconds = timestamp_microseconds;
        self.send_packet(packet, addr).await
    }

    /// Try to send the given packet to the remote peer.
    async fn send_packet(&self, packet: Packet, addr: SocketAddr) -> Result<()> {
        let bytes: Vec<u8> = packet.try_into()?;
        let bytes_len = bytes.len();

        select! {
            _ = time::sleep(self.timeout) => Err(Error::Io(format!("connection timed out after {}s", self.timeout.as_secs()))),
            result = self.socket.send_to(&bytes, addr) => {
                match result {
                    Ok(_) => {
                        trace!("Utp socket {} sent {} bytes to {}", self, bytes_len, addr);
                        Ok(())
                    },
                    Err(e) => Err(Error::Io(e.to_string())),
                }
            }
        }
    }

    /// Close the given connection.
    async fn close_connection(&self, key: UtpConnId) {
        self.connections.write().await.remove(&key);
    }
}

/// The type of an UTP packet.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum StateType {
    /// Regular packet type
    Data = 0,
    /// Finalize the connection
    Fin = 1,
    /// State packet
    State = 2,
    /// Terminate the connection forcefully
    Reset = 3,
    /// Initiate a connection
    Syn = 4,
}

impl TryFrom<u8> for StateType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(StateType::Data),
            1 => Ok(StateType::Fin),
            2 => Ok(StateType::State),
            3 => Ok(StateType::Reset),
            4 => Ok(StateType::Syn),
            _ => Err(Error::UnsupportedMessage(value)),
        }
    }
}

/// A connection identifier of an utp stream.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct UtpConnId {
    pub recv_id: u16,
    pub send_id: u16,
}

impl UtpConnId {
    pub fn new() -> Self {
        let mut thread_ng = thread_rng();
        let connection_id_recv: u16 = thread_ng.gen();
        let connection_id_send: u16 = connection_id_recv + 1;

        Self {
            recv_id: connection_id_recv,
            send_id: connection_id_send,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Message {
    /// Connect to the utp peer with the connection id
    Connect(u16),
    /// Acknowledgment of a new connection
    ConnectAccept,
    /// Close the connection
    Close(u16),
}

impl Into<Packet> for Message {
    fn into(self) -> Packet {
        match self {
            Message::Connect(connection_id) => Packet {
                state_type: StateType::Syn,
                extension: 0,
                connection_id,
                timestamp_microseconds: 0,
                timestamp_difference_microseconds: 0,
                window_size: 0,
                sequence_number: 1,
                acknowledge_number: 0,
            },
            Message::Close(connection_id) => Packet {
                state_type: StateType::Reset,
                extension: 0,
                connection_id,
                timestamp_microseconds: 0,
                timestamp_difference_microseconds: 0,
                window_size: 0,
                sequence_number: 0,
                acknowledge_number: 0,
            },
            _ => todo!("{:?}", self),
        }
    }
}

impl TryFrom<Packet> for Message {
    type Error = Error;

    fn try_from(value: Packet) -> Result<Self> {
        trace!("Trying to parse message type {:?}", value.state_type);
        match value.state_type {
            StateType::Reset => Ok(Message::Close(value.connection_id)),
            _ => Err(Error::UnsupportedMessage(value.state_type as u8)),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Packet {
    /// The packet type
    pub state_type: StateType,
    /// The extensions number, 0 if no extension
    pub extension: u8,
    /// Unique connection identifier of the stream to which the packet belongs
    pub connection_id: u16,
    /// The timestamp of when this packet was sent
    pub timestamp_microseconds: u32,
    /// The difference between the local time and the timestamp in the last received packet
    pub timestamp_difference_microseconds: u32,
    /// The number of bytes in-flight that have not been acked yet
    pub window_size: u32,
    /// The packet sequence number
    pub sequence_number: u16,
    /// The sequence number of the last received packet
    pub acknowledge_number: u16,
}

impl TryInto<Vec<u8>> for Packet {
    type Error = Error;

    fn try_into(self) -> Result<Vec<u8>> {
        trace!("Parsing packet {:?}", self);
        let mut buffer = vec![0u8; 2];

        // write the type & version into the first byte
        buffer[0] = (self.state_type as u8) << 4 | 1;
        // write the extension number in the next byte
        buffer[1] = self.extension;
        // write the connection number
        buffer.write_u16::<BigEndian>(self.connection_id)?;
        // write the current timestamp
        buffer.write_u32::<BigEndian>(self.timestamp_microseconds)?;
        // write the timestamp difference
        buffer.write_u32::<BigEndian>(self.timestamp_difference_microseconds)?;
        // write the window size
        buffer.write_u32::<BigEndian>(self.window_size)?;
        // write the sequence number
        buffer.write_u16::<BigEndian>(self.sequence_number)?;
        // write the acknowledgment number
        buffer.write_u16::<BigEndian>(self.acknowledge_number)?;

        Ok(buffer)
    }
}

impl TryFrom<&[u8]> for Packet {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(value);

        // read the type & version from the first byte
        let byte = cursor.read_u8()?;
        let state_type = StateType::try_from(byte >> 4)?;
        let version = byte & 0x0f;
        // read the extension from the second byte
        let extension = cursor.read_u8()?;
        let connection_id = cursor.read_u16::<BigEndian>()?;
        let timestamp_microseconds = cursor.read_u32::<BigEndian>()?;
        let timestamp_difference_microseconds = cursor.read_u32::<BigEndian>()?;
        let window_size = cursor.read_u32::<BigEndian>()?;
        let sequence_number = cursor.read_u16::<BigEndian>()?;
        let acknowledge_number = cursor.read_u16::<BigEndian>()?;

        // check the package version
        if version != 1 {
            return Err(Error::UnsupportedVersion(version as u32));
        }

        Ok(Self {
            state_type,
            extension,
            connection_id,
            timestamp_microseconds,
            timestamp_difference_microseconds,
            window_size,
            sequence_number,
            acknowledge_number,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use popcorn_fx_core::init_logger;

    #[test]
    fn test_packet_try_from() {
        init_logger!();
        let timestamp_microseconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u32;
        let packet = Packet {
            state_type: StateType::Syn,
            extension: 0,
            connection_id: 12345,
            timestamp_microseconds,
            timestamp_difference_microseconds: 0,
            window_size: 0,
            sequence_number: 0,
            acknowledge_number: 0,
        };

        let bytes: Vec<u8> = packet
            .clone()
            .try_into()
            .expect("expected the packet to have been serialized");
        assert_eq!(
            20,
            bytes.len(),
            "expected the header to have a length of 20 bytes"
        );

        let result = Packet::try_from(bytes.as_slice())
            .expect("expected the packet to have been deserialized");

        assert_eq!(packet, result);
    }
}
