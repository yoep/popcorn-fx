use crate::ipc::errors::{Error, Result};
use crate::ipc::proto::message::FxMessage;
use byteorder::{BigEndian, ByteOrder};
use log::{debug, error, trace, warn};
use protobuf::Message;
use std::collections::HashMap;
use std::io;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{split, AsyncRead, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::{oneshot, Mutex, MutexGuard, Notify};
use tokio::{select, time};
use tokio_util::sync::{CancellationToken, WaitForCancellationFuture};

/// An IPC channel used for communication with the parent process.
#[derive(Debug, Clone)]
pub struct IpcChannel {
    inner: Arc<InnerIpcChannel>,
}

impl IpcChannel {
    pub fn new(stream: TcpStream, timeout: Duration) -> Self {
        let (reader, writer) = split(stream);
        let (reader_sender, reader_receiver) = unbounded_channel();

        let inner = Arc::new(InnerIpcChannel {
            sequence_id: Default::default(),
            writer: Mutex::new(writer),
            buffer: Default::default(),
            buffer_notify: Default::default(),
            pending_requests: Default::default(),
            timeout,
            cancellation_token: Default::default(),
        });

        let inner_main = inner.clone();
        tokio::spawn(async move {
            inner_main.start(reader_receiver).await;
        });

        let mut reader = IpcChannelReader {
            reader,
            sender: reader_sender,
            timeout,
            cancellation_token: inner.cancellation_token.clone(),
        };
        tokio::spawn(async move {
            reader.start().await;
        });

        Self { inner }
    }

    /// Try to receive a new message from the channel.
    ///
    /// # Returns
    ///
    /// It returns a message when received, else [None] when the channel is closed.
    pub async fn recv(&self) -> Option<FxMessage> {
        loop {
            {
                let mut buffer = self.inner.buffer.lock().await;
                if !buffer.is_empty() {
                    return Some(buffer.remove(0));
                }
            }

            if self.inner.cancellation_token.is_cancelled() {
                return None;
            }

            self.inner.buffer_notify.notified().await;
        }
    }

    /// Try to get a response for the given message from the channel.
    /// This will create a new receiver which will wait for a response to the given message.
    ///
    /// # Returns
    ///
    /// It returns a receiver for the response when completed, else the error that occurred while sending
    /// the message to the channel.
    pub async fn get(
        &self,
        message: impl Message,
        message_type: &str,
    ) -> Result<oneshot::Receiver<FxMessage>> {
        self.inner.get(message, message_type).await
    }

    /// Send the given message to the channel.
    pub async fn send(&self, message: impl Message, message_type: &str) -> Result<()> {
        self.inner.send_message(message, message_type, None).await?;
        Ok(())
    }

    /// Reply to a request message with a response.
    /// This will link the response to the request and send it to the channel.
    pub async fn send_reply(
        &self,
        request: &FxMessage,
        message: impl Message,
        message_type: &str,
    ) -> Result<()> {
        self.inner
            .send_message(message, message_type, Some(request.sequence_id))
            .await?;
        Ok(())
    }

    /// Close the IPC channel.
    pub fn close(&self) {
        trace!("IPC channel is being closed by client");
        self.inner.cancellation_token.cancel();
    }

    /// Get a future which resolves when the channel is closed.
    /// The future might immediately resolve if the channel is already closed.
    pub fn closed(&self) -> WaitForCancellationFuture<'_> {
        self.inner.cancellation_token.cancelled()
    }
}

/// A builder pattern for creating new [FxMessage] instances.
#[derive(Debug, Default)]
pub struct FxMessageBuilder {
    type_: Option<String>,
    sequence_id: Option<u32>,
    reply_to: Option<u32>,
    payload: Option<Vec<u8>>,
}

impl FxMessageBuilder {
    /// Get a new builder instance.
    /// This is an alias for the [FxMessageBuilder::default()].
    pub fn new() -> Self {
        Self::default()
    }

    pub fn type_(&mut self, type_: impl Into<String>) -> &mut Self {
        self.type_ = Some(type_.into());
        self
    }

    pub fn sequence_id(&mut self, sequence_id: u32) -> &mut Self {
        self.sequence_id = Some(sequence_id);
        self
    }

    pub fn reply_to(&mut self, reply_to: Option<u32>) -> &mut Self {
        self.reply_to = reply_to;
        self
    }

    pub fn payload(&mut self, payload: Vec<u8>) -> &mut Self {
        self.payload = Some(payload);
        self
    }

    /// Create a new [FxMessage] instance, consuming all the data stored within the builder.
    ///
    /// # Panics
    ///
    /// It panics when the `type` or `sequence_id` have not been set.
    pub fn build(&mut self) -> FxMessage {
        FxMessage {
            type_: self.type_.take().expect("type is required"),
            sequence_id: self.sequence_id.take().expect("sequence ID is required"),
            reply_to: self.reply_to.take(),
            payload: self.payload.take().unwrap_or(Vec::with_capacity(0)),
            special_fields: Default::default(),
        }
    }
}

#[derive(Debug)]
struct InnerIpcChannel {
    sequence_id: AtomicU32,
    writer: Mutex<WriteHalf<TcpStream>>,
    buffer: Mutex<Vec<FxMessage>>,
    buffer_notify: Notify,
    pending_requests: Mutex<HashMap<u32, oneshot::Sender<FxMessage>>>,
    timeout: Duration,
    cancellation_token: CancellationToken,
}

impl InnerIpcChannel {
    async fn start(&self, mut reader_receiver: UnboundedReceiver<FxMessage>) {
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(message) = reader_receiver.recv() => self.handle_message(message).await,
            }
        }
        self.buffer_notify.notify_waiters();
        debug!("IPC channel main loop ended");
    }

    async fn handle_message(&self, message: FxMessage) {
        match &message.reply_to {
            None => {
                self.buffer.lock().await.push(message);
                self.buffer_notify.notify_waiters();
            }
            Some(reply_id) => match self.pending_requests.lock().await.remove(reply_id) {
                None => {
                    warn!(
                        "IPC channel failed to process message, unknown reply id {}",
                        reply_id
                    );
                }
                Some(sender) => {
                    if let Err(_) = sender.send(message) {
                        error!("IPC channel failed to send reply, reply channel has already been consumed");
                    }
                }
            },
        }
    }

    async fn get(
        &self,
        message: impl Message,
        message_type: &str,
    ) -> Result<oneshot::Receiver<FxMessage>> {
        let sequence_id = self.sequence_id.fetch_add(1, Ordering::SeqCst);
        let (sender, receiver) = oneshot::channel();

        {
            let mut requests = self.pending_requests.lock().await;
            requests.insert(sequence_id, sender);
        }

        self.send_message_with_sequence(sequence_id, message, message_type, None)
            .await?;
        Ok(receiver)
    }

    /// Try to send the given message to the channel.
    async fn send_message(
        &self,
        message: impl Message,
        message_type: &str,
        reply_to_id: Option<u32>,
    ) -> Result<()> {
        let sequence_id = self.sequence_id.fetch_add(1, Ordering::SeqCst);
        self.send_message_with_sequence(sequence_id, message, message_type, reply_to_id)
            .await
    }

    async fn send_message_with_sequence(
        &self,
        sequence_id: u32,
        message: impl Message,
        message_type: &str,
        reply_to_id: Option<u32>,
    ) -> Result<()> {
        let bytes = message.write_to_bytes()?;
        self.send_payload(sequence_id, bytes, message_type, reply_to_id)
            .await?;
        Ok(())
    }

    /// Try to send the given message payload to the channel.
    async fn send_payload(
        &self,
        sequence_id: u32,
        payload: Vec<u8>,
        message_type: &str,
        reply_to_id: Option<u32>,
    ) -> Result<()> {
        let mut writer = self.writer.lock().await;

        if reply_to_id.is_some() && message_type.contains("Request") {
            warn!(
                "IPC channel detected potential incorrect response type \"{}\"",
                message_type
            );
        }

        let message = FxMessageBuilder::new()
            .type_(message_type)
            .sequence_id(sequence_id)
            .reply_to(reply_to_id)
            .payload(payload)
            .build();
        let bytes = message.write_to_bytes()?;

        // write the message length as the first 4 bytes
        let mut length_buffer = vec![0u8; 4];
        BigEndian::write_u32(&mut length_buffer[..4], bytes.len() as u32);

        trace!(
            "IPC channel is trying to write message \"{}\" ({} bytes)",
            message_type,
            bytes.len()
        );
        let start_time = Instant::now();
        select! {
            _ = time::sleep(self.timeout) => Err(Error::Io(io::Error::new(io::ErrorKind::TimedOut, "writer timed out sending payload"))),
           result = Self::write_channel_message(&mut writer, &length_buffer, &bytes) => result,
        }?;
        let elapsed = start_time.elapsed();
        debug!(
            "IPC channel wrote message \"{}\" ({} bytes) in {}.{:03}ms",
            message_type,
            bytes.len(),
            elapsed.as_millis(),
            elapsed.as_micros()
        );
        Ok(())
    }

    async fn write_channel_message(
        writer: &mut MutexGuard<'_, WriteHalf<TcpStream>>,
        length_buffer: &[u8],
        bytes: &[u8],
    ) -> Result<()> {
        writer.write(length_buffer).await?;
        writer.write_all(&bytes).await?;
        writer.flush().await?;
        Ok(())
    }
}

#[derive(Debug)]
struct IpcChannelReader<R: AsyncRead> {
    reader: ReadHalf<R>,
    sender: UnboundedSender<FxMessage>,
    timeout: Duration,
    cancellation_token: CancellationToken,
}

impl<R: AsyncRead> IpcChannelReader<R> {
    async fn start(&mut self) {
        loop {
            let mut buffer = [0u8; 4];

            select! {
                _ = self.cancellation_token.cancelled() => break,
                read_result = self.reader.read_exact(&mut buffer) => {
                    match read_result {
                        Ok(0) => {
                            trace!("IPC channel reader received EOF");
                            self.cancellation_token.cancel();
                            break;
                        },
                        Ok(len) => {
                            if let Err(e) = self.handle_received_message(&buffer, len).await {
                                error!("IPC channel reader failed to handle message, {}", e);
                                break;
                            }
                        },
                        Err(e) => {
                            error!("IPC channel reader encountered an error, {}", e);
                            self.cancellation_token.cancel();
                            break;
                        }
                    }
                },
            }
        }
        debug!("IPC channel reader main loop ended");
    }

    async fn handle_received_message(&mut self, buffer: &[u8], buffer_len: usize) -> Result<()> {
        if buffer_len != 4 {
            return Err(Error::InvalidLength);
        }

        let start_time = Instant::now();
        let len = BigEndian::read_u32(&buffer[..4]) as usize;
        let mut buffer = vec![0u8; len];

        trace!("IPC channel reader is reading message (size {})", len);
        select! {
            _ = time::sleep(self.timeout) => Err(Error::Io(io::Error::new(io::ErrorKind::TimedOut, "timed out while reading message payload"))),
            result = self.reader.read_exact(&mut buffer) => result.map_err(Error::from),
        }?;
        let elapsed = start_time.elapsed();

        let message = FxMessage::parse_from_bytes(&buffer)?;
        debug!(
            "IPC channel read message \"{}\" ({} bytes) in {}.{:03}ms",
            message.type_,
            buffer.len(),
            elapsed.as_millis(),
            elapsed.as_micros()
        );
        if let Err(e) = self.sender.send(message) {
            warn!(
                "IPC channel reader failed to send payload for processing, {}",
                e
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ipc::proto::update::{update, GetUpdateStateRequest, GetUpdateStateResponse};
    use crate::ipc::test::create_channel_pair;
    use crate::timeout;

    use popcorn_fx_core::init_logger;
    use std::time::Duration;

    #[tokio::test]
    async fn test_ipc_channel_get() {
        init_logger!();
        let (incoming, outgoing) = create_channel_pair().await;

        let receiver = incoming
            .get(
                GetUpdateStateRequest::default(),
                GetUpdateStateRequest::NAME,
            )
            .await
            .unwrap();
        let message = timeout!(outgoing.recv(), Duration::from_millis(200))
            .expect("expected to receive a message");

        let result = outgoing
            .send_reply(
                &message,
                GetUpdateStateResponse {
                    state: update::State::NO_UPDATE_AVAILABLE.into(),
                    special_fields: Default::default(),
                },
                GetUpdateStateResponse::NAME,
            )
            .await;
        assert_eq!(Ok(()), result, "expected the response to have been sent");

        let result =
            timeout!(receiver, Duration::from_millis(200)).expect("expected to receive a response");
        assert_eq!(GetUpdateStateResponse::NAME, result.type_.as_str());
    }
}
