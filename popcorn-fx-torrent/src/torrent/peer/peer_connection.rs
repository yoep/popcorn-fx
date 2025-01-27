use crate::torrent::peer::protocol_bt::{Handshake, Message};
use crate::torrent::peer::protocol_utp::UtpStream;
use crate::torrent::peer::{
    ConnectionType, DataTransferStats, Error, PeerConn, PeerId, PeerResponse, Result,
};
use async_trait::async_trait;
use byteorder::BigEndian;
use byteorder::ByteOrder;
use derive_more::Display;
use log::{trace, warn};
use std::fmt::Debug;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, WriteHalf};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::select;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

/// The bytes length of an expected handshake message
const HANDSHAKE_MESSAGE_LENGTH: usize = 68;

#[derive(Debug, Display)]
#[display(fmt = "{}[{}]", id, addr)]
pub struct PeerConnection<W>
where
    W: AsyncWrite + Debug + Send,
{
    /// The unique peer id of the connection
    id: PeerId,
    /// The remote peer address
    addr: SocketAddr,
    /// The receiver of the peer connection reader
    receiver: Mutex<UnboundedReceiver<PeerReaderEvent>>,
    /// The writer of the connection
    writer: PeerWriter<W>,
    cancellation_token: CancellationToken,
}

impl<W> PeerConnection<W>
where
    W: AsyncWrite + Debug + Send,
{
    pub fn new_tcp(
        id: PeerId,
        addr: SocketAddr,
        stream: TcpStream,
        runtime: Arc<Runtime>,
    ) -> PeerConnection<TcpStream> {
        let cancellation_token = CancellationToken::new();
        let (sender, receiver) = unbounded_channel();
        let (reader, writer) = tokio::io::split(stream);

        let mut reader = PeerReader::new(id, addr, reader, sender, cancellation_token.clone());
        runtime.spawn(async move { reader.start_read_loop().await });

        PeerConnection::<TcpStream> {
            id,
            addr,
            receiver: Mutex::new(receiver),
            writer: PeerWriter::new(writer),
            cancellation_token,
        }
    }

    pub fn new_utp(
        id: PeerId,
        addr: SocketAddr,
        stream: UtpStream,
        runtime: Arc<Runtime>,
    ) -> PeerConnection<UtpStream> {
        let cancellation_token = CancellationToken::new();
        let (sender, receiver) = unbounded_channel();
        let (reader, writer) = tokio::io::split(stream);

        let mut reader = PeerReader::new(id, addr, reader, sender, cancellation_token.clone());
        runtime.spawn(async move { reader.start_read_loop().await });

        PeerConnection::<UtpStream> {
            id,
            addr,
            receiver: Mutex::new(receiver),
            writer: PeerWriter::new(writer),
            cancellation_token,
        }
    }
}

#[async_trait]
impl<W> PeerConn for PeerConnection<W>
where
    W: AsyncWrite + Debug + Send,
{
    async fn recv(&self) -> Option<PeerResponse> {
        if let Some(event) = self.receiver.lock().await.recv().await {
            return Some(match event {
                PeerReaderEvent::Handshake(bytes) => {
                    match Handshake::from_bytes(&self.addr, bytes.as_slice()) {
                        Ok(e) => PeerResponse::Handshake(e),
                        Err(e) => PeerResponse::Error(e),
                    }
                }
                PeerReaderEvent::Message(message, stats) => PeerResponse::Message(message, stats),
                PeerReaderEvent::Closed => {
                    let _ = self.close().await;
                    PeerResponse::Closed
                }
            });
        }

        None
    }

    async fn write<'a>(&'a self, bytes: &'a [u8]) -> Result<()> {
        self.writer.write(bytes).await
    }

    async fn close(&self) -> Result<()> {
        trace!("Peer {} connection is closing", self);
        self.cancellation_token.cancel();
        self.writer.shutdown().await;
        Ok(())
    }
}

impl<W> Drop for PeerConnection<W>
where
    W: AsyncWrite + Debug + Send,
{
    fn drop(&mut self) {
        trace!("Peer {} connection is being dropped", self);
        self.cancellation_token.cancel();
    }
}

#[derive(Debug)]
struct PeerWriter<W>
where
    W: AsyncWrite + Debug,
{
    writer: Mutex<WriteHalf<W>>,
}

impl<W> PeerWriter<W>
where
    W: AsyncWrite + Debug,
{
    fn new(writer: WriteHalf<W>) -> Self {
        Self {
            writer: Mutex::new(writer),
        }
    }

    async fn write<'a>(&'a self, bytes: &'a [u8]) -> Result<()> {
        let mut writer = self.writer.lock().await;
        writer.write_all(bytes.as_ref()).await?;
        writer.flush().await?;
        Ok(())
    }

    async fn shutdown(&self) {
        let _ = self.writer.lock().await.shutdown().await;
    }
}

unsafe impl<W> Send for PeerWriter<W> where W: AsyncWrite + Debug {}

/// The events of the peer reader.
#[derive(Debug, Clone)]
enum PeerReaderEvent {
    /// Received a handshake bytes from the remote peer.
    Handshake(Vec<u8>),
    /// Received a message from the remote peer.
    Message(Message, DataTransferStats),
    /// The connection was closed by the remote peer.
    Closed,
}

/// The peer reader is a buffered reader which reads messages from the peer connection stream.
#[derive(Debug, Display)]
#[display(fmt = "{}[{}]", id, addr)]
struct PeerReader<R>
where
    R: AsyncRead + Unpin,
{
    id: PeerId,
    addr: SocketAddr,
    reader: BufReader<R>,
    sender: UnboundedSender<PeerReaderEvent>,
    cancellation_token: CancellationToken,
}

impl<R> PeerReader<R>
where
    R: AsyncRead + Unpin,
{
    /// Create a new reader for the peer connection reader stream.
    fn new(
        id: PeerId,
        addr: SocketAddr,
        reader: R,
        sender: UnboundedSender<PeerReaderEvent>,
        cancellation_token: CancellationToken,
    ) -> Self {
        Self {
            id,
            addr,
            reader: BufReader::new(reader),
            sender,
            cancellation_token,
        }
    }

    /// Start the main loop of the reader.
    async fn start_read_loop(&mut self) {
        // as initial message, try to read the handshake
        let cancellation_token = self.cancellation_token.clone();
        select! {
            _ = cancellation_token.cancelled() => return,
            read_result = self.read(HANDSHAKE_MESSAGE_LENGTH) => {
                match read_result {
                    Ok(buffer) => {
                        Self::send (
                            self.id,
                            &self.addr,
                            &self.sender,
                            PeerReaderEvent::Handshake(buffer)
                        );
                    }
                    Err(e) => warn!("Peer {} failed to read handshake, {}", self, e),
                }
            }
        }

        loop {
            let mut buffer = vec![0u8; 4];

            select! {
                _ = self.cancellation_token.cancelled() => break,
                read_result = self.reader.read_exact(&mut buffer) => {
                    match read_result {
                        Ok(0) => {
                            trace!("Peer {} reader received EOF", self);
                            break
                        },
                        Ok(buffer_size) => {
                            if let Err(e)  = self.read_next(&buffer, buffer_size).await {
                                if e != Error::Closed {
                                    warn!("Peer {} failed to read message, {}", self, e);
                                }
                                break
                            }
                        },
                        Err(e) => {
                            if e.kind() != io::ErrorKind::UnexpectedEof {
                                warn!("Peer {} reader encountered an error, {}", self, Error::from(e));
                            }
                            break
                        }
                    }
                }
            }
        }

        trace!("Peer {} main reader loop ended", self);
        Self::send(self.id, &self.addr, &self.sender, PeerReaderEvent::Closed);
    }

    /// Try to read a specific number of bytes from the stream.
    ///
    /// # Arguments
    ///
    /// * `len` - The number of bytes to read from the stream.
    async fn read(&mut self, len: usize) -> Result<Vec<u8>> {
        let mut buffer = vec![0u8; len];
        let read_result = self.reader.read_exact(&mut buffer).await;

        match read_result {
            Ok(0) => Err(Error::Closed),
            Ok(_) => Ok(buffer),
            Err(e) => Err(Error::Io(e)),
        }
    }

    async fn read_next(&mut self, buffer: &[u8], buffer_size: usize) -> Result<()> {
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
        let id = self.id.clone();
        let addr = self.addr.clone();
        let sender = self.sender.clone();
        tokio::spawn(async move {
            match Message::try_from(bytes.as_ref()) {
                Ok(msg) => {
                    Self::send(
                        id,
                        &addr,
                        &sender,
                        PeerReaderEvent::Message(
                            msg,
                            DataTransferStats {
                                transferred_bytes: bytes.len(),
                                elapsed_micro: elapsed.as_micros(),
                            },
                        ),
                    );
                }
                Err(e) => warn!(
                    "Peer {}[{}] reader received invalid message payload, {}",
                    id, addr, e
                ),
            }
        });

        Ok(())
    }

    fn send(
        id: PeerId,
        addr: &SocketAddr,
        sender: &UnboundedSender<PeerReaderEvent>,
        event: PeerReaderEvent,
    ) {
        if let Err(e) = sender.send(event) {
            warn!("Peer {}[{}] reader failed to send event, {}", id, addr, e)
        }
    }
}
