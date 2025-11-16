use std::fmt::{Debug, Display, Formatter};
use std::io;
use std::io::{Cursor, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::Duration;

use crate::torrent::tracker::{
    AnnounceEntryResponse, AnnounceEvent, Announcement, ConnectionMetrics, ScrapeResult,
    TrackerClientConnection, TrackerHandle,
};
use crate::torrent::tracker::{Result, TrackerError};
use crate::torrent::InfoHash;
use async_trait::async_trait;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use derive_more::Display;
use itertools::Itertools;
use log::{debug, trace};
use tokio::net::UdpSocket;
use tokio::select;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tokio_util::bytes::Buf;
use tokio_util::sync::CancellationToken;

const ERROR_CONNECTION_NOT_INITIALIZED: &'static str = "udp connection not started";

/// The UDP connection of a tracker.
#[derive(Debug, Display)]
#[display(fmt = "{}", addrs)]
pub struct UdpConnection {
    /// The handle of the tracker
    handle: TrackerHandle,
    addrs: AddressManager,
    session: Option<UdpConnectionSession>,
    timeout: Duration,
    metrics: ConnectionMetrics,
    cancellation_token: CancellationToken,
}

#[async_trait]
impl TrackerClientConnection for UdpConnection {
    async fn start(&mut self) -> Result<()> {
        let socket = UdpSocket::bind("0:0").await?;
        let mut connected = false;

        // try to connect to an available address known for the tracker
        while let Some(addr) = self.next_addr().await {
            trace!("Trying to connect to {:?}", addr);
            match socket.connect(addr).await {
                Ok(_) => {
                    trace!("Successfully connected to tracker address {}", addr);
                    connected = true;
                    break;
                }
                Err(e) => {
                    debug!("Failed to connect to tracker address {}, {}", addr, e);
                }
            }
        }

        if connected {
            // generate a new transaction id on each connection attempt
            let transaction_id = Self::generate_transaction_id();
            Self::send_message(
                RequestMessage::Connect,
                0x41727101980, // the magical connection id constant, see BEP15
                transaction_id,
                &socket,
                &self.metrics,
                &self.cancellation_token,
            )
            .await?;
            // once we're able to send a successful message to the tracker
            // we'll store the socket into a valid session for further use
            self.session = Some(UdpConnectionSession {
                connection_id: Default::default(), // set the initial connection id to uninitialized (= 0)
                transaction_id,
                socket,
            });

            let response = self.read().await?;
            match response {
                ResponseMessage::Connection(response) => {
                    debug!("Received connect response {:?}", response);
                    let session = self.session.as_mut().unwrap();
                    // update the active connection session
                    session.connection_id = response.connection_id;
                    Ok(())
                }
                _ => {
                    // invalidate the active session as the connect request failed
                    self.session = None;
                    Err(TrackerError::Io(io::Error::new(
                        io::ErrorKind::Other,
                        format!(
                            "expected Response::Connection, but got {:?} instead",
                            response
                        ),
                    )))
                }
            }
        } else {
            Err(TrackerError::from(io::Error::from(
                io::ErrorKind::AddrNotAvailable,
            )))
        }
    }

    async fn announce(&self, announce: Announcement) -> Result<AnnounceEntryResponse> {
        self.do_announce(announce).await
    }

    async fn scrape(&self, hashes: &[InfoHash]) -> Result<ScrapeResult> {
        self.send(RequestMessage::Scrape(ScrapeRequest {
            hashes: hashes.to_vec(),
        }))
        .await?;
        let response = self.read().await?;
        match response {
            ResponseMessage::Scrape(response) => {
                trace!(
                    "Udp tracker {} is parsing scrape response {:?}",
                    self,
                    response
                );
                let mut result = ScrapeResult::default();
                for (index, response) in response.metrics.into_iter().enumerate() {
                    if let Some(hash) = hashes.get(index) {
                        result.files.insert(
                            hash.clone(),
                            crate::torrent::tracker::tracker::ScrapeFileMetrics {
                                complete: response.seeders,
                                incomplete: response.leechers,
                                downloaded: response.completed,
                            },
                        );
                    } else {
                        return Err(TrackerError::Parse(format!(
                            "Udp tracker {} scrape response exceeded {}/{} expected hashes",
                            self,
                            index,
                            hashes.len()
                        )));
                    }
                }
                Ok(result)
            }
            ResponseMessage::Error(e) => Err(TrackerError::AnnounceError(e)),
            _ => Err(TrackerError::Io(io::Error::new(
                io::ErrorKind::Other,
                format!("expected Response::Scrape, but got {:?} instead", response),
            ))),
        }
    }

    fn metrics(&self) -> &ConnectionMetrics {
        &self.metrics
    }

    fn close(&self) {
        trace!("Closing udp connection");
        self.cancellation_token.cancel();
    }
}

impl UdpConnection {
    pub fn new(handle: TrackerHandle, addrs: &[SocketAddr], timeout: Duration) -> Self {
        Self {
            handle,
            addrs: AddressManager::new(addrs),
            session: Default::default(),
            timeout,
            metrics: Default::default(),
            cancellation_token: Default::default(),
        }
    }

    /// Get the next available address of the tracker.
    ///
    /// # Returns
    ///
    /// It returns an address if one is available, else [None].
    async fn next_addr(&self) -> Option<&SocketAddr> {
        self.addrs.next_addr().await
    }

    /// Try to send the given request message to the tracker.
    /// This method can only be used if a [RequestMessage::Connect] has already been established.
    ///
    /// # Returns
    ///
    /// It returns an error if the request message couldn't be sent.
    async fn send(&self, message: RequestMessage) -> Result<()> {
        trace!(
            "Udp tracker {} is trying to send message {:?}",
            self,
            message
        );
        if let Some(session) = &self.session {
            Self::send_message(
                message,
                session.connection_id,
                session.transaction_id,
                &session.socket,
                &self.metrics,
                &self.cancellation_token,
            )
            .await
        } else {
            Err(TrackerError::Connection(
                ERROR_CONNECTION_NOT_INITIALIZED.to_string(),
            ))
        }
    }

    async fn read(&self) -> Result<ResponseMessage> {
        trace!("Trying to read udp message");
        let buffer = self.read_from_socket().await?;
        let mut cursor = Cursor::new(buffer[0..].to_vec());
        // read the action from the start of the cursor
        // after that, the cursor is delegated to the struct `TryFrom` handler
        let action: Action = cursor.read_u32::<BigEndian>()?.try_into()?;
        let transaction_id = cursor.read_u32::<BigEndian>()?;
        let response = UdpResponse {
            transaction_id,
            cursor,
        };

        trace!("Handling udp {} response", action);
        match action {
            Action::Connect => {
                ConnectResponse::try_from(response).map(|e| ResponseMessage::Connection(e))
            }
            Action::Announce => {
                AnnounceResponse::try_from(response).map(|e| ResponseMessage::Announce(e))
            }
            Action::Scrape => {
                ScrapeResponse::try_from(response).map(|e| ResponseMessage::Scrape(e))
            }
            Action::Error => {
                ErrorResponse::try_from(response).map(|e| ResponseMessage::Error(e.message))
            }
        }
    }

    /// Read the next incoming message buffer from the socket
    async fn read_from_socket(&self) -> Result<Vec<u8>> {
        if let Some(session) = &self.session {
            trace!(
                "Reading udp message from socket {:?}",
                session.socket.peer_addr()?
            );
            let mut buffer = vec![0; 16 * 1024];
            let buffer_size = timeout(self.timeout.clone(), session.socket.recv(&mut buffer))
                .await?
                .map_err(|e| {
                    self.metrics.timeouts.inc();
                    TrackerError::from(e)
                })?;

            // make sure we shrink the buffer to the expected size before returning
            self.metrics.bytes_in.inc_by(buffer_size as u64);
            Ok(buffer.into_iter().take(buffer_size).collect())
        } else {
            Err(TrackerError::Connection(
                ERROR_CONNECTION_NOT_INITIALIZED.to_string(),
            ))
        }
    }

    async fn do_announce(&self, announce: Announcement) -> Result<AnnounceEntryResponse> {
        let info_hash = announce.info_hash.short_info_hash_bytes();
        let event = announce.event;
        let request = AnnounceRequest {
            info_hash,
            peer_id: announce.peer_id.value(),
            downloaded: announce.bytes_completed,
            left: announce.bytes_remaining,
            corrupt: 0,
            uploaded: 0,
            event,
            ip_address: 0,
            key: 0,
            num_want: 200,
            redundant: 0,
            listen_port: announce.peer_port,
        };

        trace!(
            "Udp tracker {} is sending announce request {:?}",
            self,
            request
        );
        self.send(RequestMessage::Announce(request)).await?;
        let response = self.read().await?;
        match response {
            ResponseMessage::Announce(response) => {
                debug!(
                    "Udp tracker {} received announce response {:?}",
                    self, response
                );
                Ok(AnnounceEntryResponse {
                    interval_seconds: response.interval as u64,
                    leechers: response.leechers as u64,
                    seeders: response.seeders as u64,
                    peers: response.peers,
                })
            }
            ResponseMessage::Error(e) => Err(TrackerError::AnnounceError(e)),
            _ => Err(TrackerError::Io(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "expected Response::Announce, but got {:?} instead",
                    response
                ),
            ))),
        }
    }

    /// Try to send the given request message over the UDP socket to the tracker.
    /// This will write the message, together with the relevant info, as bytes over the given socket.
    ///
    /// # Returns
    ///
    /// It returns an error when the message couldn't be sent over the given socket.
    async fn send_message(
        message: RequestMessage,
        connection_id: u64,
        transaction_id: u32,
        socket: &UdpSocket,
        metrics: &ConnectionMetrics,
        cancellation_token: &CancellationToken,
    ) -> Result<()> {
        let mut buffer: Vec<u8> = Vec::new();
        let action = message.action();
        let message_bytes = TryInto::<Vec<u8>>::try_into(message)?;

        // write the connection id
        buffer.write_u64::<BigEndian>(connection_id)?;
        // write the action
        buffer.write_u32::<BigEndian>(action as u32)?;
        // write the transaction id
        buffer.write_u32::<BigEndian>(transaction_id)?;
        // write the message
        buffer.write_all(&message_bytes)?;

        trace!(
            "Udp tracker is sending a total of {} bytes to {:?}",
            buffer.len(),
            socket.peer_addr()?
        );
        metrics.bytes_out.inc_by(buffer.len() as u64);
        select! {
            _ = cancellation_token.cancelled() => Err(TrackerError::Connection("connection is being closed".to_string())),
            response = socket.send(buffer.as_ref()) => {
                let _ = response?;
                Ok(())
            },
        }
    }

    fn generate_transaction_id() -> u32 {
        // don't use 0, because that has special meaning (uninitialized)
        rand::random::<u32>() + 1
    }
}

#[derive(Debug)]
struct AddressManager {
    addr_cursor: Mutex<usize>,
    addrs: Vec<SocketAddr>,
}

impl AddressManager {
    pub fn new(addrs: &[SocketAddr]) -> Self {
        Self {
            addr_cursor: Default::default(),
            addrs: addrs.to_vec(),
        }
    }

    /// Get the next available address from the address manager.
    /// It returns [None] if there are no more addresses left.
    pub async fn next_addr(&self) -> Option<&SocketAddr> {
        let mut cursor = self.addr_cursor.lock().await;

        if self.addrs.is_empty() || *cursor >= self.addrs.len() {
            return None;
        }

        let addr = self.addrs.get(*cursor);
        *cursor += 1;
        addr
    }
}

impl Display for AddressManager {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.addrs)
    }
}

/// Contains the session information about an active udp connection.
#[derive(Debug)]
struct UdpConnectionSession {
    transaction_id: u32,
    connection_id: u64,
    socket: UdpSocket,
}

#[repr(u32)]
#[derive(Debug, Display, Clone)]
enum Action {
    #[display(fmt = "connect")]
    Connect = 0,
    #[display(fmt = "announce")]
    Announce = 1,
    #[display(fmt = "scrape")]
    Scrape = 2,
    #[display(fmt = "error")]
    Error = 3,
}

impl TryFrom<u32> for Action {
    type Error = TrackerError;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            0 => Ok(Action::Connect),
            1 => Ok(Action::Announce),
            2 => Ok(Action::Scrape),
            3 => Ok(Action::Error),
            _ => Err(TrackerError::from(io::Error::from(
                io::ErrorKind::InvalidData,
            ))),
        }
    }
}

/// The UDP request message to send to a tracker.
#[derive(Debug)]
enum RequestMessage {
    Connect,
    Announce(AnnounceRequest),
    Scrape(ScrapeRequest),
}

impl RequestMessage {
    /// Get the related action to this request message.
    /// It returns the relevant action of this request message.
    fn action(&self) -> Action {
        match self {
            RequestMessage::Connect => Action::Connect,
            RequestMessage::Announce(_) => Action::Announce,
            RequestMessage::Scrape(_) => Action::Scrape,
        }
    }
}

impl TryInto<Vec<u8>> for RequestMessage {
    type Error = TrackerError;

    fn try_into(self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        match self {
            RequestMessage::Announce(e) => {
                buffer.write_all(e.info_hash.as_ref())?;
                buffer.write_all(e.peer_id.as_ref())?;
                buffer.write_u64::<BigEndian>(e.downloaded)?;
                buffer.write_u64::<BigEndian>(e.left)?;
                buffer.write_u64::<BigEndian>(e.uploaded)?;
                buffer.write_u32::<BigEndian>(e.event as u32)?;
                buffer.write_u32::<BigEndian>(e.ip_address)?;
                buffer.write_u32::<BigEndian>(e.key)?;
                buffer.write_u32::<BigEndian>(e.num_want)?;
                buffer.write_u16::<BigEndian>(e.listen_port)?;
            }
            RequestMessage::Scrape(request) => {
                let bytes = request
                    .hashes
                    .into_iter()
                    .map(|e| e.short_info_hash_bytes())
                    .map(|e| e.to_vec())
                    .concat();

                buffer.write_all(bytes.as_slice())?;
            }
            _ => {}
        }

        Ok(buffer)
    }
}

/// The UDP response message received from a tracker.
#[derive(Debug)]
enum ResponseMessage {
    Connection(ConnectResponse),
    Announce(AnnounceResponse),
    Scrape(ScrapeResponse),
    Error(String),
}

#[derive(Debug)]
struct ConnectResponse {
    pub transaction_id: u32,
    pub connection_id: u64,
}

impl TryFrom<UdpResponse> for ConnectResponse {
    type Error = TrackerError;

    fn try_from(mut response: UdpResponse) -> Result<Self> {
        let connection_id = response.cursor.read_u64::<BigEndian>()?;

        Ok(Self {
            transaction_id: response.transaction_id,
            connection_id,
        })
    }
}

#[derive(Debug)]
struct AnnounceRequest {
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
    pub downloaded: u64,
    pub uploaded: u64,
    pub left: u64,
    pub corrupt: i64,
    pub redundant: i64,
    pub event: AnnounceEvent,
    pub ip_address: u32,
    pub key: u32,
    pub num_want: u32,
    pub listen_port: u16,
}

#[derive(Debug)]
struct AnnounceResponse {
    /// The interval in seconds between successive announcements
    pub interval: u32,
    /// The number of peers with incomplete downloads
    pub leechers: u32,
    /// The number of peers with complete downloads
    pub seeders: u32,
    /// The discovered peers address for the tracker
    pub peers: Vec<SocketAddr>,
}

impl TryFrom<UdpResponse> for AnnounceResponse {
    type Error = TrackerError;

    fn try_from(mut response: UdpResponse) -> Result<Self> {
        let interval = response.cursor.read_u32::<BigEndian>()?;
        let leechers = response.cursor.read_u32::<BigEndian>()?;
        let seeders = response.cursor.read_u32::<BigEndian>()?;

        let mut addrs = Vec::new();

        // we currently only support ipv4
        loop {
            if let Ok(ip) = response.cursor.read_u32::<BigEndian>() {
                let port = response.cursor.read_u16::<BigEndian>()?;
                addrs.push(SocketAddrV4::new(Ipv4Addr::from(ip), port).into());
            } else {
                break;
            }
        }

        Ok(Self {
            interval,
            leechers,
            seeders,
            peers: addrs,
        })
    }
}

#[derive(Debug)]
struct ErrorResponse {
    /// The error message returned by the tracker
    pub message: String,
}

impl TryFrom<UdpResponse> for ErrorResponse {
    type Error = TrackerError;

    fn try_from(mut response: UdpResponse) -> Result<Self> {
        let mut message = String::new();

        // try to read the error message
        response.cursor.read_to_string(&mut message)?;

        Ok(Self { message })
    }
}

#[derive(Debug)]
struct UdpResponse {
    pub transaction_id: u32,
    pub cursor: Cursor<Vec<u8>>,
}

#[derive(Debug)]
struct ScrapeRequest {
    hashes: Vec<InfoHash>,
}

#[derive(Debug, Default)]
struct ScrapeResponse {
    metrics: Vec<ScrapeFileMetrics>,
}

impl TryFrom<UdpResponse> for ScrapeResponse {
    type Error = TrackerError;

    fn try_from(mut response: UdpResponse) -> Result<Self> {
        let mut scrape_response = ScrapeResponse::default();

        while response.cursor.has_remaining() {
            let seeders = response.cursor.read_u32::<BigEndian>()?;
            let completed = response.cursor.read_u32::<BigEndian>()?;
            let leechers = response.cursor.read_u32::<BigEndian>()?;

            scrape_response.metrics.push(ScrapeFileMetrics {
                seeders,
                completed,
                leechers,
            });
        }

        Ok(scrape_response)
    }
}

#[derive(Debug)]
struct ScrapeFileMetrics {
    seeders: u32,
    completed: u32,
    leechers: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::torrent::peer::PeerId;
    use crate::torrent::tests::create_metadata;
    use crate::torrent::TorrentMetadata;

    use crate::init_logger;
    use tokio::net::lookup_host;

    #[test]
    fn test_generate_transaction_id() {
        let result = UdpConnection::generate_transaction_id();

        assert_ne!(
            0, result,
            "expected the transaction id to be greater than 0"
        );
    }

    #[tokio::test]
    async fn test_udp_connection_next_addr() {
        init_logger!();
        let tracker_handle = TrackerHandle::new();
        let socket_addr = (Ipv4Addr::UNSPECIFIED, 1599).try_into().unwrap();
        let connection = UdpConnection::new(tracker_handle, &[socket_addr], Duration::from_secs(1));

        let result = connection.next_addr().await;
        assert_eq!(Some(&socket_addr), result);

        let result = connection.next_addr().await;
        assert_eq!(None, result);
    }

    #[tokio::test]
    async fn test_udp_tracker_announce() {
        init_logger!();
        let torrent_info = create_metadata("debian-udp.torrent");
        let announce = Announcement {
            info_hash: torrent_info.info_hash.clone(),
            peer_id: PeerId::new(),
            peer_port: 6881,
            event: AnnounceEvent::Started,
            bytes_completed: 0,
            bytes_remaining: u64::MAX,
        };
        let mut connection = create_connection(&torrent_info).await;

        connection
            .start()
            .await
            .expect("expected the connection to start");

        let result = connection
            .announce(announce)
            .await
            .expect("expected the announce to succeed");
        assert_ne!(
            0, result.interval_seconds,
            "expected the interval to be greater than 0"
        );
        assert_ne!(
            0,
            result.peers.len(),
            "expected the number of peers to be greater than 0"
        );
    }

    #[tokio::test]
    async fn test_udp_tracker_scrape() {
        init_logger!();
        let torrent_info = create_metadata("debian-udp.torrent");
        let mut connection = create_connection(&torrent_info).await;

        connection
            .start()
            .await
            .expect("expected the connection to start");

        let result = connection
            .scrape(&vec![torrent_info.info_hash])
            .await
            .expect("expected the scrape to succeed");
        assert_eq!(
            1,
            result.files.len(),
            "expected the scrape metrics to match the torrent info"
        )
    }

    #[tokio::test]
    async fn test_address_manager_next_addr() {
        let addrs = vec![SocketAddr::from(([127, 0, 0, 1], 6881))];
        let manager = AddressManager::new(&addrs);

        let result = manager.next_addr().await;
        assert_ne!(None, result, "expected an address to be returned");

        let result = manager.next_addr().await;
        assert_eq!(None, result, "expected no address to be returned");
    }

    async fn create_connection(metadata: &TorrentMetadata) -> UdpConnection {
        let tracker_handle = TrackerHandle::new();
        let addrs = get_tracker_addresses(&metadata).await;

        UdpConnection::new(tracker_handle, &addrs, Duration::from_secs(1))
    }

    /// Get the unordered tracker addresses of the given torrent info.
    async fn get_tracker_addresses(torrent_info: &TorrentMetadata) -> Vec<SocketAddr> {
        let mut addresses = Vec::new();
        for url in torrent_info.trackers().into_iter() {
            let host = url.host_str().unwrap();
            let port = url.port().unwrap_or(80);

            if let Ok(e) = lookup_host((host, port))
                .await
                .map(|e| e.collect::<Vec<SocketAddr>>())
            {
                addresses.extend(e);
            }
        }
        addresses
    }
}
