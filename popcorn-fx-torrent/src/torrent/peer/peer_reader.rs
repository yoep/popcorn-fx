use crate::torrent::peer::protocol::Message;
use crate::torrent::peer::{DataTransferStats, Error, PeerClientInfo};
use byteorder::BigEndian;
use byteorder::ByteOrder;
use derive_more::Display;
use log::{trace, warn};
use std::time::Instant;
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};
use tokio::select;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

/// The events of the peer reader.
#[derive(Debug, Clone)]
pub enum PeerReaderEvent {
    /// Received a message from the remote peer.
    Message(Message, DataTransferStats),
    /// The connection was closed by the remote peer.
    Closed,
}

/// The peer reader is a buffered reader which reads messages from the peer connection stream.
#[derive(Debug, Display)]
#[display(fmt = "{}", client)]
pub struct PeerReader<R>
where
    R: AsyncRead + Unpin,
{
    client: PeerClientInfo,
    reader: BufReader<R>,
    sender: Sender<PeerReaderEvent>,
    cancellation_token: CancellationToken,
}

impl<R> PeerReader<R>
where
    R: AsyncRead + Unpin,
{
    /// Create a new reader for the peer connection reader stream.
    pub fn new(
        client: PeerClientInfo,
        reader: R,
        sender: Sender<PeerReaderEvent>,
        cancellation_token: CancellationToken,
    ) -> Self {
        Self {
            client,
            reader: BufReader::new(reader),
            sender,
            cancellation_token,
        }
    }

    /// Start the main loop of the reader.
    pub async fn start_read_loop(&mut self) {
        loop {
            let mut buffer = vec![0u8; 4];

            select! {
                _ = self.cancellation_token.cancelled() => break,
                read_result = self.reader.read_exact(&mut buffer) => {
                    match read_result {
                        Ok(0) => {
                            trace!("Peer reader {} EOF", self);
                            break
                        },
                        Ok(buffer_size) => {
                            if let Err(e) = self.read_next(&buffer, buffer_size).await {
                                if e != Error::Closed {
                                    warn!("Peer {} failed to read message, {}", self, e);
                                }
                                break
                            }
                        },
                        Err(e) => {
                            warn!("Peer {} reader encountered an error, {}", self, Error::from(e));
                            break
                        }
                    }
                }
            }
        }

        trace!("Peer {} main reader loop ended", self);
        Self::send(&self.client, self.sender.clone(), PeerReaderEvent::Closed).await;
    }

    /// Try to read a specific number of bytes from the stream.
    ///
    /// # Arguments
    ///
    /// * `len` - The number of bytes to read from the stream.
    pub async fn read(&mut self, len: usize) -> crate::torrent::peer::Result<Vec<u8>> {
        let mut buffer = vec![0u8; len];
        let read_result = self.reader.read_exact(&mut buffer).await;

        match read_result {
            Ok(0) => Err(Error::Closed),
            Ok(_) => Ok(buffer),
            Err(e) => Err(Error::Io(e.to_string())),
        }
    }

    async fn read_next(
        &mut self,
        buffer: &[u8],
        buffer_size: usize,
    ) -> crate::torrent::peer::Result<()> {
        // we expect to receive the incoming message length as a BigEndian
        if buffer_size != 4 {
            return Err(Error::InvalidLength(4, buffer_size as u32));
        }

        let length = BigEndian::read_u32(buffer);
        let start_time = Instant::now();
        let bytes = self.read(length as usize).await?;
        let elapsed = start_time.elapsed();

        // we want to unblock the reader thread as soon as possible
        // so we're going to move this whole process into a new separate thread
        let client = self.client.clone();
        let sender = self.sender.clone();
        tokio::spawn(async move {
            match Message::try_from(bytes.as_ref()) {
                Ok(msg) => {
                    Self::send(
                        &client,
                        sender,
                        PeerReaderEvent::Message(
                            msg,
                            DataTransferStats {
                                transferred_bytes: bytes.len(),
                                elapsed_micro: elapsed.as_micros(),
                            },
                        ),
                    )
                    .await;
                }
                Err(e) => warn!("Received invalid message payload for {}, {}", client, e),
            }
        });

        Ok(())
    }

    async fn send(
        client: &PeerClientInfo,
        sender: Sender<PeerReaderEvent>,
        event: PeerReaderEvent,
    ) {
        if let Err(e) = sender.send(event).await {
            warn!("Failed to send peer reader event of {}, {}", client, e)
        }
    }
}
