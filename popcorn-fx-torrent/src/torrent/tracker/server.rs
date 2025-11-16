use crate::torrent::peer::PeerId;
use crate::torrent::tracker::http::HttpServer;
use crate::torrent::tracker::{
    AnnounceEntryResponse, AnnounceEvent, Announcement, ConnectionMetrics, Result,
    ScrapeFileMetrics, ScrapeResult, TrackerError, TrackerHandle,
};
use crate::torrent::InfoHash;
use async_trait::async_trait;
use derive_more::Display;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use log::debug;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::{oneshot, Mutex};
use tokio_util::sync::CancellationToken;

/// A request received by a tracker server connection.
///
/// This represents either an announce or scrape request made by a tracker client.
#[derive(Debug)]
pub enum ServerRequest {
    Announcement {
        addr: SocketAddr,
        request: Announcement,
        response: oneshot::Sender<AnnounceEntryResponse>,
    },
    Scrape {
        request: Vec<InfoHash>,
        response: oneshot::Sender<ScrapeResult>,
    },
}

/// The underlying listener which accepts incoming tracker requests.
///
/// Implementations are responsible for accepting incoming tracker
/// requests (announce/scrape) and exposing connection-related metrics.
#[async_trait]
pub trait TrackerListener: Send {
    /// Waits for the next incoming server request.
    ///
    /// Returns `None` when the connection is shutting down, and no further
    /// requests will be produced.
    async fn accept(&self) -> Option<ServerRequest>;

    /// Returns the local address on which this listener is listening.
    fn addr(&self) -> &SocketAddr;

    /// Returns metrics for this listener.
    fn metrics(&self) -> &ConnectionMetrics;

    /// Closes the listener and stops accepting new connections.
    fn close(&self);
}

/// A simple tracker server that can handle announce and scrape requests.
///
/// The server can listen on one or more underlying [`TrackerListener`]s
/// (e.g. HTTP, UDP) and maintains in-memory state about known torrents and peers.
#[derive(Debug, Clone)]
pub struct TrackerServer {
    inner: Arc<InnerServer>,
}

impl TrackerServer {
    /// Creates a new tracker server listening on an ephemeral port.
    ///
    /// Internally, this delegates to [`TrackerServer::with_port`] with `0`.
    pub async fn new() -> Result<Self> {
        Self::with_port(0).await
    }

    /// Creates a new tracker server listening on the specified `port`.
    pub async fn with_port(port: u16) -> Result<Self> {
        let server = HttpServer::with_port(port).await?;
        Self::with_listeners(vec![Box::new(server)]).await
    }

    /// Creates a new tracker server backed by the provided listeners.
    ///
    /// # Errors
    ///
    /// Returns [`TrackerError::Unavailable`] if `listeners` is empty.
    pub async fn with_listeners(listeners: Vec<Box<dyn TrackerListener>>) -> Result<Self> {
        let addr = if let Some(conn) = listeners.get(0) {
            conn.addr().clone()
        } else {
            return Err(TrackerError::Unavailable);
        };
        let inner = Arc::new(InnerServer {
            handle: Default::default(),
            addr,
            announce_interval: Duration::from_secs(5 * 60),
            torrents: Default::default(),
            cancellation: Default::default(),
        });

        let main_inner = inner.clone();
        tokio::spawn(async move {
            main_inner.start(listeners).await;
        });

        Ok(Self { inner })
    }

    /// Returns the socket address this server is listening on.
    pub fn addr(&self) -> &SocketAddr {
        &self.inner.addr
    }

    /// Adds or updates a peer in the tracker state for the given torrent.
    pub async fn add_peer(
        &self,
        info_hash: InfoHash,
        addr: SocketAddr,
        peer_id: PeerId,
        peer_port: u16,
        completed: bool,
    ) {
        let event = if completed {
            AnnounceEvent::Completed
        } else {
            AnnounceEvent::None
        };

        let _ = self
            .inner
            .handle_announcement(
                addr,
                Announcement {
                    info_hash,
                    peer_id,
                    peer_port,
                    event,
                    bytes_completed: 0,
                    bytes_remaining: 0,
                },
            )
            .await;
    }
}

#[derive(Debug, Display)]
#[display(fmt = "{}", handle)]
struct InnerServer {
    handle: TrackerHandle,
    addr: SocketAddr,
    announce_interval: Duration,
    torrents: Mutex<HashMap<InfoHash, HashMap<PeerEntry, TorrentPeer>>>,
    cancellation: CancellationToken,
}

impl InnerServer {
    async fn start(&self, listeners: Vec<Box<dyn TrackerListener>>) {
        loop {
            let mut futures =
                FuturesUnordered::from_iter(listeners.iter().map(|e| e.accept())).fuse();
            select! {
                _ = self.cancellation.cancelled() => break,
                Some(Some(request)) = futures.next() => self.handle_request(request).await,
            }
        }

        listeners.into_iter().for_each(|e| e.close());
        debug!("Tracker server {} main loop ended", self);
    }

    async fn handle_request(&self, request: ServerRequest) {
        match request {
            ServerRequest::Announcement {
                addr,
                request,
                response,
            } => {
                let _ = response.send(self.handle_announcement(addr, request).await);
            }
            ServerRequest::Scrape {
                request, response, ..
            } => {
                let _ = response.send(self.handle_scrape(request).await);
            }
        }
    }

    async fn handle_announcement(
        &self,
        addr: SocketAddr,
        announcement: Announcement,
    ) -> AnnounceEntryResponse {
        let mut torrents = self.torrents.lock().await;
        let torrent_peers = torrents.entry(announcement.info_hash.clone()).or_default();
        let entry = PeerEntry::new(announcement.peer_id, addr.ip());

        // update or remove the entry within the tracked torrents
        if announcement.event == AnnounceEvent::Stopped {
            torrent_peers.remove(&entry);
        } else {
            let torrent_peer = torrent_peers.entry(entry).or_insert(TorrentPeer {
                peer_port: announcement.peer_port,
                bytes_completed: announcement.bytes_completed,
                bytes_remaining: announcement.bytes_remaining,
                completed: false,
            });

            torrent_peer.completed = Self::is_torrent_peer_completed(&announcement);
            torrent_peer.bytes_completed = announcement.bytes_completed;
            torrent_peer.bytes_remaining = announcement.bytes_remaining;
        }

        // retrieve all peers except for the addr
        let mut leechers = 0;
        let mut seeders = 0;
        let mut peers = Vec::new();

        for (entry, torrent_peer) in torrent_peers
            .iter()
            .filter(|(entry, _)| entry.id != announcement.peer_id)
        {
            if torrent_peer.completed {
                seeders += 1;
            } else {
                leechers += 1;
            }

            peers.push((entry.ip, torrent_peer.peer_port).into());
        }

        AnnounceEntryResponse {
            interval_seconds: self.announce_interval.as_secs(),
            leechers,
            seeders,
            peers,
        }
    }

    async fn handle_scrape(&self, hashes: Vec<InfoHash>) -> ScrapeResult {
        let torrents = self.torrents.lock().await;

        ScrapeResult {
            files: hashes
                .into_iter()
                .map(|hash| {
                    if let Some(peers) = torrents.get(&hash) {
                        let mut metrics = ScrapeFileMetrics::default();

                        for (_, peer) in peers {
                            if peer.completed {
                                metrics.complete += 1;
                                metrics.downloaded += 1;
                            } else {
                                metrics.incomplete += 1;
                            }
                        }

                        (hash, metrics)
                    } else {
                        (hash, ScrapeFileMetrics::default())
                    }
                })
                .collect(),
        }
    }

    fn is_torrent_peer_completed(announcement: &Announcement) -> bool {
        announcement.event == AnnounceEvent::Completed
            || (announcement.bytes_remaining == 0 && announcement.bytes_completed != 0)
    }
}

/// A unique peer entry within the torrent tracker server.
/// Each announcement is always indexed based on the peer and ip address to prevent spoofing of peers.
#[derive(Debug, PartialEq, Eq, Hash)]
struct PeerEntry {
    id: PeerId,
    ip: IpAddr,
}

impl PeerEntry {
    fn new(id: PeerId, ip: IpAddr) -> Self {
        Self { id, ip }
    }
}

#[derive(Debug)]
struct TorrentPeer {
    peer_port: u16,
    bytes_completed: u64,
    bytes_remaining: u64,
    completed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_logger;
    use std::net::Ipv4Addr;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_handle_announcement_started() {
        init_logger!();
        let info_hash = InfoHash::from_str("EADAF0EFEA39406914414D359E0EA16416409BD7").unwrap();
        let peer_id = PeerId::new();
        let peer_port = 6881;
        let ip = Ipv4Addr::LOCALHOST;
        let expected_peer_entry = PeerEntry::new(peer_id.clone(), IpAddr::V4(ip.clone()));
        let server = TrackerServer::new().await.unwrap();

        server
            .inner
            .handle_announcement(
                (Ipv4Addr::LOCALHOST, 8000).into(),
                Announcement {
                    info_hash: info_hash.clone(),
                    peer_id,
                    peer_port,
                    event: AnnounceEvent::Started,
                    bytes_completed: 0,
                    bytes_remaining: 0,
                },
            )
            .await;

        let torrents = server.inner.torrents.lock().await;
        let torrent = torrents
            .get(&info_hash)
            .expect("expected the torrent entry to have been stored");
        let result = torrent
            .get(&expected_peer_entry)
            .expect("expected the peer entry to have been stored");

        assert_eq!(
            peer_port, result.peer_port,
            "expected the peer port to match the announcement"
        );
        assert_eq!(
            false, result.completed,
            "expected the torrent to not have been completed"
        );
    }

    #[tokio::test]
    async fn test_handle_announcement_completed() {
        init_logger!();
        let info_hash = InfoHash::from_str("EADAF0EFEA39406914414D359E0EA16416409BD7").unwrap();
        let peer_id = PeerId::new();
        let ip = Ipv4Addr::LOCALHOST;
        let expected_peer_entry = PeerEntry::new(peer_id.clone(), IpAddr::V4(ip.clone()));
        let server = TrackerServer::new().await.unwrap();

        server
            .inner
            .handle_announcement(
                (Ipv4Addr::LOCALHOST, 8000).into(),
                Announcement {
                    info_hash: info_hash.clone(),
                    peer_id,
                    peer_port: 6881,
                    event: AnnounceEvent::Completed,
                    bytes_completed: 0,
                    bytes_remaining: 0,
                },
            )
            .await;

        let torrents = server.inner.torrents.lock().await;
        let torrent = torrents
            .get(&info_hash)
            .expect("expected the torrent entry to have been stored");
        let result = torrent
            .get(&expected_peer_entry)
            .expect("expected the peer entry to have been stored");

        assert_eq!(
            true, result.completed,
            "expected the torrent to not have been completed"
        );
    }

    #[tokio::test]
    async fn test_handle_scrape() {
        init_logger!();
        let info_hash = InfoHash::from_str("EADAF0EFEA39406914414D359E0EA16416409BD7").unwrap();
        let peer_id = PeerId::new();
        let expected_result = ScrapeResult {
            files: vec![(
                info_hash.clone(),
                ScrapeFileMetrics {
                    complete: 0,
                    incomplete: 1,
                    downloaded: 0,
                },
            )]
            .into_iter()
            .collect(),
        };
        let server = TrackerServer::new().await.unwrap();

        // add dummy
        server
            .add_peer(
                info_hash.clone(),
                (Ipv4Addr::LOCALHOST, 8000).into(),
                peer_id.clone(),
                6881,
                false,
            )
            .await;

        let result = server.inner.handle_scrape(vec![info_hash.clone()]).await;

        assert_eq!(expected_result, result);
    }
}
