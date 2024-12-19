use crate::torrent::peer::PeerId;
use crate::torrent::tracker::http::HttpConnection;
use crate::torrent::tracker::udp::UdpConnection;
use crate::torrent::tracker::{AnnounceEvent, Result, TrackerError};
use crate::torrent::InfoHash;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use derive_more::Display;
use log::{debug, trace};
use popcorn_fx_core::core::Handle;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::lookup_host;
use tokio::sync::RwLock;
use tokio::{select, time};
use url::Url;

const DEFAULT_CONNECTION_TIMEOUT_SECONDS: u64 = 10;
const DEFAULT_ANNOUNCEMENT_INTERVAL_SECONDS: u64 = 120;

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
    /// A `Result` containing the `Announce` struct with tracker response data or an error if the announcement failed.
    async fn announce(
        &self,
        info_hash: InfoHash,
        event: AnnounceEvent,
    ) -> Result<AnnounceEntryResponse>;

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
    peer_id: PeerId,
    endpoints: Vec<SocketAddr>,
    connection: Box<dyn TrackerConnection>,
    /// The timeout for tracker connections before failing
    timeout: Duration,
    /// The interval in seconds to do another announcement to the tracker
    announcement_interval_seconds: RwLock<u64>,
    /// The last time an announcement was made by this tracker
    last_announcement: RwLock<DateTime<Utc>>,
}

impl Tracker {
    pub fn builder() -> TrackerBuilder {
        TrackerBuilder::builder()
    }

    pub async fn new(
        url: Url,
        tier: u8,
        peer_id: PeerId,
        timeout: Duration,
        announcement_interval_seconds: u64,
    ) -> Result<Self> {
        trace!("Trying to create new tracker for {}", url);
        let handle = TrackerHandle::new();
        let endpoints = Self::resolve(&url).await?;
        let connection =
            Self::create_connection(&url, peer_id, &endpoints, timeout.clone()).await?;
        let last_announcement = DateTime::from_timestamp(0, 0).unwrap();

        trace!("Resolved tracker {} to {:?}", url, endpoints);
        Ok(Self {
            handle,
            url,
            tier,
            peer_id,
            endpoints,
            connection,
            timeout,
            announcement_interval_seconds: RwLock::new(announcement_interval_seconds),
            last_announcement: RwLock::new(last_announcement),
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
    pub async fn last_announcement(&self) -> DateTime<Utc> {
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
    /// Returns the announcement response from the tracker.
    pub async fn announce(
        &self,
        info_hash: InfoHash,
        event: AnnounceEvent,
    ) -> Result<AnnounceEntryResponse> {
        match self.connection.announce(info_hash, event).await {
            Ok(e) => {
                {
                    let mut mutex = self.last_announcement.write().await;
                    *mutex = Utc::now();
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

    async fn create_connection(
        url: &Url,
        peer_id: PeerId,
        addrs: &[SocketAddr],
        timeout: Duration,
    ) -> Result<Box<dyn TrackerConnection>> {
        trace!("Trying to connect to tracker at {}", url);
        let scheme = url.scheme();
        let mut connection: Box<dyn TrackerConnection>;

        match scheme {
            "udp" => {
                connection = Box::new(UdpConnection::new(addrs, peer_id, timeout));
            }
            "http" | "https" => {
                connection = Box::new(HttpConnection::new(url.clone(), peer_id, timeout));
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

    async fn resolve(url: &Url) -> Result<Vec<SocketAddr>> {
        let host = url.host_str().unwrap();
        let port = url.port().unwrap_or(80);

        trace!("Resolving tracker {}:{}", host, port);
        lookup_host((host, port))
            .await
            .map(|e| e.collect())
            .map_err(|e| TrackerError::Io(e.to_string()))
    }
}

impl Drop for Tracker {
    fn drop(&mut self) {
        self.connection.close();
        // todo: add announce event stopped
    }
}

#[derive(Debug, Default)]
pub struct TrackerBuilder {
    url: Option<Url>,
    tier: Option<u8>,
    peer_id: Option<PeerId>,
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

        let result = execute_tracker_announcement(info).await;

        assert_ne!(
            0,
            result.peers.len(),
            "expected the announce to return peers"
        );
    }

    #[tokio::test]
    async fn test_tracker_announce_https() {
        init_logger!();
        let data = read_test_file_to_bytes("debian.torrent");
        let info = TorrentMetadata::try_from(data.as_slice()).unwrap();

        let result = execute_tracker_announcement(info).await;

        assert_ne!(
            0,
            result.peers.len(),
            "expected the announce to return peers"
        );
    }

    async fn execute_tracker_announcement(info: TorrentMetadata) -> AnnounceEntryResponse {
        let peer_id = PeerId::new();
        let tracker_uris = info.tiered_trackers();
        let tracker_uri = tracker_uris.get(&0).map(|e| e.get(0).unwrap()).unwrap();
        let info_hash = info.info_hash.clone();
        let tracker = Tracker::builder()
            .url(tracker_uri.clone())
            .peer_id(peer_id)
            .timeout(Duration::from_secs(1))
            .build()
            .await
            .unwrap();

        tracker
            .announce(info_hash, AnnounceEvent::Started)
            .await
            .expect("expected the announce to succeed")
    }
}
