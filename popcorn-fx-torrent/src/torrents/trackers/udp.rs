use std::fmt::Debug;
use std::io;
use std::io::{Cursor, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::Duration;

use crate::torrents::peers::PeerId;
use crate::torrents::trackers::manager::AnnounceEvent;
use crate::torrents::trackers::{AnnounceEntryResponse, TrackerConnection};
use crate::torrents::trackers::{Result, TrackerError};
use crate::torrents::InfoHash;
use async_trait::async_trait;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use derive_more::Display;
use log::{debug, trace};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

const ERROR_CONNECTION_NOT_INITIALIZED: &'static str = "udp connection not started";

#[derive(Debug)]
pub struct UdpConnection {
    peer_id: PeerId,
    addrs: AddressManager,
    session: Option<UdpConnectionSession>,
    timeout: Duration,
    cancel: CancellationToken,
}

#[async_trait]
impl TrackerConnection for UdpConnection {
    async fn start(&mut self) -> Result<()> {
        let socket = UdpSocket::bind("0:0").await?;
        let mut connected = false;

        // try to connect to an available address known for the tracker
        while let Some(addr) = self.next_addr().await {
            trace!("Trying to connect to {:?}", addr);
            match socket.connect(addr).await {
                Ok(_) => {
                    debug!("Successfully connected to tracker address {}", addr);
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
            let connect_request = ConnectRequest::new(transaction_id);
            Self::send_with_socket(
                Into::<Vec<u8>>::into(connect_request),
                &socket,
                &self.cancel,
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
                Response::Connection(response) => {
                    debug!("Received connect response {:?}", response);
                    let session = self.session.as_mut().unwrap();
                    // update the active connection session
                    session.connection_id = response.connection_id;
                    Ok(())
                }
                _ => {
                    // invalidate the active session as the connect request failed
                    self.session = None;
                    Err(TrackerError::Io(format!(
                        "expected Response::Connection, but got {:?} instead",
                        response
                    )))
                }
            }
        } else {
            Err(TrackerError::from(io::Error::from(
                io::ErrorKind::AddrNotAvailable,
            )))
        }
    }

    async fn announce(
        &self,
        info_hash: InfoHash,
        event: AnnounceEvent,
    ) -> Result<AnnounceEntryResponse> {
        self.do_announce(info_hash, event).await
    }

    fn close(&mut self) {
        trace!("Closing udp connection");
        self.cancel.cancel();
    }
}

impl UdpConnection {
    pub fn new(addrs: &[SocketAddr], peer_id: PeerId, timeout: Duration) -> Self {
        Self {
            peer_id,
            addrs: AddressManager::new(addrs),
            session: Default::default(),
            timeout,
            cancel: Default::default(),
        }
    }

    async fn next_addr(&self) -> Option<&SocketAddr> {
        self.addrs.next_addr().await
    }

    async fn send<T>(&self, message: T) -> Result<()>
    where
        T: AsRef<[u8]>,
    {
        trace!("Trying to send udp message");
        if let Some(session) = &self.session {
            Self::send_with_socket(message, &session.socket, &self.cancel).await
        } else {
            Err(TrackerError::Connection(
                ERROR_CONNECTION_NOT_INITIALIZED.to_string(),
            ))
        }
    }

    async fn read(&self) -> Result<Response> {
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
            Action::Connect => ConnectResponse::try_from(response).map(|e| Response::Connection(e)),
            Action::Announce => AnnounceResponse::try_from(response).map(|e| Response::Announce(e)),
            _ => todo!(),
        }
    }

    /// Read the next incoming message buffer from the socket
    async fn read_from_socket(&self) -> Result<Vec<u8>> {
        if let Some(session) = &self.session {
            trace!(
                "Reading udp message from socket {:?}",
                session.socket.peer_addr().unwrap()
            );
            let mut buffer = vec![0; 16 * 1024];
            let buffer_size = timeout(self.timeout.clone(), session.socket.recv(&mut buffer))
                .await?
                .map_err(|e| TrackerError::from(e))?;

            // make sure we shrink the buffer to the expected size before returning
            Ok(buffer.into_iter().take(buffer_size).collect())
        } else {
            Err(TrackerError::Connection(
                ERROR_CONNECTION_NOT_INITIALIZED.to_string(),
            ))
        }
    }

    async fn do_announce(
        &self,
        info_hash: InfoHash,
        event: AnnounceEvent,
    ) -> Result<AnnounceEntryResponse> {
        if let Some(session) = &self.session {
            let request = AnnounceRequest {
                transaction_id: session.transaction_id,
                connection_id: session.connection_id,
                info_hash: info_hash.short_info_hash_bytes(),
                peer_id: self.peer_id.value(),
                downloaded: 0,
                left: u64::MAX,
                corrupt: 0,
                uploaded: 0,
                event,
                ip_address: 0,
                key: 0,
                num_want: 100,
                redundant: 0,
                listen_port: 6881,
            };

            trace!("Sending announce request {:?}", request);
            self.send(Into::<Vec<u8>>::into(request)).await?;
            let response = self.read().await?;
            return match response {
                Response::Announce(response) => {
                    debug!("Received announce response {:?}", response);
                    Ok(AnnounceEntryResponse {
                        interval_seconds: response.interval as u64,
                        leechers: response.leechers as u64,
                        seeders: response.seeders as u64,
                        peers: response.peers,
                    })
                }
                _ => Err(TrackerError::Io(format!(
                    "expected Response::Announce, but got {:?} instead",
                    response
                ))),
            };
        }

        return Err(TrackerError::Connection(
            ERROR_CONNECTION_NOT_INITIALIZED.to_string(),
        ));
    }

    async fn send_with_socket<T>(
        message: T,
        socket: &UdpSocket,
        cancellation_token: &CancellationToken,
    ) -> Result<()>
    where
        T: AsRef<[u8]>,
    {
        trace!("Sending udp message to {:?}", socket.peer_addr().unwrap());
        tokio::select! {
            _ = cancellation_token.cancelled() => Err(TrackerError::Connection("connection is being closed".to_string())),
            response = socket.send(message.as_ref()) => {
                let size = response?;
                trace!("Send a total of {} bytes", size);
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

#[derive(Debug)]
enum Response {
    Connection(ConnectResponse),
    Announce(AnnounceResponse),
    Scrape,
}

#[derive(Debug)]
struct ConnectRequest {
    pub transaction_id: u32,
}

impl ConnectRequest {
    pub fn new(transaction_id: u32) -> Self {
        Self { transaction_id }
    }
}

impl Into<Vec<u8>> for ConnectRequest {
    fn into(self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.write_u32::<BigEndian>(0x0417).unwrap();
        buffer.write_u32::<BigEndian>(0x27101980).unwrap(); // connection_id
        buffer
            .write_u32::<BigEndian>(Action::Connect as u32)
            .unwrap();
        buffer.write_u32::<BigEndian>(self.transaction_id).unwrap();
        buffer
    }
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
    pub transaction_id: u32,
    pub connection_id: u64,
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

impl Into<Vec<u8>> for AnnounceRequest {
    fn into(self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.write_u64::<BigEndian>(self.connection_id).unwrap();
        buffer
            .write_u32::<BigEndian>(Action::Announce as u32)
            .unwrap();
        buffer.write_u32::<BigEndian>(self.transaction_id).unwrap();
        buffer.write_all(self.info_hash.as_ref()).unwrap();
        buffer.write_all(self.peer_id.as_ref()).unwrap();
        buffer.write_u64::<BigEndian>(self.downloaded).unwrap();
        buffer.write_u64::<BigEndian>(self.left).unwrap();
        buffer.write_u64::<BigEndian>(self.uploaded).unwrap();
        buffer.write_u32::<BigEndian>(self.event as u32).unwrap();
        buffer.write_u32::<BigEndian>(self.ip_address).unwrap();
        buffer.write_u32::<BigEndian>(self.key).unwrap();
        buffer.write_u32::<BigEndian>(self.num_want).unwrap();
        buffer.write_u16::<BigEndian>(self.listen_port).unwrap();
        buffer
    }
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
struct UdpResponse {
    pub transaction_id: u32,
    pub cursor: Cursor<Vec<u8>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torrents::TorrentInfo;
    use popcorn_fx_core::core::block_in_place;
    use popcorn_fx_core::testing::{init_logger, read_test_file_to_bytes};
    use tokio::net::lookup_host;
    use tokio::runtime::Runtime;

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
        init_logger();
        let socket_addr = ([127, 0, 0, 1], 1599).try_into().unwrap();
        let peer_id = PeerId::new();
        let connection = UdpConnection::new(&[socket_addr], peer_id, Duration::from_secs(1));

        let result = connection.next_addr().await;
        assert_eq!(Some(&socket_addr), result);

        let result = connection.next_addr().await;
        assert_eq!(None, result);
    }

    #[test]
    fn test_announce() {
        init_logger();
        let runtime = Runtime::new().expect("expected a runtime");
        let torrent_info_data = read_test_file_to_bytes("debian-udp.torrent");
        let torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
        let peer_id = PeerId::new();
        let addrs: Vec<SocketAddr> = get_tracker_addresses(&torrent_info);
        let mut connection = UdpConnection::new(&addrs, peer_id, Duration::from_secs(1));

        runtime
            .block_on(connection.start())
            .expect("expected the connection to start");
        let result = runtime
            .block_on(connection.announce(torrent_info.info_hash, AnnounceEvent::Started))
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
    async fn test_address_manager_next_addr() {
        let addrs = vec![SocketAddr::from(([127, 0, 0, 1], 6881))];
        let manager = AddressManager::new(&addrs);

        let result = manager.next_addr().await;
        assert_ne!(None, result, "expected an address to be returned");

        let result = manager.next_addr().await;
        assert_eq!(None, result, "expected no address to be returned");
    }

    /// Get the unordered tracker addresses of the given torrent info.
    fn get_tracker_addresses(torrent_info: &TorrentInfo) -> Vec<SocketAddr> {
        torrent_info
            .trackers()
            .into_iter()
            .filter_map(|url| {
                let host = url.host_str().unwrap();
                let port = url.port().unwrap_or(80);

                block_in_place(lookup_host((host, port)))
                    .map(|e| e.collect::<Vec<SocketAddr>>())
                    .ok()
            })
            .flatten()
            .collect()
    }
}
