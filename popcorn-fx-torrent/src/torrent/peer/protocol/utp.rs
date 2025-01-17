use crate::torrent::peer::{Error, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use derive_more::Display;
use fx_handle::Handle;
use itertools::Itertools;
use log::{debug, error, trace, warn};
use rand::{random, thread_rng, Rng};
use std::collections::{HashMap, VecDeque};
use std::io::{Cursor, Read, Write};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::{Mutex, RwLock};
use tokio::time::interval;
use tokio::{select, time};
use tokio_util::sync::CancellationToken;

/// The maximum size of a single uTP packet (= max UDP size).
const MAX_PACKET_SIZE: usize = 65_535;
/// The maximum size of a payload in a single uTP packet (= max UDP size - max uTP header size).
const MAX_PACKET_PAYLOAD_SIZE: usize = MAX_PACKET_SIZE - 26;
/// The maximum amount out-of-order packets which can stored in memory.
const MAX_UNACKED_PACKETS: usize = 128;

/// The UTP socket identifier.
pub type UtpHandle = Handle;

/// The packet connection identifier.
type ConnectionId = u16;
/// The packet sequence number.
type SequenceNumber = u16;

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
        let (incoming_stream_sender, incoming_stream_receiver) = unbounded_channel();
        let inner = Arc::new(InnerUtpSocket {
            handle: Default::default(),
            port,
            socket,
            addr,
            connections: Default::default(),
            pending_packets: Default::default(),
            incoming_stream_sender,
            incoming_stream_receiver: Mutex::new(incoming_stream_receiver),
            timeout,
            cancellation_token: Default::default(),
            runtime,
        });

        let inner_main_loop = inner.clone();
        inner
            .runtime
            .spawn(async move { inner_main_loop.start(&inner_main_loop).await });

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
        let stream = UtpStream::new_outgoing(
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

    /// Try to receive the next incoming uTP stream of the socket.
    /// These streams can only be received by one caller at a time.
    ///
    /// It returns the next uTP stream if available, else [None].
    pub async fn recv(&self) -> Option<UtpStream> {
        let mut receiver = self.inner.incoming_stream_receiver.lock().await;
        select! {
            _ = self.inner.cancellation_token.cancelled() => None,
            stream = receiver.recv() => stream,
        }
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
    /// Try to create a new outgoing uTP stream for the given address.
    /// This will initiate the SYN process with the remote socket address.
    async fn new_outgoing(
        key: UtpConnId,
        addr: SocketAddr,
        socket: Arc<InnerUtpSocket>,
        message_receiver: UnboundedReceiver<SocketMessage>,
        runtime: Arc<Runtime>,
    ) -> Result<Self> {
        let seq_number = 1;
        let inner = Self::new(
            key,
            addr,
            socket,
            UtpStreamState::Initializing,
            seq_number,
            0,
            seq_number - 1,
        );

        let inner_main_loop = inner.clone();
        runtime.spawn(async move {
            inner_main_loop.start(message_receiver).await;
        });

        inner.send_syn().await?;
        Ok(Self { inner })
    }

    /// Try to accept a new incoming uTP stream for the given address.
    /// This will finish the SYN process with the remote socket address.
    async fn new_incoming(
        key: UtpConnId,
        addr: SocketAddr,
        socket: Arc<InnerUtpSocket>,
        ack_number: u16,
        message_receiver: UnboundedReceiver<SocketMessage>,
        runtime: Arc<Runtime>,
    ) -> Result<Self> {
        let inner = Self::new(
            key,
            addr,
            socket,
            UtpStreamState::SynRecv,
            random(),
            ack_number,
            ack_number - 1,
        );

        let inner_main_loop = inner.clone();
        runtime.spawn(async move {
            inner_main_loop.start(message_receiver).await;
        });

        inner.send_state().await?;
        inner.update_state(UtpStreamState::Connected).await;
        Ok(Self { inner })
    }

    /// Get the remote socket address of the uTP stream.
    pub fn addr(&self) -> SocketAddr {
        self.inner.addr
    }

    /// Check if the uTP stream is closed.
    /// In this state, the stream is no longer able to send or receive any packets.
    pub async fn is_closed(&self) -> bool {
        *self.inner.state.read().await == UtpStreamState::Closed
    }

    /// Get the current latency of the stream in microseconds.
    pub async fn latency(&self) -> u32 {
        *self.inner.timestamp_difference_microseconds.lock().await
    }

    /// Send the given bytes to the remote uTP stream peer.
    pub async fn send(&self, bytes: &[u8]) -> Result<()> {
        self.inner.send_data(bytes).await
    }

    /// Receive bytes of the remote peer of the uTP stream.
    pub async fn recv(&self) -> Option<Vec<u8>> {
        self.inner.message_receiver.lock().await.recv().await
    }

    /// Close the uTP stream.
    pub async fn close(&self) -> Result<()> {
        self.inner.close().await
    }

    /// Get the current state of the uTP stream.
    async fn state(&self) -> UtpStreamState {
        *self.inner.state.read().await
    }

    fn new(
        key: UtpConnId,
        addr: SocketAddr,
        socket: Arc<InnerUtpSocket>,
        state: UtpStreamState,
        seq_number: u16,
        ack_number: u16,
        last_ack_number: u16,
    ) -> Arc<InnerUtpStream> {
        let (sender, receiver) = unbounded_channel();
        Arc::new(InnerUtpStream {
            key,
            socket,
            addr,
            state: RwLock::new(state),
            seq_number: Mutex::new(seq_number),
            ack_number: Mutex::new(ack_number),
            last_ack_number: Mutex::new(last_ack_number),
            pending_incoming_packets: Default::default(),
            pending_outgoing_packets: Default::default(),
            timestamp_difference_microseconds: Default::default(),
            message_sender: sender,
            message_receiver: Mutex::new(receiver),
            cancellation_token: Default::default(),
        })
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
    /// The last sent packet sequence number to the remote peer.
    seq_number: Mutex<SequenceNumber>,
    /// The last packet sequence number that has been acknowledged to the remote peer. (outgoing)
    ack_number: Mutex<SequenceNumber>,
    /// Our last packet sequence number that was acknowledged by the remote peer. (incoming)
    last_ack_number: Mutex<SequenceNumber>,
    /// The pending incoming packets which have been received out of order from the remote peer.
    pending_incoming_packets: Mutex<HashMap<SequenceNumber, Message>>,
    /// The pending packets which have not been acked by the remote peer.
    pending_outgoing_packets: RwLock<Vec<PendingPacket>>,
    /// The delay of packets between the sender and receiver of packets.
    timestamp_difference_microseconds: Mutex<u32>,
    /// The sender for received data from the remote peer.
    message_sender: UnboundedSender<Vec<u8>>,
    /// The receiver for received data from the remote peer.
    message_receiver: Mutex<UnboundedReceiver<Vec<u8>>>,
    /// The cancellation token of the stream
    cancellation_token: CancellationToken,
}

impl InnerUtpStream {
    /// Starts the main loop of the utp stream for processing messages.
    async fn start(&self, mut message_receiver: UnboundedReceiver<SocketMessage>) {
        let mut resend_interval = interval(Duration::from_millis(750));
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                message = message_receiver.recv() => {
                    if let Some(message) = message {
                        self.handle_received_message(message).await;
                    } else {
                        debug!("Utp stream {} socket has been closed", self);
                        break;
                    }
                }
                _ = resend_interval.tick() => self.resend_timeout_packets().await,
            }
        }

        let _ = self.close().await;
        debug!("Utp stream {} main loop ended", self);
    }

    async fn handle_received_message(&self, socket_message: SocketMessage) {
        // check if the message is valid
        if !self.assert_message(&socket_message.message) {
            return;
        }

        // calculate the latency of the uTP stream connection from the packet
        let timestamp = now_as_micros();
        let timestamp_difference =
            timestamp.saturating_sub(socket_message.packet.timestamp_microseconds);
        *self.timestamp_difference_microseconds.lock().await = timestamp_difference;

        // process the last ack number of the remote peer
        self.handle_remote_acknowledgment(
            socket_message.packet.sequence_number,
            socket_message.packet.acknowledge_number,
        )
        .await;
        // process the extensions of the packet
        self.handle_extensions(&socket_message.packet).await;

        // check if we've already seen the packet, due to a resend delay
        let mut ack_number = self.ack_number.lock().await;
        let remote_sequence_number = socket_message.packet.sequence_number;
        let current_ack_number = *ack_number;
        if current_ack_number > remote_sequence_number {
            trace!(
                "Utp stream {} has already seen packet {}",
                self,
                remote_sequence_number
            );
            return;
        }

        // calculate the difference between the received sequence and our last inbound ack number
        let sequence_diff = current_ack_number.saturating_sub(remote_sequence_number - 1);
        let mut pending_incoming_packets = self.pending_incoming_packets.lock().await;
        // store the out-of-order ahead packet in the buffer is allowed
        if sequence_diff <= MAX_UNACKED_PACKETS as u16 {
            // buffer the incoming out-of-order packet
            pending_incoming_packets.insert(remote_sequence_number, socket_message.message);
        }
        // if the packet is in order, we process it with any other pending incoming packets
        if sequence_diff == 0 {
            loop {
                let next_seq_number = *ack_number + 1;
                if let Some(message) = pending_incoming_packets.remove(&next_seq_number) {
                    // process the incoming message in-order
                    self.process_incoming_message(message).await;
                    // increase the processed packet ack number
                    *ack_number += 1;
                } else {
                    // we don't have the next sequence packet available, stop processing messages
                    break;
                }
            }
        }
    }

    /// Handle the extensions within the header of the packet.
    async fn handle_extensions(&self, packet: &Packet) {
        match &packet.extension {
            Extension::SelectiveAck => {
                warn!("Utp stream {} selective acks are not yet supported", self);
            }
            _ => {}
        }
    }

    /// Handle the last acknowledgement number of a remote peer.
    /// This will process any outgoing pending packets up to the given `ack_number`.
    async fn handle_remote_acknowledgment(
        &self,
        remote_seq_number: SequenceNumber,
        remote_ack_number: SequenceNumber,
    ) {
        let is_state_syn_send = *self.state.read().await == UtpStreamState::SynSent;
        if is_state_syn_send && remote_ack_number == 1 {
            debug!("Utp stream {} connection established", self);
            // set the index of the remote ack number to be inline with the incoming sequence number determined by the remote peer
            *self.ack_number.lock().await = remote_seq_number - 1;
            self.update_state(UtpStreamState::Connected).await;
        }

        {
            trace!(
                "Utp stream {} is processing acknowledge number {}",
                self,
                remote_ack_number
            );
            // try to find the pending packet belonging to the ack number
            let mut pending_packets = self.pending_outgoing_packets.write().await;
            let mut last_ack_number = self.last_ack_number.lock().await;
            // as the ack number might be the highest sequence number,
            // we need to acknowledge all pending messages up to the given ack number
            for ack_number in *last_ack_number + 1..remote_ack_number {
                if let Some(packet_index) = pending_packets
                    .iter()
                    .position(|e| e.packet.sequence_number == ack_number)
                {
                    // if the packet is found, remove it from the pending state
                    pending_packets.remove(packet_index);
                    *last_ack_number = ack_number;
                } else {
                    trace!(
                        "Utp stream {} couldn't find pending packet for ack number {}",
                        self,
                        ack_number
                    );
                }
            }
        }
    }

    async fn handle_close_message(&self) {
        self.cancellation_token.cancel();
        self.update_state(UtpStreamState::Closed).await;
    }

    /// Handle a received data payload from the remote peer.
    /// This will acknowledge the packet once it has been processed.
    async fn handle_received_payload(&self, bytes: Vec<u8>) {
        if let Err(e) = self.message_sender.send(bytes) {
            warn!("Utp stream {} failed to send message data, {}", self, e);
            return;
        }
    }

    /// Process an in-order incoming uTP message.
    async fn process_incoming_message(&self, message: Message) {
        trace!("Upt stream {} is processing message {:?}", self, message);
        match message {
            Message::Connect(_) => {} // this is never actually received as it's handled by the socket
            Message::State(_, seq_number, ack_number) => {
                // check if we have acked the latest sequence_number of the remote peer
                // if our last ack is smaller, a potential packet might have been lost
                let last_ack = *self.last_ack_number.lock().await;
                if last_ack < seq_number {
                    debug!(
                        "Upt stream {} has different state info than remote, re-syncing state",
                        self
                    );
                    let _ = self.send_state().await;
                }

                self.handle_remote_acknowledgment(seq_number, ack_number)
                    .await;
            }
            Message::Data(_, payload) => {
                self.handle_received_payload(payload).await;
            }
            Message::Terminate(_) => {
                self.handle_close_message().await;
            }
            Message::Close(_) => {
                self.handle_close_message().await;
            }
        }
    }

    fn assert_message(&self, message: &Message) -> bool {
        let connection_id: ConnectionId;

        match message {
            Message::State(id, _, _) => connection_id = *id,
            Message::Data(id, _) => connection_id = *id,
            Message::Close(id) => connection_id = *id,
            Message::Terminate(id) => connection_id = *id,
            _ => return false,
        }

        if connection_id != self.key.recv_id {
            debug!(
                "Utp stream {} received invalid message id {}",
                self, connection_id
            );
            return false;
        }

        true
    }

    /// Send the initial syn message to the remote peer.
    async fn send_syn(&self) -> Result<()> {
        let syn_message = Message::Connect(self.key.recv_id);
        self.send(syn_message).await?;
        self.increase_sequence().await;
        self.update_state(UtpStreamState::SynSent).await;
        Ok(())
    }

    /// Send the current uTP state info to the remote peer.
    async fn send_state(&self) -> Result<()> {
        let seq_number = *self.seq_number.lock().await;
        let ack_number = *self.ack_number.lock().await;
        let message = Message::State(self.key.send_id, seq_number, ack_number);
        self.send(message).await?;
        Ok(())
    }

    async fn send_data(&self, bytes: &[u8]) -> Result<()> {
        // check the length of the data
        if bytes.len() > MAX_PACKET_PAYLOAD_SIZE {
            return Err(Error::Io(format!(
                "data length is exceeding the maximum of {}",
                MAX_PACKET_PAYLOAD_SIZE
            )));
        }

        let message = Message::Data(self.key.send_id, bytes.to_vec());
        self.send(message).await?;
        self.increase_sequence().await;
        Ok(())
    }

    async fn send(&self, message: Message) -> Result<()> {
        trace!("Utp stream {} is sending {:?}", self, message);
        let addr = self.addr;
        let sequence_number = *self.seq_number.lock().await;
        let acknowledge_number = *self.ack_number.lock().await;
        let timestamp_difference = *self.timestamp_difference_microseconds.lock().await;
        let window_size = self.window_size().await;

        // send the message
        let start_time = Instant::now();
        let pending_packet = self
            .socket
            .send_message(
                message,
                addr,
                sequence_number,
                acknowledge_number,
                timestamp_difference,
                window_size,
            )
            .await?;
        let elapsed = start_time.elapsed();
        trace!(
            "Utp stream {} sent {:?} in {}.{:03}ms",
            self,
            pending_packet.packet,
            elapsed.as_millis(),
            elapsed.subsec_micros() % 1000
        );

        // store the pending packet
        self.pending_outgoing_packets
            .write()
            .await
            .push(pending_packet);
        Ok(())
    }

    /// Resend all packets which have not yet been acked and have timed out.
    async fn resend_timeout_packets(&self) {
        let timeout_threshold = *self.timestamp_difference_microseconds.lock().await;
        if timeout_threshold == 0 {
            return;
        }

        let timestamp_now = now_as_micros();
        let window_size = self.window_size().await;
        let timestamp_difference_microseconds =
            *self.timestamp_difference_microseconds.lock().await;
        let mut pending_packets = self.pending_outgoing_packets.write().await;
        for pending_packet in pending_packets
            .iter_mut()
            .filter(|e| timestamp_now - e.packet.timestamp_microseconds > timeout_threshold)
            .take(10)
        {
            // update the packet with the latest info
            pending_packet.packet.window_size = window_size;
            pending_packet.packet.acknowledge_number = *self.ack_number.lock().await;
            pending_packet.packet.timestamp_difference_microseconds =
                timestamp_difference_microseconds;

            trace!(
                "Utp stream {} is resending packet {:?}",
                self,
                pending_packet
            );
            match self
                .socket
                .send_packet(pending_packet.packet.clone(), self.addr)
                .await
            {
                Ok(_) => {
                    pending_packet.increase_resend();
                }
                Err(e) => {
                    debug!("Utp stream {} failed to resend packet, {}", self, e);
                    pending_packet.increase_failures();
                }
            }
        }
    }

    /// Get the current window size of all in-flight stream messages that have not yet been acked.
    async fn window_size(&self) -> u32 {
        self.pending_outgoing_packets
            .read()
            .await
            .iter()
            .map(|e| e.packet_size() as u32)
            .sum()
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
        *self.seq_number.lock().await += 1;
    }

    /// Try to gracefully close the connection with the remote peer.
    async fn close(&self) -> Result<()> {
        if *self.state.read().await == UtpStreamState::Closed {
            return Ok(());
        }

        let result = self.send(Message::Close(self.key.send_id)).await;
        self.socket.close_connection(self.key).await;
        // update the state to close before cancelling the context
        // as the main loop might otherwise execute the close twice
        self.update_state(UtpStreamState::Closed).await;
        self.cancellation_token.cancel();
        result
    }
}

/// The state of an uTP stream connection.
#[derive(Debug, Copy, Clone, PartialEq)]
enum UtpStreamState {
    /// The stream is being initialized and no state is known at the moment
    Initializing,
    /// The stream has sent the SYN packet
    SynSent,
    /// The stream has received the SYN packet
    SynRecv,
    /// The stream has successfully connected with the remote uTP socket
    Connected,
    /// The stream has been closed
    Closed,
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
    connections: RwLock<HashMap<UtpConnId, UnboundedSender<SocketMessage>>>,
    /// The packets which are still pending and didn't receive an ACK yet
    pending_packets: RwLock<Vec<PendingPacket>>,
    /// The sender of new incoming utp streams.
    incoming_stream_sender: UnboundedSender<UtpStream>,
    /// The receiver of new incoming utp streams.
    incoming_stream_receiver: Mutex<UnboundedReceiver<UtpStream>>,
    /// The connection timeout
    timeout: Duration,
    /// The termination cancellation token
    cancellation_token: CancellationToken,
    /// The shared runtime of the socket
    runtime: Arc<Runtime>,
}

impl InnerUtpSocket {
    /// Start the main loop of the utp socket.
    async fn start(&self, socket: &Arc<InnerUtpSocket>) {
        loop {
            // a packet header should always exist out of at least 20 bytes
            let mut packet_header_bytes = vec![0u8; MAX_PACKET_SIZE];

            select! {
                _ = self.cancellation_token.cancelled() => break,
                result = self.socket.recv_from(&mut packet_header_bytes) => {
                    match result {
                        Ok((bytes_read, addr)) => {
                            if bytes_read > 0 {
                                self.handle_packet_bytes(&packet_header_bytes[..bytes_read], addr, socket).await;
                            } else {
                                debug!("Utp socket {} reader received EOF", self);
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

    /// Handle a received packet payload from the socket.
    async fn handle_packet_bytes(
        &self,
        packet_bytes: &[u8],
        addr: SocketAddr,
        socket: &Arc<InnerUtpSocket>,
    ) {
        trace!("Utp socket {} received {} bytes", self, packet_bytes.len());
        match Packet::try_from(packet_bytes) {
            Ok(packet) => {
                trace!("Utp socket {} received packet {:?}", self, packet);
                // check if the packet is a new incoming connection
                if packet.state_type == StateType::Syn {
                    self.handle_incoming_connection(packet, addr, socket).await;
                } else {
                    trace!(
                        "Utp socket {} is parsing incoming message {:?}",
                        self,
                        packet
                    );
                    match Message::try_from(&packet) {
                        Ok(message) => self.handle_received_message(message, packet).await,
                        Err(e) => warn!("Utp socket {} failed to parse packet, {}", self, e),
                    }
                }
            }
            Err(e) => warn!("Utp socket {} failed to parse packet, {}", self, e),
        }
    }

    /// Handle a received parsed message from the socket.
    async fn handle_received_message(&self, message: Message, packet: Packet) {
        let mut key_to_remove: Option<UtpConnId> = None;

        {
            // try to find the relevant connection of the received packet
            let connections = self.connections.read().await;
            let connection_id = packet.connection_id;
            if let Some((key, connection_message_sender)) = connections
                .iter()
                .find(|(key, _)| key.recv_id == connection_id)
            {
                if let Err(e) = connection_message_sender.send(SocketMessage { message, packet }) {
                    trace!(
                        "Utp socket {} connection {} has been closed, {}",
                        self,
                        connection_id,
                        e
                    );
                    key_to_remove = Some(key.clone());
                }
            } else {
                trace!(
                    "Utp socket {} received packet with unknown connection id {}",
                    self,
                    packet.connection_id
                );
            }
        }

        if let Some(key) = key_to_remove {
            debug!("Utp socket {} dropping connection {:?}", self, key);
            self.connections.write().await.remove(&key);
        }
    }

    /// Try to handle a new incoming uTP connection.
    async fn handle_incoming_connection(
        &self,
        packet: Packet,
        addr: SocketAddr,
        socket: &Arc<InnerUtpSocket>,
    ) {
        let key = UtpConnId {
            recv_id: packet.connection_id + 1,
            send_id: packet.connection_id,
        };
        let ack_number: u16 = packet.sequence_number;
        let (message_sender, message_receiver) = unbounded_channel();

        match UtpStream::new_incoming(
            key,
            addr,
            socket.clone(),
            ack_number,
            message_receiver,
            self.runtime.clone(),
        )
        .await
        {
            Ok(stream) => {
                debug!("Utp socket {} established connection with {}", self, addr);
                {
                    let mut connections = self.connections.write().await;
                    connections.insert(key, message_sender);
                }

                let _ = self.incoming_stream_sender.send(stream);
            }
            Err(e) => debug!(
                "Utp socket {} failed to accept incoming connection from {}, {}",
                self, addr, e
            ),
        }
    }

    /// Try to send the given message to the remote peer.
    ///
    /// # Arguments
    ///
    /// * `message` - The message that needs to be sent.
    /// * `stream` - The stream that is sending the message.
    async fn send_message(
        &self,
        message: Message,
        addr: SocketAddr,
        sequence_number: SequenceNumber,
        acknowledge_number: SequenceNumber,
        timestamp_difference: u32,
        window_size: u32,
    ) -> Result<PendingPacket> {
        trace!(
            "Utp stream {} is parsing outgoing message {:?}",
            self,
            message
        );
        let packet = Packet::from_message(
            message,
            sequence_number,
            acknowledge_number,
            timestamp_difference,
            window_size,
        );
        self.send_packet(packet, addr).await
    }

    /// Try to send the given packet to the remote peer.
    async fn send_packet(&self, packet: Packet, addr: SocketAddr) -> Result<PendingPacket> {
        let mut packet = packet;
        packet.timestamp_microseconds = now_as_micros();

        // convert the packet into the payload bytes
        let bytes: Vec<u8> = packet.as_bytes()?;
        let bytes_len = bytes.len();

        select! {
            _ = time::sleep(self.timeout) => Err(Error::Io(format!("connection timed out after {}s", self.timeout.as_secs()))),
            result = self.socket.send_to(&bytes, addr) => {
                match result {
                    Ok(_) => {
                        trace!("Utp socket {} sent {} bytes to {}", self, bytes_len, addr);
                        Ok(PendingPacket::new(packet))
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

impl From<&Message> for StateType {
    fn from(value: &Message) -> Self {
        match value {
            Message::Connect(_) => StateType::Syn,
            Message::State(_, _, _) => StateType::State,
            Message::Data(_, _) => StateType::Data,
            Message::Terminate(_) => StateType::Reset,
            Message::Close(_) => StateType::Fin,
        }
    }
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

#[derive(Debug)]
struct SocketMessage {
    /// The parsed message of the uTP protocol
    message: Message,
    /// The original uTP packet
    packet: Packet,
}

/// A parsed uTP message.
#[derive(Debug, Clone, PartialEq)]
enum Message {
    /// Connect to the utp peer with the connection id
    Connect(ConnectionId),
    /// The latest known state of an uTP peer with `sequence_number` & `acknowledge_number`.
    State(ConnectionId, SequenceNumber, SequenceNumber),
    /// Message containing data information
    Data(ConnectionId, Vec<u8>),
    /// Terminate the connection forcefully.
    Terminate(ConnectionId),
    /// Close the connection
    Close(ConnectionId),
}

impl TryFrom<&Packet> for Message {
    type Error = Error;

    fn try_from(value: &Packet) -> Result<Self> {
        trace!("Trying to parse message type {:?}", value.state_type);
        match value.state_type {
            StateType::Syn => Ok(Message::Connect(value.connection_id)),
            StateType::State => Ok(Message::State(
                value.connection_id,
                value.sequence_number,
                value.acknowledge_number,
            )),
            StateType::Data => Ok(Message::Data(value.connection_id, value.payload.clone())),
            StateType::Fin => Ok(Message::Close(value.connection_id)),
            StateType::Reset => Ok(Message::Terminate(value.connection_id)),
        }
    }
}

/// The extensions of an uTP packet.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
enum Extension {
    None = 0,
    SelectiveAck = 1,
}

impl TryFrom<u8> for Extension {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Extension::None),
            1 => Ok(Extension::SelectiveAck),
            _ => Err(Error::UnsupportedExtensions(value)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Packet {
    /// The packet type
    pub state_type: StateType,
    /// The uTP packet extension
    pub extension: Extension,
    /// Unique connection identifier of the stream to which the packet belongs
    pub connection_id: ConnectionId,
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
    /// The payload of the packet.
    pub payload: Vec<u8>,
}

impl Packet {
    /// Convert the packet into the uTP protocol wire bytes.
    fn as_bytes(&self) -> Result<Vec<u8>> {
        trace!("Parsing packet {:?}", self);
        let mut buffer = vec![0u8; 2];

        // write the type & version into the first byte
        buffer[0] = (self.state_type as u8) << 4 | 1;
        // write the extension number in the next byte
        buffer[1] = self.extension as u8;
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
        // append the payload
        buffer.write_all(self.payload.as_slice())?;

        Ok(buffer)
    }

    /// Create a new packet for the given uTP message.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to parse into an uTP packet.
    /// * `sequence_number` - The current sequence number of the uTP stream.
    /// * `acknowledge_number` - The last received sequence number of the uTP stream from the remote peer.
    /// * `timestamp_difference_microseconds` - The latency of the uTP stream connection in microseconds.
    fn from_message(
        message: Message,
        sequence_number: SequenceNumber,
        acknowledge_number: SequenceNumber,
        timestamp_difference_microseconds: u32,
        window_size: u32,
    ) -> Self {
        match message {
            Message::Connect(connection_id) => Packet {
                state_type: StateType::Syn,
                extension: Extension::None,
                connection_id,
                timestamp_microseconds: 0,
                timestamp_difference_microseconds,
                window_size,
                sequence_number,
                acknowledge_number,
                payload: Vec::with_capacity(0),
            },
            Message::State(connection_id, seq_number, ack_number) => Packet {
                state_type: StateType::State,
                extension: Extension::None,
                connection_id,
                timestamp_microseconds: 0,
                timestamp_difference_microseconds,
                window_size,
                sequence_number: seq_number,
                acknowledge_number: ack_number,
                payload: Vec::with_capacity(0),
            },
            Message::Data(connection_id, payload) => Packet {
                state_type: StateType::Data,
                extension: Extension::None,
                connection_id,
                timestamp_microseconds: 0,
                timestamp_difference_microseconds,
                window_size,
                sequence_number,
                acknowledge_number,
                payload,
            },
            Message::Terminate(connection_id) => Packet {
                state_type: StateType::Reset,
                extension: Extension::None,
                connection_id,
                timestamp_microseconds: 0,
                timestamp_difference_microseconds,
                window_size,
                sequence_number,
                acknowledge_number,
                payload: Vec::with_capacity(0),
            },
            Message::Close(connection_id) => Packet {
                state_type: StateType::Fin,
                extension: Extension::None,
                connection_id,
                timestamp_microseconds: 0,
                timestamp_difference_microseconds,
                window_size,
                sequence_number,
                acknowledge_number,
                payload: Vec::with_capacity(0),
            },
        }
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
        let extension = Extension::try_from(cursor.read_u8()?)?;
        let connection_id = cursor.read_u16::<BigEndian>()?;
        let timestamp_microseconds = cursor.read_u32::<BigEndian>()?;
        let timestamp_difference_microseconds = cursor.read_u32::<BigEndian>()?;
        let window_size = cursor.read_u32::<BigEndian>()?;
        let sequence_number = cursor.read_u16::<BigEndian>()?;
        let acknowledge_number = cursor.read_u16::<BigEndian>()?;
        // read the remaining bytes as payload
        let mut payload = Vec::new();
        cursor.read_to_end(&mut payload)?;

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
            payload,
        })
    }
}

#[derive(Debug)]
struct PendingPacket {
    packet: Packet,
    total_resends: u32,
    total_failures: u32,
}

impl PendingPacket {
    fn new(packet: Packet) -> Self {
        Self {
            packet,
            total_resends: 0,
            total_failures: 0,
        }
    }

    /// Get the data size of the packet.
    fn packet_size(&self) -> usize {
        self.packet.payload.len()
    }

    /// Increase the resend counter of the pending packet.
    fn increase_resend(&mut self) {
        self.packet.timestamp_microseconds = now_as_micros();
        self.total_resends += 1;
    }

    /// Increase the failures counter of the pending packet.
    /// This indicates that the packet resend failed.
    fn increase_failures(&mut self) {
        self.total_failures += 1;
    }
}

/// Get the current system time as UNIX timestamp in micro seconds.
fn now_as_micros() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|e| e.as_micros() as u32)
        .map_err(|e| {
            error!(
                "Unable to get current system time, invalid system time, {}",
                e
            );
            e
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use popcorn_fx_core::{assert_timeout, available_port, init_logger};

    #[test]
    fn test_utp_conn_id_new() {
        let key = UtpConnId::new();

        let expected_result = key.recv_id + 1;
        let result = key.send_id;

        assert_eq!(
            expected_result, result,
            "expected the send id to be 1 higher than the receive id"
        );
    }

    #[test]
    fn test_state_type_from_integer() {
        let expected_result = StateType::Data;
        let result = StateType::try_from(0);
        assert_eq!(Ok(expected_result), result);

        let expected_result = StateType::Fin;
        let result = StateType::try_from(1);
        assert_eq!(Ok(expected_result), result);

        let expected_result = StateType::State;
        let result = StateType::try_from(2);
        assert_eq!(Ok(expected_result), result);

        let expected_result = StateType::Reset;
        let result = StateType::try_from(3);
        assert_eq!(Ok(expected_result), result);

        let expected_result = StateType::Syn;
        let result = StateType::try_from(4);
        assert_eq!(Ok(expected_result), result);
    }

    #[test]
    fn test_state_type_from_message() {
        let connection_id = 0;

        let message = Message::Connect(connection_id);
        let result = StateType::from(&message);
        assert_eq!(StateType::Syn, result);

        let message = Message::State(connection_id, 0, 0);
        let result = StateType::from(&message);
        assert_eq!(StateType::State, result);

        let message = Message::Data(connection_id, Vec::with_capacity(0));
        let result = StateType::from(&message);
        assert_eq!(StateType::Data, result);

        let message = Message::Terminate(connection_id);
        let result = StateType::from(&message);
        assert_eq!(StateType::Reset, result);

        let message = Message::Close(connection_id);
        let result = StateType::from(&message);
        assert_eq!(StateType::Fin, result);
    }

    #[test]
    fn test_packet_try_from() {
        init_logger!();
        let timestamp_microseconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u32;
        let packet = Packet {
            state_type: StateType::Syn,
            extension: Extension::None,
            connection_id: 12345,
            timestamp_microseconds,
            timestamp_difference_microseconds: 0,
            window_size: 0,
            sequence_number: 0,
            acknowledge_number: 0,
            payload: Vec::with_capacity(0),
        };

        let bytes: Vec<u8> = packet
            .as_bytes()
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

    #[test]
    fn test_utp_socket_connect() {
        init_logger!();
        let (incoming, outgoing) = create_utp_socket_pair();
        let runtime = &outgoing.inner.runtime;

        let target_addr = incoming.addr();
        let outgoing_stream = runtime
            .block_on(outgoing.connect(target_addr))
            .expect("expected an utp stream");

        let expected_result = UtpStreamState::Connected;
        assert_timeout!(
            Duration::from_millis(500),
            expected_result == runtime.block_on(outgoing_stream.state()),
            "expected the stream to be connected"
        );

        let incoming_stream = runtime
            .block_on(incoming.recv())
            .expect("expected an uTP stream");
        let result = runtime.block_on(incoming_stream.state());
        assert_eq!(UtpStreamState::Connected, result);
    }

    #[test]
    fn test_utp_stream_send() {
        init_logger!();
        let expected_result = "Lorem ipsum dolor";
        let (incoming, outgoing) = create_utp_socket_pair();
        let runtime = &outgoing.inner.runtime;
        let target_addr = incoming.addr();
        let stream = runtime
            .block_on(outgoing.connect(target_addr))
            .expect("expected an utp stream");
        let receiving_stream = runtime
            .block_on(incoming.recv())
            .expect("expected an uTP stream");

        assert_timeout!(
            Duration::from_millis(500),
            UtpStreamState::Connected == runtime.block_on(stream.state()),
            "expected the stream to be connected"
        );

        let result = runtime.block_on(stream.send(expected_result.as_bytes()));
        assert_eq!(Ok(()), result);

        let data = runtime
            .block_on(receiving_stream.recv())
            .expect("expected a message to have been received");
        let result = String::from_utf8(data).unwrap();
        assert_eq!(expected_result, result);
    }

    fn create_utp_socket_pair() -> (UtpSocket, UtpSocket) {
        let runtime = Arc::new(Runtime::new().unwrap());
        let mut rng = thread_rng();

        let port = available_port!(rng.gen_range(20000..21000), 21000).unwrap();
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let left = runtime
            .block_on(UtpSocket::new(
                addr,
                Duration::from_secs(2),
                runtime.clone(),
            ))
            .expect("expected a new utp socket");

        let port = available_port!(rng.gen_range(21000..22000), 22000).unwrap();
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let right = runtime
            .block_on(UtpSocket::new(
                addr,
                Duration::from_secs(2),
                runtime.clone(),
            ))
            .expect("expected a new utp socket");

        (left, right)
    }
}
