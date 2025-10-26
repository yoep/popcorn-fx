use crate::torrent::peer::protocol::{Handshake, Message, UtpStream};
use crate::torrent::peer::{
    ConnectionProtocol, Error, Metrics, PeerConn, PeerId, PeerResponse, Result,
};
use async_trait::async_trait;
use byteorder::BigEndian;
use byteorder::ByteOrder;
use derive_more::Display;
use log::{debug, trace, warn};
use std::fmt::Debug;
use std::io;
use std::net::SocketAddr;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, WriteHalf};
use tokio::net::TcpStream;
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
    /// The underlying protocol being used within the connection
    protocol: ConnectionProtocol,
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
        metrics: Metrics,
    ) -> PeerConnection<TcpStream> {
        let protocol = ConnectionProtocol::Tcp;
        let cancellation_token = CancellationToken::new();
        let (sender, receiver) = unbounded_channel();
        let (reader, writer) = tokio::io::split(stream);

        let mut reader = PeerReader::new(
            id,
            addr,
            protocol,
            reader,
            sender,
            metrics,
            cancellation_token.clone(),
        );
        tokio::spawn(async move { reader.start_read_loop().await });

        PeerConnection::<TcpStream> {
            id,
            addr,
            protocol,
            receiver: Mutex::new(receiver),
            writer: PeerWriter::new(writer),
            cancellation_token,
        }
    }

    pub fn new_utp(
        id: PeerId,
        addr: SocketAddr,
        stream: UtpStream,
        metrics: Metrics,
    ) -> PeerConnection<UtpStream> {
        let protocol = ConnectionProtocol::Utp;
        let cancellation_token = CancellationToken::new();
        let (sender, receiver) = unbounded_channel();
        let (reader, writer) = tokio::io::split(stream);

        let mut reader = PeerReader::new(
            id,
            addr,
            protocol,
            reader,
            sender,
            metrics,
            cancellation_token.clone(),
        );
        tokio::spawn(async move { reader.start_read_loop().await });

        PeerConnection::<UtpStream> {
            id,
            addr,
            protocol,
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
    fn protocol(&self) -> ConnectionProtocol {
        self.protocol
    }

    async fn recv(&self) -> Option<PeerResponse> {
        if let Some(event) = self.receiver.lock().await.recv().await {
            return Some(match event {
                PeerReaderEvent::Handshake(bytes) => {
                    match Handshake::from_bytes(&self.addr, bytes.as_slice()) {
                        Ok(e) => PeerResponse::Handshake(e),
                        Err(e) => PeerResponse::Error(e),
                    }
                }
                PeerReaderEvent::HandshakeFailed(e) => PeerResponse::Error(e),
                PeerReaderEvent::Message(message) => PeerResponse::Message(message),
                PeerReaderEvent::Closed => {
                    let _ = self.close().await;
                    PeerResponse::Closed
                }
            });
        }

        None
    }

    async fn write<'a>(&'a self, bytes: &'a [u8]) -> Result<()> {
        if self.cancellation_token.is_cancelled() {
            return Err(Error::Closed);
        }

        // make sure that we interrupt any writing operations if the connection is forcefully closed
        select! {
            _ = self.cancellation_token.cancelled() => Err(Error::Closed),
            result = self.writer.write(bytes) => result,
        }
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
#[derive(Debug)]
enum PeerReaderEvent {
    /// Received a handshake bytes from the remote peer.
    Handshake(Vec<u8>),
    /// Handshake failed to be read from the remote peer.
    HandshakeFailed(Error),
    /// Received a message from the remote peer.
    Message(Message),
    /// The connection was closed by the remote peer.
    Closed,
}

/// The peer reader is a buffered reader which reads messages from the peer connection stream.
#[derive(Debug, Display)]
#[display(fmt = "{}[{}:{}]", id, protocol, addr)]
struct PeerReader<R>
where
    R: AsyncRead + Unpin,
{
    id: PeerId,
    addr: SocketAddr,
    protocol: ConnectionProtocol,
    reader: BufReader<R>,
    sender: UnboundedSender<PeerReaderEvent>,
    metrics: Metrics,
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
        protocol: ConnectionProtocol,
        reader: R,
        sender: UnboundedSender<PeerReaderEvent>,
        metrics: Metrics,
        cancellation_token: CancellationToken,
    ) -> Self {
        Self {
            id,
            addr,
            protocol,
            reader: BufReader::new(reader),
            sender,
            metrics,
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
                            self.to_string().as_str(),
                            &self.sender,
                            PeerReaderEvent::Handshake(buffer)
                        );
                    }
                    Err(e) => {
                        debug!("Peer {} failed to read handshake, {}", self, e);
                        Self::send (
                            self.to_string().as_str(),
                            &self.sender,
                            PeerReaderEvent::HandshakeFailed(e),
                        );
                        return;
                    },
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
        let self_info = self.to_string();
        Self::send(self_info.as_str(), &self.sender, PeerReaderEvent::Closed);
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
            Ok(_) => {
                self.metrics.bytes_in.inc_by(len as u64);
                Ok(buffer)
            }
            Err(e) => Err(Error::Io(e)),
        }
    }

    async fn read_next(&mut self, buffer: &[u8], buffer_size: usize) -> Result<()> {
        // we expect to receive the incoming message length as a BigEndian
        if buffer_size != 4 {
            return Err(Error::InvalidLength(4, buffer_size as u32));
        }

        let len = BigEndian::read_u32(buffer);
        let bytes = self.read(len as usize).await?;

        // we want to unblock the reader thread as soon as possible
        // so we're going to move this whole process into a new separate thread
        let self_info = self.to_string();
        let sender = self.sender.clone();
        tokio::spawn(async move {
            match Message::try_from(bytes.as_ref()) {
                Ok(msg) => {
                    Self::send(self_info.as_str(), &sender, PeerReaderEvent::Message(msg));
                }
                Err(e) => warn!(
                    "Peer {} reader received invalid message payload, {}",
                    self_info, e
                ),
            }
        });

        Ok(())
    }

    fn send(self_info: &str, sender: &UnboundedSender<PeerReaderEvent>, event: PeerReaderEvent) {
        if let Err(e) = sender.send(event) {
            warn!("Peer {} reader failed to send event, {}", self_info, e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::create_utp_socket_pair;
    use crate::init_logger;
    use crate::torrent::peer::protocol::tests::UtpPacketCaptureExtension;
    use crate::torrent::peer::protocol::Piece;
    use crate::torrent::peer::tests::create_utp_stream_pair;
    use crate::torrent::peer::ProtocolExtensionFlags;
    use crate::torrent::InfoHash;

    use std::net::Ipv4Addr;
    use std::str::FromStr;
    use tokio::net::TcpListener;

    mod utp {
        use super::*;
        use crate::timeout;
        use std::time::Duration;

        #[tokio::test]
        async fn test_peer_connection_receive() {
            init_logger!();
            let data = "Mauris venenatis malesuada tellus vel imperdiet. Pellentesque quis blandit tellus. Aenean commodo neque id sem dictum aliquam at vel arcu.";
            let hash = "urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7";
            let info_hash = InfoHash::from_str(hash).unwrap();
            let peer_id = PeerId::new();
            let protocol_extension_flags = ProtocolExtensionFlags::LTEP;
            let incoming_capture = UtpPacketCaptureExtension::new();
            let outgoing_capture = UtpPacketCaptureExtension::new();
            let (incoming_socket, outgoing_socket) = create_utp_socket_pair!(
                vec![Box::new(incoming_capture.clone())],
                vec![Box::new(outgoing_capture.clone())]
            );
            let (incoming_stream, mut outgoing_stream) =
                create_utp_stream_pair(&incoming_socket, &outgoing_socket).await;
            let connection = PeerConnection::<UtpStream>::new_utp(
                peer_id,
                incoming_stream.addr(),
                incoming_stream,
                Metrics::new(),
            );

            // write the handshake to the receiving connection
            let handshake = Handshake::new(info_hash.clone(), peer_id, protocol_extension_flags);
            let handshake_bytes = TryInto::<Vec<u8>>::try_into(handshake).unwrap();
            outgoing_stream.write_all(&handshake_bytes).await.unwrap();
            outgoing_stream.flush().await.unwrap();

            // try to get the handshake from the receiving stream
            let result = connection
                .recv()
                .await
                .expect("expected to receive the handshake");
            if let PeerResponse::Handshake(result) = result {
                assert_eq!(
                    info_hash, result.info_hash,
                    "expected the info hash to match"
                );
                assert_eq!(peer_id, result.peer_id, "expected the peer id to match");
                assert_eq!(
                    protocol_extension_flags, result.supported_extensions,
                    "expected the supported protocol extensions to match"
                );
            } else {
                assert!(
                    false,
                    "expected PeerResponse::Handshake, but got {:?} instead",
                    result
                );
            }

            // write some random data to the receiving connection
            let message = Message::Piece(Piece {
                index: 0,
                begin: 0,
                data: data.as_bytes().to_vec(),
            });
            let bytes = message_as_bytes(&message);
            outgoing_stream.write_all(&bytes).await.unwrap();
            outgoing_stream.flush().await.unwrap();

            // try to read the message from the receiving stream
            let result = connection
                .recv()
                .await
                .expect("expected to receive the message");
            if let PeerResponse::Message(result) = result {
                assert_eq!(message, result, "expected the message to match");
            } else {
                assert!(
                    false,
                    "expected PeerResponse::Message, but got {:?} instead",
                    result
                );
            }
        }

        #[tokio::test]
        async fn test_peer_connection_write() {
            init_logger!();
            let info_hash =
                InfoHash::from_str("urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7").unwrap();
            let peer_id = PeerId::new();
            let protocol_extensions = ProtocolExtensionFlags::Fast;
            let handshake = Handshake::new(info_hash.clone(), peer_id, protocol_extensions);
            let handshake_bytes = TryInto::<Vec<u8>>::try_into(handshake).unwrap();
            let (incoming_socket, outgoing_socket) = create_utp_socket_pair!();
            let (mut incoming_stream, outgoing_stream) =
                create_utp_stream_pair(&incoming_socket, &outgoing_socket).await;
            let outgoing_stream_addr = outgoing_stream.addr();
            let connection = PeerConnection::<UtpStream>::new_utp(
                peer_id,
                outgoing_stream_addr,
                outgoing_stream,
                Metrics::new(),
            );

            connection
                .write(handshake_bytes.as_slice())
                .await
                .expect("expected the handshake to have been written");

            let mut buffer = vec![0u8; HANDSHAKE_MESSAGE_LENGTH];
            let _ = timeout!(
                incoming_stream.read_exact(&mut buffer),
                Duration::from_millis(500),
                "expected to have received the handshake bytes"
            )
            .unwrap();

            let result = Handshake::from_bytes(&outgoing_stream_addr, buffer.as_slice()).unwrap();
            assert_eq!(
                info_hash, result.info_hash,
                "expected the info hash to match"
            );
            assert_eq!(peer_id, result.peer_id, "expected the peer id to match");
            assert_eq!(
                protocol_extensions, result.supported_extensions,
                "expected the supported protocol extensions to match"
            );
        }
    }

    mod peer_reader {
        use super::*;

        use crate::timeout;

        use std::pin::Pin;
        use std::task::{Context, Poll};
        use tokio::io::ReadBuf;
        use tokio::time;
        use tokio::time::Duration;

        struct HandshakeFailureReader;

        impl AsyncRead for HandshakeFailureReader {
            fn poll_read(
                self: Pin<&mut Self>,
                _: &mut Context<'_>,
                _: &mut ReadBuf<'_>,
            ) -> Poll<io::Result<()>> {
                Poll::Ready(Err(io::Error::new(
                    io::ErrorKind::ConnectionReset,
                    "handshake failed",
                )))
            }
        }

        #[tokio::test]
        async fn test_handshake_failure() {
            init_logger!();
            let (sender, mut rx) = unbounded_channel();
            let mut reader = PeerReader::new(
                PeerId::new(),
                SocketAddr::from(([127, 0, 0, 1], 6881)),
                ConnectionProtocol::Tcp,
                HandshakeFailureReader {},
                sender,
                Metrics::new(),
                CancellationToken::new(),
            );

            let _ = select! {
                _ = time::sleep(Duration::from_millis(250)) => Err(Error::Io(io::Error::new(io::ErrorKind::TimedOut, "expected the reader main loop to have ended"))),
                _ = reader.start_read_loop() => Ok(()),
            }.unwrap();

            let result = timeout!(
                rx.recv(),
                Duration::from_millis(500),
                "expected a peer reader event"
            )
            .unwrap();
            if let PeerReaderEvent::HandshakeFailed(_) = result {
            } else {
                assert!(
                    false,
                    "expected PeerReaderEvent::HandshakeFailed, but got {:?} instead",
                    result
                )
            }
        }
    }

    #[tokio::test]
    async fn test_peer_connection_shutdown() {
        init_logger!();
        let message = "Lorem ipsum dolor";
        let socket_addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0));
        let incoming = TcpListener::bind(socket_addr)
            .await
            .expect("expected the tcp listener to bind");
        let incoming_port = incoming.local_addr().unwrap().port();

        tokio::spawn(async move { while let Ok((_stream, _addr)) = incoming.accept().await {} });

        let outgoing_stream = TcpStream::connect((socket_addr.ip(), incoming_port))
            .await
            .expect("expected to create an outgoing connection");
        let connection = PeerConnection::<TcpStream>::new_tcp(
            PeerId::new(),
            socket_addr,
            outgoing_stream,
            Metrics::new(),
        );

        connection
            .close()
            .await
            .expect("expected the connection to close");
        let result = connection.write(message.as_bytes()).await;

        assert_eq!(
            Err(Error::Closed),
            result,
            "expected the connection write function to return Error::Closed"
        );
    }

    fn message_as_bytes(message: &Message) -> Vec<u8> {
        let mut bytes = vec![0u8; 4];
        let message_bytes = TryInto::<Vec<u8>>::try_into(message.clone()).unwrap();

        BigEndian::write_u32(&mut bytes[..4], message_bytes.len() as u32);
        bytes.extend_from_slice(message_bytes.as_slice());

        bytes
    }
}
