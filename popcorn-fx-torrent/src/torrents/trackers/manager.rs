use std::fmt::{Debug, Formatter};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use derive_more::Display;
use log::{debug, info, trace, warn};
use tokio::runtime::Runtime;
use tokio::sync::RwLock;
use tokio::{select, time};
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::torrents::peers::PeerId;
use crate::torrents::trackers::{AnnounceEntryResponse, Result, Tracker, TrackerError};
use crate::torrents::InfoHash;

/// Kinds of tracker announces. This is typically indicated as the ``&event=``
/// HTTP query string parameter to HTTP trackers.
#[repr(u8)]
#[derive(Debug, Display, Clone)]
pub enum Event {
    #[display(fmt = "none")]
    None = 0,
    #[display(fmt = "completed")]
    Completed = 1,
    #[display(fmt = "started")]
    Started = 2,
    #[display(fmt = "stopped")]
    Stopped = 3,
    #[display(fmt = "paused")]
    Paused = 4,
}

/// The announcement result returned by all trackers.
#[derive(Default, PartialEq)]
pub struct Announcement {
    /// The total number of leechers reported by the trackers.
    pub total_leechers: u64,
    /// The total number of seeders reported by the trackers.
    pub total_seeders: u64,
    /// The list of peers' addresses reported by the trackers.
    pub peers: Vec<SocketAddr>,
}

impl Announcement {
    pub fn total_peers(&self) -> u64 {
        self.peers.len() as u64
    }
}

impl Debug for Announcement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Announcement")
            .field("total_leechers", &self.total_leechers)
            .field("total_seeders", &self.total_seeders)
            .field("peers", &self.total_peers())
            .finish()
    }
}

/// Manages trackers and handles periodic announcements.
///
/// The `TrackerManager` is responsible for managing a list of trackers, performing automatic announcements, and handling tracker updates.
#[derive(Debug)]
pub struct TrackerManager {
    inner: Arc<InnerTrackerManager>,
    runtime: Arc<Runtime>,
    cancellation_token: CancellationToken,
}

impl TrackerManager {
    /// Creates a new `TrackerManager` instance.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The peer ID to associate with the manager.
    /// * `peer_port` - The port number on which the [Session] is listening for incoming peer connections.
    /// * `timeout` - The timeout for tracker connections.
    /// * `runtime` - The runtime environment for spawning asynchronous tasks.
    ///
    /// # Returns
    ///
    /// A `TrackerManager` instance with initialized settings.
    pub fn new(peer_id: PeerId, peer_port: u16, timeout: Duration, runtime: Arc<Runtime>) -> Self {
        let inner = Arc::new(InnerTrackerManager::new(peer_id, peer_port, timeout));
        let cancellation_token = CancellationToken::new();

        Self {
            inner,
            cancellation_token,
            runtime,
        }
    }

    /// Retrieve the trackers which are currently active within this manager.
    ///
    /// # Returns
    ///
    /// Returns an array of active tracker URLs.
    pub async fn trackers(&self) -> Vec<Url> {
        let trackers = self.inner.trackers.read().await;
        trackers.iter().map(|e| e.url().clone()).collect()
    }

    /// Adds a new tracker to the manager.
    ///
    /// # Arguments
    ///
    /// * `tracker` - The new tracker to add.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure. Returns `TrackerError::DuplicateUrl` if the URL already exists.
    pub async fn add_tracker(&self, url: &Url, tier: u8) -> Result<()> {
        match Tracker::builder()
            .url(url.clone())
            .tier(tier)
            .timeout(self.inner.connection_timeout.clone())
            .peer_id(self.inner.peer_id.clone())
            .build()
            .await
        {
            Ok(tracker) => {
                {
                    let trackers = self.inner.trackers.read().await;
                    if trackers.iter().any(|e| e.url() == tracker.url()) {
                        return Err(TrackerError::DuplicateUrl(tracker.url().clone()));
                    }
                }

                {
                    let mut mutex = self.inner.trackers.write().await;
                    debug!("Adding new {}", tracker);
                    mutex.push(tracker);
                }
            }
            Err(e) => {
                warn!("Failed to create new tracker {}: {}", url, e);
                return Err(e);
            }
        }

        Ok(())
    }

    /// Updates the info hash for the manager.
    ///
    /// # Arguments
    ///
    /// * `info_hash` - The new info hash to set.
    pub async fn update_info_hash(&self, info_hash: InfoHash) {
        self.inner.update_info_hash(info_hash).await
    }

    /// Starts announcing to all trackers with the specified info hash.
    /// This will start automatic announcement loops for each known tracker within this manager.
    ///
    /// # Arguments
    ///
    /// * `info_hash` - The info hash to announce.
    pub fn start_announcing(&self, info_hash: InfoHash) {
        trace!("Creating a new automatic announcement loop for {:?}", self);
        let announcement_cancellation_token = self.cancellation_token.clone();
        let announcement_manager = self.inner.clone();
        self.runtime.spawn(async move {
            loop {
                select! {
                    _ = announcement_cancellation_token.cancelled() => break,
                    _ = Self::do_automatic_announcements(&announcement_manager) => {},
                }

                select! {
                    _ = announcement_cancellation_token.cancelled() => break,
                    _ = time::sleep(Duration::from_secs(10)) => {},
                }
            }

            debug!(
                "Automatic announcement loop of {:?} has been terminated",
                announcement_manager
            );
        });

        let inner = self.inner.clone();
        self.runtime.spawn(async move {
            inner.update_info_hash(info_hash.clone()).await;
            inner.announce_all(info_hash).await;
        });
    }

    /// Announces to all trackers with the specified info hash.
    /// In regard to [self.start_announcing], this will not start an automatic announcement loop
    /// and is more a one of.
    ///
    /// # Arguments
    ///
    /// * `info_hash` - The info hash to announce.
    ///
    /// # Returns
    ///
    /// Returns the announcement response result.
    pub async fn announce_all(&self, info_hash: InfoHash) -> Announcement {
        let start_time = Instant::now();
        let result = self.inner.announce_all(info_hash).await;
        let elapsed = start_time.elapsed();
        trace!(
            "Announced to all trackers in {}.{:03} seconds",
            elapsed.as_secs(),
            elapsed.subsec_millis()
        );
        result
    }

    /// Performs automatic announcements to all trackers periodically.
    ///
    /// This method is called by the periodic task loop.
    ///
    /// # Arguments
    ///
    /// * `manager` - The `InnerTrackerManager` to perform announcements with.
    async fn do_automatic_announcements(manager: &Arc<InnerTrackerManager>) {
        let mut mutex = manager.trackers.write().await;
        let now = Utc::now();

        if let Some(info_hash) = manager.info_hash().await {
            for tracker in mutex.as_mut_slice() {
                let interval = tracker.announcement_interval();
                let last_announcement = tracker.last_announcement();
                let delta = now.signed_duration_since(last_announcement);

                if delta.num_seconds() >= interval as i64 {
                    match manager.announce_tracker(tracker, info_hash.clone()).await {
                        Ok(_) => {}
                        Err(e) => warn!("Failed make an announcement for {}, {}", tracker, e),
                    }
                }
            }
        } else {
            warn!("Unable to announce trackers, no info hash set");
        }
    }
}

impl Drop for TrackerManager {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}

#[derive(Debug)]
struct InnerTrackerManager {
    peer_id: PeerId,
    peer_port: u16,
    info_hash: RwLock<Option<InfoHash>>,
    trackers: RwLock<Vec<Tracker>>,
    peers: RwLock<Vec<SocketAddr>>,
    connection_timeout: Duration,
}

impl InnerTrackerManager {
    fn new(peer_id: PeerId, peer_port: u16, connection_timeout: Duration) -> Self {
        Self {
            peer_id,
            peer_port,
            info_hash: Default::default(),
            trackers: Default::default(),
            peers: Default::default(),
            connection_timeout,
        }
    }

    async fn info_hash(&self) -> Option<InfoHash> {
        self.info_hash.read().await.clone()
    }

    async fn update_info_hash(&self, info_hash: InfoHash) {
        trace!("Updating info hash to {:?}", info_hash);
        let mut mutex = self.info_hash.write().await;
        *mutex = Some(info_hash);
    }

    async fn add_peers(&self, peers: Vec<SocketAddr>) {
        trace!("Adding a total of {} peers", peers.len());
        let mut mutex = self.peers.write().await;
        for peer in peers {
            if !mutex.contains(&peer) {
                trace!("Adding peer {:?}", peer);
                mutex.push(peer);
            }
        }
    }

    async fn announce_all(&self, info_hash: InfoHash) -> Announcement {
        let mut result = Announcement::default();
        let mut total_peers = 0;

        let mut mutex = self.trackers.write().await;
        for tracker in mutex.as_mut_slice() {
            if let Ok(response) = self.announce_tracker(tracker, info_hash.clone()).await {
                result.total_leechers += response.leechers;
                result.total_seeders += response.seeders;
                result.peers.extend_from_slice(response.peers.as_slice());

                let found_peers = response.peers.len();
                self.add_peers(response.peers).await;
                total_peers += found_peers;
            }
        }

        info!("Found a total of {} peers", total_peers);
        result
    }

    async fn announce_tracker(
        &self,
        tracker: &mut Tracker,
        info_hash: InfoHash,
    ) -> Result<AnnounceEntryResponse> {
        trace!("Announcing to tracker {}", tracker);
        match tracker.announce(info_hash).await {
            Ok(response) => {
                debug!(
                    "Tracker {} announcement found {} peers",
                    tracker,
                    response.peers.len()
                );
                Ok(response)
            }
            Err(e) => {
                warn!("Tracker {} announcement failed, {:?}", tracker, e);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use popcorn_fx_core::testing::init_logger;

    use super::*;

    #[test]
    fn test_add_tracker() {
        init_logger();
        let url = Url::parse("udp://tracker.opentrackr.org:1337").unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let peer_id = PeerId::new();
        let manager = TrackerManager::new(peer_id, 6881, Duration::from_secs(1), runtime.clone());

        let result = runtime.block_on(manager.add_tracker(&url, 0));

        assert_eq!(Ok(()), result);
    }
}
