use crate::torrent::peer::protocol::utils::now_as_micros;
use crate::torrent::peer::protocol::{
    ConnectionId, Extension, Packet, SequenceNumber, StateType, UtpConnId, UtpSocketContext,
    UtpSocketExtension, UtpSocketExtensions, MAX_PACKET_PAYLOAD_SIZE,
};
use crate::torrent::peer::{Error, Result};
use async_trait::async_trait;
use derive_more::Display;
use futures::Future;
use log::{debug, trace, warn};
use rand::random;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::io;
use std::net::SocketAddr;
use std::pin::{pin, Pin};
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::select;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::{Mutex, MutexGuard, RwLock};
use tokio::time::interval;
use tokio_util::sync::CancellationToken;

/// The maximum amount out-of-order packets which can stored in memory.
const MAX_UNACKED_PACKETS: usize = 128;
/// The maximum amount of bytes allowed within the read buffer.
const MAX_READ_BUFFER: usize = 1 * 1024 * 1024; // 1MB

/// The state of an uTP stream connection.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum UtpStreamState {
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

/// A parsed uTP message.
#[derive(Clone, PartialEq)]
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

impl Message {
    /// Convert this message into an uTP packet.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to parse into an uTP packet.
    /// * `sequence_number` - The current sequence number of the uTP stream.
    /// * `acknowledge_number` - The last received sequence number of the uTP stream from the remote peer.
    /// * `timestamp_difference_microseconds` - The latency of the uTP stream connection in microseconds.
    pub fn into_packet(
        self,
        sequence_number: SequenceNumber,
        acknowledge_number: SequenceNumber,
        timestamp_difference_microseconds: u32,
        window_size: u32,
    ) -> Packet {
        let timestamp_microseconds = now_as_micros();
        match self {
            Message::Connect(connection_id) => Packet {
                state_type: StateType::Syn,
                extension: Extension::None,
                connection_id,
                timestamp_microseconds,
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
                timestamp_microseconds,
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
                timestamp_microseconds,
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
                timestamp_microseconds,
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
                timestamp_microseconds,
                timestamp_difference_microseconds,
                window_size,
                sequence_number,
                acknowledge_number,
                payload: Vec::with_capacity(0),
            },
        }
    }
}

impl TryFrom<&Packet> for Message {
    type Error = Error;

    fn try_from(value: &Packet) -> Result<Self> {
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

impl Debug for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Connect(id) => write!(f, "Connect({})", id),
            Message::State(id, seq, ack) => write!(f, "State({}, {}, {})", id, seq, ack),
            Message::Data(id, data) => write!(f, "Data({}, len {})", id, data.len()),
            Message::Terminate(id) => write!(f, "Terminate({})", id),
            Message::Close(id) => write!(f, "Close({})", id),
        }
    }
}

/// A uTorrent transport protocol connection stream.
/// This stream allows to read and write to a specific uTP connection.
#[derive(Debug, Display)]
#[display(fmt = "{}", inner)]
pub struct UtpStream {
    inner: Arc<UtpStreamContext>,
}

impl UtpStream {
    /// Try to create a new outgoing uTP stream for the given address.
    /// This will initiate the SYN process with the remote socket address.
    pub(crate) async fn new_outgoing(
        key: UtpConnId,
        addr: SocketAddr,
        socket: Arc<UtpSocketContext>,
        message_receiver: UnboundedReceiver<Packet>,
        extensions: Arc<UtpSocketExtensions>,
    ) -> Result<Self> {
        let seq_number = 1;
        let inner = Self::new(
            key,
            addr,
            socket,
            UtpStreamState::Initializing,
            seq_number,
            0,
            extensions,
        );

        let inner_main_loop = inner.clone();
        tokio::spawn(async move {
            inner_main_loop.start(message_receiver).await;
        });

        inner.send_syn().await?;
        Ok(Self { inner })
    }

    /// Try to accept a new incoming uTP stream for the given address.
    /// This will finish the SYN process with the remote socket address.
    pub(crate) async fn new_incoming(
        key: UtpConnId,
        addr: SocketAddr,
        socket: Arc<UtpSocketContext>,
        ack_number: u16,
        message_receiver: UnboundedReceiver<Packet>,
        extensions: Arc<UtpSocketExtensions>,
    ) -> Result<Self> {
        let inner = Self::new(
            key,
            addr,
            socket,
            UtpStreamState::SynRecv,
            random(),
            ack_number,
            extensions,
        );

        let inner_main_loop = inner.clone();
        tokio::spawn(async move {
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

    /// Get the state if the uTP stream connection.
    pub async fn state(&self) -> UtpStreamState {
        *self.inner.state.read().await
    }

    /// Close the uTP stream.
    pub async fn close(&self) -> Result<()> {
        self.inner.close().await
    }

    fn new(
        key: UtpConnId,
        addr: SocketAddr,
        socket: Arc<UtpSocketContext>,
        state: UtpStreamState,
        seq_number: u16,
        ack_number: u16,
        extensions: Arc<UtpSocketExtensions>,
    ) -> Arc<UtpStreamContext> {
        Arc::new(UtpStreamContext {
            key,
            addr,
            socket,
            state: RwLock::new(state),
            seq_number: Mutex::new(seq_number),
            ack_number: Mutex::new(ack_number),
            last_ack_number: Mutex::new(seq_number - 1),
            pending_incoming_packets: Default::default(),
            pending_outgoing_packets: Default::default(),
            timestamp_difference_microseconds: Default::default(),
            read_buffer: Default::default(),
            read_buffer_waker: Default::default(),
            write_buffer: Default::default(),
            write_buffer_waker: Default::default(),
            remote_window_size: Mutex::new(MAX_READ_BUFFER as u32),
            extensions,
            cancellation_token: Default::default(),
        })
    }
}

impl AsyncRead for UtpStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let mut data = match pin!(self.inner.read_buffer.lock()).poll(cx) {
            Poll::Ready(e) => e,
            Poll::Pending => return Poll::Pending,
        };

        if data.is_empty() {
            match self.inner.register_read_waker(cx) {
                Some(e) => return e,
                None => {}
            }
        }

        let to_copy = std::cmp::min(data.len(), buf.remaining());
        buf.put_slice(data.drain(..to_copy).as_slice());

        Poll::Ready(Ok(()))
    }
}

impl AsyncWrite for UtpStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::result::Result<usize, io::Error>> {
        let mut data = match pin!(self.inner.write_buffer.lock()).poll(cx) {
            Poll::Ready(e) => e,
            Poll::Pending => return Poll::Pending,
        };

        data.extend_from_slice(buf);

        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), io::Error>> {
        let mut data = match pin!(self.inner.write_buffer.lock()).poll(cx) {
            Poll::Ready(e) => e,
            Poll::Pending => return Poll::Pending,
        };

        // if there's no data to flush, return success immediately
        if data.is_empty() {
            return Poll::Ready(Ok(()));
        }

        // check if the current stream state allows writing data to the remote peer
        let is_writing_allowed = match pin!(self.inner.is_writing_allowed(data.as_slice())).poll(cx)
        {
            Poll::Ready(e) => e,
            Poll::Pending => return Poll::Pending,
        };
        if !is_writing_allowed {
            self.inner.register_write_waker(cx);
            return Poll::Pending;
        }

        let result = pin!(self.inner.send_data(data.drain(..).as_slice())).poll(cx);
        match result {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => {
                if let Error::Io(e) = e {
                    Poll::Ready(Err(e))
                } else {
                    Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, e.to_string())))
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), io::Error>> {
        pin!(self.close())
            .poll(cx)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
    }

    fn is_write_vectored(&self) -> bool {
        true
    }
}

impl Drop for UtpStream {
    fn drop(&mut self) {
        trace!("Utp stream {} is being dropped", self);
        self.inner.cancellation_token.cancel();
    }
}

#[derive(Debug, Display)]
#[display(fmt = "{} ({})", socket, addr)]
pub struct UtpStreamContext {
    /// The unique key of the utp stream
    key: UtpConnId,
    /// The remote connected address
    addr: SocketAddr,
    /// The uTP socket writer channel
    socket: Arc<UtpSocketContext>,
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
    /// The uTP stream incoming data buffer of the remote peer.
    read_buffer: Mutex<Vec<u8>>,
    /// The waker awaiting data in the incoming data buffer.
    read_buffer_waker: Mutex<Option<Waker>>,
    /// The uTP stream outgoing data buffer to the remote peer.
    write_buffer: Mutex<Vec<u8>>,
    /// The waker awaiting certain states to send the outgoing data buffer.
    write_buffer_waker: Mutex<Option<Waker>>,
    /// The currently allowed window size of the remote peer.
    remote_window_size: Mutex<u32>,
    /// The immutable extensions of the uTP stream.
    extensions: Arc<UtpSocketExtensions>,
    /// The cancellation token of the stream
    cancellation_token: CancellationToken,
}

impl UtpStreamContext {
    /// Starts the main loop of the utp stream for processing messages.
    async fn start(&self, mut message_receiver: UnboundedReceiver<Packet>) {
        let mut resend_interval = interval(Duration::from_millis(500));
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                packet = message_receiver.recv() => {
                    if let Some(packet) = packet {
                        self.handle_received_packet(packet).await;
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

    /// Get the extensions of the uTP stream.
    pub fn extensions(&self) -> &[Box<dyn UtpSocketExtension>] {
        &self.extensions
    }

    /// Check if writing of the given payload to the remote peer is allowed.
    /// It checks if the stream is in a valid state, and that the remote peer window size allows the writing of the given data.
    ///
    /// # Returns
    ///
    /// It returns true when writing to the remote peer is allowed, else false.
    async fn is_writing_allowed(&self, data: &[u8]) -> bool {
        let state = *self.state.read().await;
        let remote_window_size = *self.remote_window_size.lock().await;
        let data_len = data.len();
        let is_remote_writing_allowed = remote_window_size >= data_len as u32;

        state == UtpStreamState::Connected && is_remote_writing_allowed
    }

    /// Try to parse the given received packet of the remote peer.
    async fn handle_received_packet(&self, mut packet: Packet) {
        // check if the packet is valid for this stream
        if !self.assert_packet(&packet) {
            return;
        }

        // process the extensions for the given packet
        self.process_incoming_extensions(&mut packet).await;
        // calculate the latency of the uTP stream connection from the packet
        self.update_timestamp_difference(&packet).await;
        // update the remote window size info
        self.update_remote_window_size(&packet).await;

        match Message::try_from(&packet) {
            Ok(message) => self.handle_received_message(message, packet).await,
            Err(e) => debug!("Utp stream {} failed to parse packet, {}", self, e),
        }
    }

    /// Try to process the received remote peer message.
    async fn handle_received_message(&self, message: Message, packet: Packet) {
        // process the last ack number of the remote peer
        self.handle_remote_acknowledgment(packet.acknowledge_number)
            .await;
        // process the syn acknowledgment of the remote peer if applicable
        self.handle_syn_ack(packet.sequence_number, packet.state_type)
            .await;

        // check if we've already seen the packet, this can happen due to a resend delay
        let mut ack_number = self.ack_number.lock().await;
        let remote_sequence_number = packet.sequence_number;
        let current_ack_number = *ack_number;
        if !is_less_than(current_ack_number, remote_sequence_number) {
            // check if the message is not a state packet, as state packets will always be guaranteed to be duplicates
            if packet.state_type != StateType::State {
                trace!(
                    "Utp stream {} has already seen packet {}",
                    self,
                    remote_sequence_number
                );
            }
            return;
        }

        let mut send_state_message = false;

        {
            // calculate the difference between the received sequence and our last inbound ack number
            let sequence_diff = current_ack_number.saturating_sub(remote_sequence_number - 1);
            let mut pending_incoming_packets = self.pending_incoming_packets.lock().await;
            // store the out-of-order ahead packet in the buffer is allowed
            if sequence_diff <= MAX_UNACKED_PACKETS as u16 {
                // buffer the incoming out-of-order packet
                pending_incoming_packets.insert(remote_sequence_number, message);
            }
            // if the packet is in order, we process it with any other pending incoming packets
            if sequence_diff == 0 {
                loop {
                    let next_seq_number = *ack_number + 1;
                    if let Some(message) = pending_incoming_packets.remove(&next_seq_number) {
                        // process the incoming message in-order
                        let state_type = StateType::from(&message);
                        self.process_incoming_message(message, next_seq_number)
                            .await;
                        // update the processed ack number if the message is everything but a state message
                        if state_type != StateType::State {
                            *ack_number = next_seq_number;
                            send_state_message = true;
                        }
                    } else {
                        // we don't have the next sequence packet available, stop processing messages
                        break;
                    }
                }
            }
        }

        if send_state_message {
            // confirm the processed packets if we don't have any outgoing data
            let write_buffer_len = self.write_buffer.lock().await.len();
            if write_buffer_len == 0 {
                if let Err(e) = self.send_acknowledgment(*ack_number).await {
                    debug!("Utp stream {}, failed to inform remote peer, {}", self, e);
                }
            }
        }
    }

    /// Process the stream extensions for the given packet of the remote peer.
    async fn process_incoming_extensions(&self, packet: &mut Packet) {
        for extension in self.extensions().iter() {
            extension.incoming(packet, &self).await;
        }
    }

    /// Handle the last acknowledgement number of a remote peer.
    /// This will process any outgoing pending packets up to the given `ack_number`.
    async fn handle_remote_acknowledgment(&self, remote_ack_number: SequenceNumber) {
        // try to find the pending packet belonging to the ack number
        let mut pending_packets = self.pending_outgoing_packets.write().await;
        let mut last_ack_number = self.last_ack_number.lock().await;
        // check if the ack number is not ahead of our current sequence number
        let seq_number = *self.seq_number.lock().await;
        if remote_ack_number > seq_number {
            debug!(
                "Utp stream {} received invalid ack number {}, current sequence number {}",
                self, remote_ack_number, seq_number
            );
            return;
        }
        // check if there is anything to be acked or if we've already caught up
        let ack_range = Self::calculate_ack_range(remote_ack_number, &mut last_ack_number);
        if ack_range.is_empty() {
            return;
        }

        // as the ack number might be the highest sequence number,
        // we need to acknowledge all pending messages up to the given ack number
        trace!(
            "Utp stream {} is processing remote ack number {} (ack range {:?})",
            self,
            remote_ack_number,
            ack_range
        );
        for ack_number in ack_range {
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

    /// Handle the given potential syn ack sequence number of the remote peer.
    ///
    /// # Returns
    ///
    /// It returns true if the packet is a syn ack, else false.
    async fn handle_syn_ack(&self, seq_number: SequenceNumber, packet_type: StateType) {
        let is_state_syn_send = *self.state.read().await == UtpStreamState::SynSent;
        if !is_state_syn_send || packet_type != StateType::State {
            return;
        }

        let ack_number = seq_number;
        // set the index of the remote ack number to be inline with the incoming sequence number determined by the remote peer
        // this prevents us from acknowledging every packet up to the remote sequence number
        *self.ack_number.lock().await = ack_number;
        self.update_state(UtpStreamState::Connected).await;

        debug!(
            "Utp stream {} connection established, initial ack number set to {}",
            self, ack_number
        );
    }

    /// Handle a [StateType::Fin] packet from the remote peer.
    /// This will finalize the connection gracefully.
    async fn handle_close_message(&self) {
        self.cancellation_token.cancel();
        self.update_state(UtpStreamState::Closed).await;
        self.notify_read_waker().await;
    }

    /// Handle a received data payload from the remote peer.
    async fn handle_received_payload(&self, bytes: Vec<u8>) {
        let mut data = self.read_buffer.lock().await;
        data.extend_from_slice(bytes.as_slice());
        self.notify_read_waker().await;
    }

    /// Process an in-order incoming uTP message.
    async fn process_incoming_message(&self, message: Message, seq_number: SequenceNumber) {
        trace!(
            "Utp stream {} is processing incoming message {}, {:?}",
            self,
            seq_number,
            message
        );
        match message {
            Message::Data(_, payload) => {
                self.handle_received_payload(payload).await;
            }
            Message::Terminate(_) => {
                self.handle_close_message().await;
            }
            Message::Close(_) => {
                self.handle_close_message().await;
            }
            _ => {}
        }
    }

    /// Verify if the received packet matches the expected stream connection id.
    fn assert_packet(&self, packet: &Packet) -> bool {
        let connection_id = packet.connection_id;
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
        let seq_number = *self.seq_number.lock().await;
        let ack_number = *self.ack_number.lock().await;
        let syn_message = Message::Connect(self.key.recv_id);

        self.send_message(syn_message, seq_number, ack_number)
            .await?;
        self.update_state(UtpStreamState::SynSent).await;
        Ok(())
    }

    /// Send the current uTP state info to the remote peer.
    async fn send_state(&self) -> Result<()> {
        let ack_number = *self.ack_number.lock().await;
        self.send_acknowledgment(ack_number).await
    }

    /// Send an acknowledgment for a received remote peer packet.
    async fn send_acknowledgment(&self, ack_number: SequenceNumber) -> Result<()> {
        let seq_number = *self.seq_number.lock().await;
        let message = Message::State(self.key.send_id, seq_number, ack_number);
        self.send_message(message, seq_number, ack_number).await?;
        Ok(())
    }

    /// Send the given data to the remote peer.
    /// It will send one or more packets depending on the given payload size.
    async fn send_data(&self, bytes: &[u8]) -> Result<()> {
        let mut seq_number = self.seq_number.lock().await;
        let ack_number = *self.ack_number.lock().await;

        // send the data in chunks to not exceed the maximum uTP packet size
        for chunk in bytes.chunks(MAX_PACKET_PAYLOAD_SIZE) {
            let message = Message::Data(self.key.send_id, chunk.to_vec());
            *seq_number += 1;
            self.send_message(message, *seq_number, ack_number).await?;
        }

        Ok(())
    }

    /// Send the close state to the remote peer.
    async fn send_close(&self) -> Result<()> {
        let mut seq_number = self.seq_number.lock().await;
        let ack_number = *self.ack_number.lock().await;

        *seq_number += 1;
        self.send_message(Message::Close(self.key.send_id), *seq_number, ack_number)
            .await
    }

    /// Send the given message to the remote peer.
    async fn send_message(
        &self,
        message: Message,
        seq_number: SequenceNumber,
        ack_number: SequenceNumber,
    ) -> Result<()> {
        trace!("Utp stream {} is sending {:?}", self, message);
        let addr = self.addr;
        let timestamp_difference = *self.timestamp_difference_microseconds.lock().await;
        let window_size = self.window_size().await;
        let mut packet =
            message.into_packet(seq_number, ack_number, timestamp_difference, window_size);

        // process the extensions
        for extension in self.extensions().iter() {
            extension.outgoing(&mut packet, &self).await;
        }

        let pending_packet = PendingPacket::new(packet.clone());
        let start_time = Instant::now();
        self.socket.send(packet.clone(), addr).await?;
        let elapsed = start_time.elapsed();
        trace!(
            "Utp stream {} sent {:?} in {}.{:03}ms",
            self,
            pending_packet.packet,
            elapsed.as_millis(),
            elapsed.subsec_micros() % 1000
        );

        // store the pending packet if it's not a state packet (unless it's the initial outgoing Syn state confirmation)
        // this is done as state packets don't have a unique seq number that is confirmed by the remote peer
        let state = *self.state.read().await;
        if pending_packet.packet.state_type != StateType::State || state == UtpStreamState::SynRecv
        {
            self.pending_outgoing_packets
                .write()
                .await
                .push(pending_packet);
        }
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
            .filter(|e| {
                timestamp_now - e.packet.timestamp_microseconds > timeout_threshold.min(5000)
            })
            .take(10)
        {
            // update the packet with the latest info
            pending_packet.packet.timestamp_microseconds = now_as_micros();
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
                .send(pending_packet.packet.clone(), self.addr)
                .await
            {
                Ok(_) => {
                    pending_packet.increase_resend();
                }
                Err(e) => {
                    debug!(
                        "Utp stream {} failed to resend packet {:?}, {}",
                        self, pending_packet, e
                    );
                    pending_packet.increase_failures();
                }
            }
        }
    }

    /// Get the current window size of all in-flight stream messages that have not yet been acked.
    async fn window_size(&self) -> u32 {
        let read_buffer_size = self.read_buffer.lock().await.len();
        let pending_inbound_packets_size: usize = self
            .pending_incoming_packets
            .lock()
            .await
            .iter()
            .map(|(_, message)| {
                if let Message::Data(_, data) = message {
                    return data.len();
                }

                0
            })
            .sum();

        let remaining_window_size =
            MAX_READ_BUFFER - read_buffer_size - pending_inbound_packets_size;
        remaining_window_size as u32
    }

    /// Update the stream state.
    /// The update is ignored if the stream is already in the given state.
    async fn update_state(&self, state: UtpStreamState) {
        {
            let mut mutex = self.state.write().await;
            if *mutex == state {
                return;
            }
            *mutex = state;
        }

        self.notify_write_waker().await;
        debug!("Utp stream {} state changed to {:?}", self, state);
    }

    /// Update the timestamp difference information of the stream connection.
    async fn update_timestamp_difference(&self, packet: &Packet) {
        let timestamp = now_as_micros();
        let timestamp_difference = timestamp.saturating_sub(packet.timestamp_microseconds);
        *self.timestamp_difference_microseconds.lock().await = timestamp_difference;
    }

    /// Update the currently allowed window size of the remote peer.
    /// This might wake any pending writes if the window size was modified.
    async fn update_remote_window_size(&self, packet: &Packet) {
        let mut mutex = self.remote_window_size.lock().await;
        let remote_window_size = packet.window_size;

        *mutex = remote_window_size;
        self.notify_write_waker().await;
    }

    /// Try to gracefully close the connection with the remote peer.
    async fn close(&self) -> Result<()> {
        let state = *self.state.read().await;
        if state == UtpStreamState::Closed {
            return Ok(());
        }

        let result = self.send_close().await;
        self.socket.close_connection(self.key).await;
        // update the state to close before cancelling the context
        // as the main loop might otherwise execute the close twice
        self.update_state(UtpStreamState::Closed).await;
        self.cancellation_token.cancel();
        self.notify_write_waker().await;
        self.notify_read_waker().await;
        result
    }

    /// Notify the write waker, if present, that the state changed and the writer might be able to write data.
    async fn notify_write_waker(&self) {
        if let Some(waker) = self.write_buffer_waker.lock().await.take() {
            waker.wake();
        }
    }

    /// Notify the read waker, if present, that the state changed and the reader might be able to fetch some data.
    async fn notify_read_waker(&self) {
        if let Some(waker) = self.read_buffer_waker.lock().await.take() {
            waker.wake();
        }
    }

    /// Register a new write waker for the given context.
    fn register_write_waker(&self, cx: &mut Context) {
        if let Poll::Ready(mut mutex) = pin!(self.write_buffer_waker.lock()).poll(cx) {
            *mutex = Some(cx.waker().clone());
        }
    }

    /// Register a new read waker for the given context.
    fn register_read_waker(
        &self,
        cx: &mut Context,
    ) -> Option<Poll<std::result::Result<(), io::Error>>> {
        match pin!(self.state.read()).poll(cx) {
            Poll::Ready(state) => {
                if *state != UtpStreamState::Closed {
                    if let Poll::Ready(mut mutex) = pin!(self.read_buffer_waker.lock()).poll(cx) {
                        *mutex = Some(cx.waker().clone());
                    }

                    Some(Poll::Pending)
                } else {
                    Some(Poll::Ready(Ok(())))
                }
            }
            Poll::Pending => Some(Poll::Pending),
        }
    }

    /// Calculate the range of outgoing packets that need to be acknowledged.
    /// It might return an empty range if the outgoing packets have already been acknowledged before.
    fn calculate_ack_range(
        remote_ack_number: SequenceNumber,
        last_ack_number: &MutexGuard<SequenceNumber>,
    ) -> std::ops::Range<SequenceNumber> {
        let start_index = **last_ack_number + 1;
        let end_index = remote_ack_number + 1;

        // check if the ack range has already been processed
        // this can happen if a packet has been resend or received out-of-order
        if end_index < start_index {
            return 0..0;
        }

        start_index..end_index
    }
}

/// The selective acks extension for the uTP socket connection.
/// This allows non-sequentially ack packets.
#[derive(Debug)]
pub struct UtpSelectiveAckExtension;

#[async_trait]
impl UtpSocketExtension for UtpSelectiveAckExtension {
    async fn incoming(&self, packet: &mut Packet, stream: &UtpStreamContext) {
        match packet.extension {
            Extension::SelectiveAck => {
                // TODO
                warn!(
                    "Utp stream {} selective acks extensions not yet implemented",
                    stream
                );
            }
            _ => {}
        }
    }

    async fn outgoing(&self, _packet: &mut Packet, _stream: &UtpStreamContext) {
        // TODO
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

/// Determines if the value `a` is considered less than the value `b` using a wrap-around comparison.
///
/// This function is particularly useful in contexts where values wrap around a fixed range, such as
/// sequence numbers in a circular buffer or modular arithmetic.
///
/// # Returns
///
/// It returns `true` if `a` is considered less than `b` according to the wrap-around logic; otherwise, returns `false`.
fn is_less_than(a: u16, b: u16) -> bool {
    if b < 0x8000 {
        a < b || a >= b.wrapping_sub(0x8000)
    } else {
        a < b && a >= b.wrapping_sub(0x8000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::torrent::peer::protocol::tests::UtpPacketCaptureExtension;
    use crate::torrent::peer::tests::{create_utp_socket, create_utp_stream_pair};
    use crate::{create_utp_socket_pair, recv_timeout};

    use popcorn_fx_core::{assert_timeout, init_logger};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::sync::mpsc::unbounded_channel;

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

    #[tokio::test]
    async fn test_utp_stream_new_incoming() {
        init_logger!();
        let initial_sequence_number = 1u16;
        let (_sender, receiver) = unbounded_channel();
        let socket = create_utp_socket().await;
        let context = socket.context();
        let capture = UtpPacketCaptureExtension::new();

        let result = UtpStream::new_incoming(
            UtpConnId::new(),
            SocketAddr::from(socket.addr()),
            context.clone(),
            initial_sequence_number,
            receiver,
            Arc::new(vec![Box::new(capture.clone())]),
        )
        .await
        .expect("expected an uTP stream to have been created");

        // check the initial sequence number
        let outgoing_packet = capture
            .outgoing_packets()
            .await
            .get(0)
            .cloned()
            .expect("expected an outgoing packet to have been sent");
        let seq_number_result = *result.inner.seq_number.lock().await;
        assert_ne!(
            1u16, seq_number_result,
            "expected our own seq_number to be random picked"
        );
        assert_eq!(
            outgoing_packet.sequence_number, seq_number_result,
            "expected the random seq_number to have been sent in the syn ack to the remote peer"
        );

        // check the initial remote ack number
        let ack_number_result = *result.inner.ack_number.lock().await;
        assert_eq!(
            1u16, ack_number_result,
            "expected the initial remote ack_number to match"
        );
        assert_eq!(
            outgoing_packet.acknowledge_number, ack_number_result,
            "expected the initial seq_number to have been acked to the remote peer"
        );

        // check the initial last_ack_number which should be one less than the initial state seq_number
        let expected_last_ack = seq_number_result - 1;
        let last_ack_result = *result.inner.last_ack_number.lock().await;
        assert_eq!(
            expected_last_ack, last_ack_result,
            "expected the remote last acknowledged number to match"
        );
    }

    #[tokio::test]
    async fn test_utp_stream_handle_received_packet_ack_syn_sent() {
        init_logger!();
        let sequence_number = 64;
        let (_sender, receiver) = unbounded_channel();
        let socket = create_utp_socket().await;
        let context = socket.context();
        let capture = UtpPacketCaptureExtension::new();

        let stream = UtpStream::new_outgoing(
            UtpConnId::new(),
            SocketAddr::from(socket.addr()),
            context.clone(),
            receiver,
            Arc::new(vec![Box::new(capture.clone())]),
        )
        .await
        .expect("expected an uTP stream to have been created");

        let packet = Packet {
            state_type: StateType::State,
            extension: Extension::None,
            connection_id: stream.inner.key.recv_id,
            timestamp_microseconds: now_as_micros(),
            timestamp_difference_microseconds: 1500,
            window_size: MAX_READ_BUFFER as u32,
            sequence_number,
            acknowledge_number: 1,
            payload: vec![],
        };
        stream.inner.handle_received_packet(packet).await;

        // check the current ack number
        let incoming_packet = capture
            .incoming_packets()
            .await
            .get(0)
            .cloned()
            .expect("expected to have received an incoming syn ack packet");
        let result = *stream.inner.ack_number.lock().await;
        assert_eq!(sequence_number, result, "expected the ack number of the remote peer to have been set to the incoming sequence number");
        assert_eq!(
            incoming_packet.sequence_number, result,
            "expected the ack seq_number to have been the same as the initial ack packet"
        );

        // check the pending outgoing packets
        let result = stream.inner.pending_outgoing_packets.read().await;
        assert_eq!(
            0,
            result.len(),
            "expected the syn packet to have been confirmed, got {:?} instead",
            &*result
        );
    }

    #[tokio::test]
    async fn test_utp_stream_handle_received_message_state_update() {
        init_logger!();
        let expected_sequence_number = 13;
        let (_sender, receiver) = unbounded_channel();
        let socket = create_utp_socket().await;
        let context = socket.context();

        let stream = UtpStream::new_incoming(
            UtpConnId::new(),
            SocketAddr::from(socket.addr()),
            context.clone(),
            expected_sequence_number,
            receiver,
            Arc::new(vec![Box::new(UtpPacketCaptureExtension::new())]),
        )
        .await
        .expect("expected an uTP stream to have been created");

        let packet = Packet {
            state_type: StateType::State,
            extension: Extension::None,
            connection_id: stream.inner.key.recv_id,
            timestamp_microseconds: 0,
            timestamp_difference_microseconds: 0,
            window_size: 0,
            sequence_number: 64,
            acknowledge_number: 1,
            payload: vec![],
        };
        let message = Message::try_from(&packet).unwrap();
        stream.inner.update_state(UtpStreamState::Connected).await;
        stream.inner.handle_received_message(message, packet).await;

        let ack_number = *stream.inner.ack_number.lock().await;
        assert_eq!(
            expected_sequence_number, ack_number,
            "expected the ack number to not have been updated"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_utp_stream_connection_pairing() {
        init_logger!();
        let expected_outgoing_syn_sequence_number = 1u16;
        let incoming_capture = UtpPacketCaptureExtension::new();
        let outgoing_capture = UtpPacketCaptureExtension::new();
        let (incoming, outgoing) = create_utp_socket_pair!(
            vec![Box::new(incoming_capture.clone())],
            vec![Box::new(outgoing_capture.clone())]
        );
        let (incoming_stream, outgoing_stream) = create_utp_stream_pair(&incoming, &outgoing).await;

        assert_timeout!(
            Duration::from_millis(500),
            UtpStreamState::Connected == *incoming_stream.inner.state.read().await,
            "expected the incoming stream to be connected"
        );
        assert_timeout!(
            Duration::from_millis(500),
            UtpStreamState::Connected == *outgoing_stream.inner.state.read().await,
            "expected the outgoing stream to be connected"
        );

        // check the outgoing_stream packets
        let outgoing_packet = outgoing_capture
            .outgoing_packets()
            .await
            .get(0)
            .cloned()
            .expect("expected to have sent a packet");
        let incoming_packet = outgoing_capture
            .incoming_packets()
            .await
            .get(0)
            .cloned()
            .expect("expected to have received a packet");
        assert_eq!(
            StateType::Syn,
            outgoing_packet.state_type,
            "expected the initial outgoing packet to be a syn"
        );
        assert_eq!(
            expected_outgoing_syn_sequence_number, outgoing_packet.sequence_number,
            "expected the initial outgoing seq_number to be 1"
        );
        assert_eq!(
            StateType::State,
            incoming_packet.state_type,
            "expected the initial incoming packet to be state ack for the syn"
        );
        assert_eq!(
            expected_outgoing_syn_sequence_number, incoming_packet.acknowledge_number,
            "expected the initial ack to confirm the syn packet"
        );

        // check the incoming_stream packets
        let outgoing_packet = incoming_capture
            .outgoing_packets()
            .await
            .get(0)
            .cloned()
            .expect("expected to have sent a packet");
        assert_eq!(
            StateType::State,
            outgoing_packet.state_type,
            "expected the initial outgoing packet to be a confirmation of the syn packet"
        );
        assert_eq!(
            expected_outgoing_syn_sequence_number, outgoing_packet.acknowledge_number,
            "expected the initial seq_number of the syn packet to be acknowledged"
        );
        assert_eq!(
            incoming_packet, outgoing_packet,
            "expected the outgoing packet to have been the same as the receiving end"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_utp_stream_outgoing_write_incoming_read() {
        init_logger!();
        let expected_result = "Nullam varius felis in massa eleifend consectetur.";
        let incoming_capture = UtpPacketCaptureExtension::new();
        let outgoing_capture = UtpPacketCaptureExtension::new();
        let (incoming, outgoing) = create_utp_socket_pair!(
            vec![Box::new(incoming_capture.clone())],
            vec![Box::new(outgoing_capture.clone())]
        );
        let (mut incoming_stream, mut outgoing_stream) =
            create_utp_stream_pair(&incoming, &outgoing).await;
        let (tx, mut rx) = unbounded_channel();

        assert_timeout!(
            Duration::from_millis(500),
            UtpStreamState::Connected == *outgoing_stream.inner.state.read().await,
            "expected the stream to be connected"
        );

        tokio::spawn(async move {
            let mut buffer = vec![0u8; expected_result.as_bytes().len()];
            let result_buffer_len = incoming_stream
                .read_exact(&mut buffer)
                .await
                .expect("expected a message to have been received");
            tx.send((result_buffer_len, buffer)).unwrap();
        });

        let bytes = expected_result.as_bytes();
        let bytes_len = bytes.len();
        outgoing_stream.write(bytes).await.unwrap();
        outgoing_stream.flush().await.unwrap();

        // check the outgoing packets of the outgoing_stream
        let outgoing_packets = outgoing_capture.outgoing_packets().await.clone();
        let syn_packet = outgoing_packets
            .get(0)
            .expect("expected an outgoing syn packet");
        assert_eq!(
            StateType::Syn,
            syn_packet.state_type,
            "expected the initial outgoing message to be a syn"
        );
        assert_eq!(
            1u16, syn_packet.sequence_number,
            "expected the syn packet to have seq_number 1"
        );
        let data_packet = outgoing_packets
            .get(1)
            .expect("expected an outgoing data packet");
        assert_eq!(
            StateType::Data,
            data_packet.state_type,
            "expected the 2nd outgoing packet to be a data packet"
        );
        assert_eq!(
            2u16, data_packet.sequence_number,
            "expected the seq_number to have been increased"
        );

        // check the read result of the receiving stream
        let (result_buffer_len, buffer) = recv_timeout!(&mut rx, Duration::from_millis(500));
        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(
            bytes_len, result_buffer_len,
            "expected the read bytes to be the same as the written bytes"
        );
        assert_eq!(expected_result, result);

        // check the outgoing packets of the incoming_stream
        let outgoing_packets = incoming_capture.outgoing_packets().await.clone();
        let syn_ack_packet = outgoing_packets
            .get(0)
            .expect("expected initial syn ack packet");
        assert_eq!(
            StateType::State,
            syn_ack_packet.state_type,
            "expected the initial outgoing packet to be a syn ack state packet"
        );
        assert_eq!(
            syn_packet.sequence_number, syn_ack_packet.acknowledge_number,
            "expected the syn seq_number to be acked"
        );
        let data_ack_packet = outgoing_packets.get(1).expect("expected a data ack packet");
        assert_eq!(
            StateType::State,
            data_ack_packet.state_type,
            "expected the data packet to be acked"
        );
        assert_eq!(
            data_packet.sequence_number, data_ack_packet.acknowledge_number,
            "expected the data seq_number to be acked"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_utp_stream_outgoing_read_incoming_write() {
        init_logger!();
        let expected_result = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.";
        let (incoming, outgoing) = create_utp_socket_pair!();
        let (mut incoming_stream, mut outgoing_stream) =
            create_utp_stream_pair(&incoming, &outgoing).await;
        let (tx, mut rx) = unbounded_channel();

        assert_timeout!(
            Duration::from_millis(500),
            UtpStreamState::Connected == *outgoing_stream.inner.state.read().await,
            "expected the stream to be connected"
        );

        tokio::spawn(async move {
            let mut buffer = vec![0u8; expected_result.as_bytes().len()];
            let result_buffer_len = outgoing_stream
                .read_exact(&mut buffer)
                .await
                .expect("expected a message to have been received");
            tx.send((result_buffer_len, buffer)).unwrap();
        });

        let bytes = expected_result.as_bytes();
        let bytes_len = bytes.len();
        incoming_stream.write(bytes).await.unwrap();
        incoming_stream.flush().await.unwrap();

        let (result_buffer_len, buffer) = recv_timeout!(&mut rx, Duration::from_millis(500));
        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(
            bytes_len, result_buffer_len,
            "expected the read bytes to be the same as the written bytes"
        );
        assert_eq!(expected_result, result);
    }

    // FIXME: unstable in Github actions
    #[ignore]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_utp_stream_close() {
        init_logger!();
        let (incoming, outgoing) = create_utp_socket_pair!();
        let (incoming_stream, outgoing_stream) = create_utp_stream_pair(&incoming, &outgoing).await;

        // close the outgoing stream
        outgoing_stream
            .close()
            .await
            .expect("expected the stream to close");

        // check if the incoming stream has also been closed
        assert_timeout!(
            Duration::from_secs(1),
            UtpStreamState::Closed == *incoming_stream.inner.state.read().await,
            "expected the stream to be closed"
        );
    }

    // FIXME: unstable in Github actions
    #[ignore]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_utp_stream_shutdown() {
        init_logger!();
        let (incoming, outgoing) = create_utp_socket_pair!();
        let (incoming_stream, mut outgoing_stream) =
            create_utp_stream_pair(&incoming, &outgoing).await;

        // close the stream through the shutdown fn
        outgoing_stream
            .shutdown()
            .await
            .expect("expected the stream to close");

        // check if the incoming stream has also been closed
        assert_timeout!(
            Duration::from_millis(500),
            UtpStreamState::Closed == *incoming_stream.inner.state.read().await,
            "expected the stream to be closed"
        );
    }

    #[tokio::test]
    async fn test_calculate_ack_range() {
        let mutex = Mutex::new(0);
        let mut last_ack = mutex.lock().await;

        let result = UtpStreamContext::calculate_ack_range(1, &last_ack);
        assert_eq!(1..2, result);
        assert_eq!(1, result.len(), "expected a total of 1 packet to be acked");

        *last_ack = 10;
        let result = UtpStreamContext::calculate_ack_range(8, &last_ack);
        assert_eq!(0..0, result, "expected an empty range to be acked");

        *last_ack = 9;
        let result = UtpStreamContext::calculate_ack_range(15, &last_ack);
        assert_eq!(10..16, result);
        assert_eq!(6, result.len(), "expected a total of 6 packets to be acked");
    }

    #[test]
    fn test_is_less_than() {
        let a = 1000;
        let b = 2000;
        assert_eq!(true, is_less_than(a, b));

        let a = 60000;
        let b = 1000;
        assert_eq!(true, is_less_than(a, b));

        let a = 30000;
        let b = 20000;
        assert_eq!(false, is_less_than(a, b));
    }
}
