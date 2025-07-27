use crate::torrent::dns::DnsResolver;
use crate::torrent::peer::PeerId;
use crate::torrent::tracker::http::HttpConnection;
use crate::torrent::tracker::udp::UdpConnection;
use crate::torrent::tracker::{AnnounceEvent, Result, TrackerError};
use crate::torrent::InfoHash;
use async_trait::async_trait;
use derive_more::Display;
use fx_handle::Handle;
use log::{debug, trace};
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::ops::Sub;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::{select, time};
use url::Url;

const DEFAULT_CONNECTION_TIMEOUT_SECONDS: u64 = 10;
const DEFAULT_ANNOUNCEMENT_INTERVAL_SECONDS: u64 = 120;

/// The announcement information for a tracker.
/// This is the most recent torrent information that should be shared with the tracker.
#[derive(Debug, Clone)]
pub struct Announcement {
    /// The info hash of the torrent
    pub info_hash: InfoHash,
    /// The tracker announcement event
    pub event: AnnounceEvent,
    /// The number of piece bytes completed by the torrent
    pub bytes_completed: u64,
    /// The number of piece bytes remaining to be downloaded by the torrent
    pub bytes_remaining: u64,
}

/// Represents the response from a tracker announcement.
///
/// This struct contains the information returned by a tracker when announcing a peer.
/// It includes the interval at which the peer should re-announce, the number of leechers and seeders,
/// and a list of peer addresses.
#[derive(Debug, Clone)]
pub struct AnnounceEntryResponse {
    /// The interval (in seconds) at which the peer should re-announce itself to the tracker.
    pub interval_seconds: u64,
    /// The number of leechers currently downloading the torrent.
    pub leechers: u64,
    /// The number of seeders currently sharing the torrent.
    pub seeders: u64,
    /// A list of addresses (as `SocketAddr`) of peers to connect to.
    pub peers: Vec<SocketAddr>,
}

/// The metrics result of a tracker scrape operation.
#[derive(Debug, Default, Clone, Deserialize, PartialEq)]
pub struct ScrapeResult {
    /// The file metrics from the scrape result
    pub files: HashMap<InfoHash, ScrapeFileMetrics>,
}

/// The metrics of a specific torrent file.
#[derive(Debug, Default, Clone, Deserialize, PartialEq)]
pub struct ScrapeFileMetrics {
    /// The number of active peers that have completed downloading.
    pub complete: u32,
    /// The number of active peers that have not completed downloading.
    pub incomplete: u32,
    /// The number of peers that have ever completed downloading.
    pub downloaded: u32,
}

/// Trait that defines the underlying tracker connection protocol.
///
/// This trait defines the methods required to interact with a tracker, including connecting to the tracker,
/// announcing a peer and closing the connection.
///
/// Implementations of this trait will provide specific logic for different tracker connection protocols or types.
#[async_trait]
pub trait TrackerConnection: Debug + Send + Sync {
    /// Asynchronously start the tracker connection.
    ///
    /// This method should connect to one of the addresses provided by the tracker.
    ///
    /// # Returns
    ///
    /// A `Result` that is `Ok` if the connection was successful or an `Err` if there was an issue.
    async fn start(&mut self) -> Result<()>;

    /// Announce the given torrent hash to the tracker.
    /// This will send the known peer info to the tracker with the type of announcement.
    ///
    /// # Arguments
    ///
    /// * `info_hash` - The `InfoHash` of the torrent to announce.
    /// * `event` - The announcement event type to announce.
    ///
    /// # Returns
    ///
    /// It returns the tracker announcement response for the given announcement.
    async fn announce(&self, announcement: Announcement) -> Result<AnnounceEntryResponse>;

    /// Scrape the tracker for metrics the given info hashes.
    ///
    /// # Arguments
    ///
    /// * `hashes` - The info hashes to retrieve the metrics from.
    ///
    /// # Returns
    ///
    /// It returns the scrape result from the tracker for the given hashes.  
    async fn scrape(&self, hashes: &[InfoHash]) -> Result<ScrapeResult>;

    /// Close the tracker connection and cancel any pending tasks.
    ///
    /// This method should gracefully shut down the connection to the tracker and cancel any ongoing operations.
    fn close(&mut self);
}

/// The tracker identifier handle
pub type TrackerHandle = Handle;

#[derive(Debug, Display)]
#[display(fmt = "[{}] ({}){}", handle, tier, url)]
pub struct Tracker {
    /// The unique tracker handle
    handle: TrackerHandle,
    /// The tracker url
    url: Url,
    tier: u8,
    /// The unique peer id used within the torrent peer communication
    peer_id: PeerId,
    /// The peer port on which the torrent is listening for accepting incoming connections
    peer_port: u16,
    endpoints: Vec<SocketAddr>,
    connection: Box<dyn TrackerConnection>,
    /// The timeout for tracker connections before failing
    timeout: Duration,
    /// The interval in seconds to do another announcement to the tracker
    announcement_interval_seconds: RwLock<u64>,
    /// The last time an announcement was made by this tracker
    last_announcement: RwLock<Instant>,
}

impl Tracker {
    pub fn builder() -> TrackerBuilder {
        TrackerBuilder::builder()
    }

    pub async fn new(
        url: Url,
        tier: u8,
        peer_id: PeerId,
        peer_port: u16,
        timeout: Duration,
        announcement_interval_seconds: u64,
    ) -> Result<Self> {
        trace!("Trying to create new tracker for {}", url);
        let handle = TrackerHandle::new();
        let endpoints = DnsResolver::new(url.to_string())
            .resolve()
            .await
            .map_err(|e| TrackerError::Io(e.to_string()))?;
        let connection =
            Self::create_connection(&url, peer_id, peer_port, &endpoints, timeout.clone()).await?;

        trace!("Resolved tracker {} to {:?}", url, endpoints);
        Ok(Self {
            handle,
            url,
            tier,
            peer_id,
            peer_port,
            endpoints,
            connection,
            timeout,
            announcement_interval_seconds: RwLock::new(announcement_interval_seconds),
            last_announcement: RwLock::new(
                Instant::now().sub(Duration::from_secs(DEFAULT_ANNOUNCEMENT_INTERVAL_SECONDS)),
            ),
        })
    }

    /// The unique handle for this tracker.
    pub fn handle(&self) -> TrackerHandle {
        self.handle
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Get the expected announcement interval in seconds.
    ///
    /// # Returns
    ///
    /// Returns the interval in seconds for the announcements.
    pub async fn announcement_interval(&self) -> u64 {
        self.announcement_interval_seconds.read().await.clone()
    }

    /// Retrieve the last time this tracker made an announcement.
    ///
    /// # Returns
    ///
    /// Returns the last time this tracker made an announcement.
    pub async fn last_announcement(&self) -> Instant {
        self.last_announcement.read().await.clone()
    }

    /// Announce the given event to this tracker for the given torrent info hash.
    ///
    /// # Arguments
    ///
    /// * `info_hash` - The torrent info hash to make the announcement for
    /// * `event` - The announcement event
    ///
    /// # Returns
    ///
    /// It returns the announcement response from the tracker.
    pub async fn announce(&self, announce: Announcement) -> Result<AnnounceEntryResponse> {
        trace!("Tracker {} is announcing {:?}", self, announce);
        match self.connection.announce(announce).await {
            Ok(e) => {
                {
                    let mut mutex = self.last_announcement.write().await;
                    *mutex = Instant::now();
                }
                {
                    let mut mutex = self.announcement_interval_seconds.write().await;
                    *mutex = e.interval_seconds;
                }

                Ok(e)
            }
            Err(e) => Err(e),
        }
    }

    /// Scrape the tracker for metrics of the given info hashes.
    ///
    /// # Arguments
    ///
    /// * `hashes` - The info hashes to retrieve the metrics from.
    ///
    /// # Returns
    ///
    /// It returns the scrape metrics result from the tracker for the given info hashes.
    pub async fn scrape(&self, hashes: &[InfoHash]) -> Result<ScrapeResult> {
        trace!("Tracker {} is scraping {:?}", self, hashes);
        self.connection.scrape(hashes).await
    }

    async fn create_connection(
        url: &Url,
        peer_id: PeerId,
        peer_port: u16,
        addrs: &[SocketAddr],
        timeout: Duration,
    ) -> Result<Box<dyn TrackerConnection>> {
        trace!("Trying to connect to tracker at {}", url);
        let scheme = url.scheme();
        let mut connection: Box<dyn TrackerConnection>;

        match scheme {
            "udp" => {
                connection = Box::new(UdpConnection::new(addrs, peer_id, peer_port, timeout));
            }
            "http" | "https" => {
                connection = Box::new(HttpConnection::new(
                    url.clone(),
                    peer_id,
                    peer_port,
                    timeout,
                ));
            }
            _ => return Err(TrackerError::UnsupportedScheme(scheme.to_string())),
        }

        let _ = select! {
            _ = time::sleep(timeout) => Err(TrackerError::Timeout(url.clone())),
            result = connection.start() => result,
        }?;

        debug!("Tracker {} connection established", url);
        Ok(connection)
    }
}

impl Drop for Tracker {
    fn drop(&mut self) {
        self.connection.close();
    }
}

#[derive(Debug, Default)]
pub struct TrackerBuilder {
    url: Option<Url>,
    tier: Option<u8>,
    peer_id: Option<PeerId>,
    peer_port: Option<u16>,
    timeout: Option<Duration>,
    default_announcement_interval_seconds: Option<u64>,
}

impl TrackerBuilder {
    pub fn builder() -> Self {
        Self::default()
    }

    pub fn url(mut self, url: Url) -> Self {
        self.url = Some(url);
        self
    }

    pub fn tier(mut self, tier: u8) -> Self {
        self.tier = Some(tier);
        self
    }

    pub fn peer_id(mut self, peer_id: PeerId) -> Self {
        self.peer_id = Some(peer_id);
        self
    }

    pub fn peer_port(mut self, peer_port: u16) -> Self {
        self.peer_port = Some(peer_port);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn default_announcement_interval_seconds(
        mut self,
        announcement_interval_seconds: u64,
    ) -> Self {
        self.default_announcement_interval_seconds = Some(announcement_interval_seconds);
        self
    }

    pub async fn build(self) -> Result<Tracker> {
        let url = self.url.expect("expected the url to be set");
        let tier = self.tier.unwrap_or(0);
        let peer_id = self.peer_id.expect("expected the peer id to be set");
        let peer_port = self.peer_port.unwrap_or(6881);
        let timeout = self
            .timeout
            .unwrap_or(Duration::from_secs(DEFAULT_CONNECTION_TIMEOUT_SECONDS));
        let default_announcement_interval_seconds = self
            .default_announcement_interval_seconds
            .unwrap_or(DEFAULT_ANNOUNCEMENT_INTERVAL_SECONDS);

        Tracker::new(
            url,
            tier,
            peer_id,
            peer_port,
            timeout,
            default_announcement_interval_seconds,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use popcorn_fx_core::init_logger;
    use popcorn_fx_core::testing::read_test_file_to_bytes;

    use crate::torrent::TorrentMetadata;

    use super::*;

    #[tokio::test]
    async fn test_tracker_new() {
        init_logger!();
        let url = Url::parse("udp://tracker.opentrackr.org:1337").unwrap();
        let peer_id = PeerId::new();

        let result = Tracker::builder()
            .url(url)
            .peer_id(peer_id)
            .build()
            .await
            .expect("expected the tracker to be created");

        assert_eq!(1, result.endpoints.len());
    }

    #[tokio::test]
    async fn test_tracker_announce_udp() {
        init_logger!();
        let data = read_test_file_to_bytes("debian-udp.torrent");
        let info = TorrentMetadata::try_from(data.as_slice()).unwrap();

        let result = execute_tracker_announcement(&info).await;

        assert_ne!(
            0,
            result.peers.len(),
            "expected the announce to return peers"
        );
    }

    #[tokio::test]
    async fn test_tracker_announce_https() {
        init_logger!();
        let data = read_test_file_to_bytes("ubuntu-https.torrent");
        let info = TorrentMetadata::try_from(data.as_slice()).unwrap();

        let result = execute_tracker_announcement(&info).await;

        assert_ne!(
            0,
            result.peers.len(),
            "expected the announce to return peers"
        );
    }

    #[tokio::test]
    async fn test_tracker_scrape_udp() {
        init_logger!();
        let data = read_test_file_to_bytes("debian-udp.torrent");
        let info = TorrentMetadata::try_from(data.as_slice()).unwrap();
        let tracker = create_tracker(&info).await;

        let result = tracker
            .scrape(&vec![info.info_hash])
            .await
            .expect("expected a scrape response");

        assert_eq!(
            1,
            result.files.len(),
            "expected the scrape files to match the files from the info hash"
        );
    }

    async fn execute_tracker_announcement(info: &TorrentMetadata) -> AnnounceEntryResponse {
        let info_hash = info.info_hash.clone();
        let announce = Announcement {
            info_hash,
            event: AnnounceEvent::Started,
            bytes_completed: 0,
            bytes_remaining: u64::MAX,
        };
        let tracker = create_tracker(&info).await;

        tracker
            .announce(announce)
            .await
            .expect("expected the announce to succeed")
    }

    async fn create_tracker(metadata: &TorrentMetadata) -> Tracker {
        let peer_id = PeerId::new();
        let tracker_uris = metadata.tiered_trackers();
        let tracker_uri = tracker_uris.get(&0).map(|e| e.get(0).unwrap()).unwrap();

        Tracker::builder()
            .url(tracker_uri.clone())
            .peer_id(peer_id)
            .timeout(Duration::from_secs(2))
            .build()
            .await
            .expect("expected the tracker to have been created")
    }
}
