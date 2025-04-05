use std::fmt::{Debug, Formatter};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::torrent::peer::PeerId;
use crate::torrent::tracker::{
    AnnounceEntryResponse, Announcement, Result, ScrapeFileMetrics, ScrapeResult, Tracker,
    TrackerError, TrackerHandle,
};
use crate::torrent::InfoHash;
use chrono::Utc;
use derive_more::Display;
use futures::future;
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use log::{debug, info, trace, warn};
use tokio::sync::RwLock;
use tokio::{select, time};
use tokio_util::sync::CancellationToken;
use url::Url;

const DEFAULT_ANNOUNCEMENT_INTERVAL_SECONDS: u64 = 30;

/// Kinds of tracker announces. This is typically indicated as the ``&event=``
/// HTTP query string parameter to HTTP trackers.
#[repr(u8)]
#[derive(Debug, Display, Copy, Clone)]
pub enum AnnounceEvent {
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
#[derive(Default, Clone, PartialEq)]
pub struct AnnouncementResult {
    /// The total number of leechers reported by the trackers.
    pub total_leechers: u64,
    /// The total number of seeders reported by the trackers.
    pub total_seeders: u64,
    /// The list of peers' addresses reported by the trackers.
    pub peers: Vec<SocketAddr>,
}

impl AnnouncementResult {
    pub fn total_peers(&self) -> u64 {
        self.peers.len() as u64
    }
}

impl Debug for AnnouncementResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Announcement")
            .field("total_leechers", &self.total_leechers)
            .field("total_seeders", &self.total_seeders)
            .field("peers", &self.total_peers())
            .finish()
    }
}

#[derive(Debug, Display, Clone, PartialEq)]
#[display(fmt = "({}) {}", tier, url)]
pub struct TrackerEntry {
    /// The tier of the tracker
    pub tier: u8,
    /// The tracker url to connect to
    pub url: Url,
}

/// The event that can be emitted by the tracker manager.
#[derive(Debug, Clone, PartialEq)]
pub enum TrackerManagerEvent {
    /// Invoked when new peer have been discovered
    PeersDiscovered(Vec<SocketAddr>),
    /// Invoked when a new tracker has been added
    TrackerAdded(TrackerHandle),
}

/// Manages trackers and handles periodic announcements.
///
/// The `TrackerManager` is responsible for managing a list of trackers, performing automatic announcements, and handling tracker updates.
#[derive(Debug, Display)]
#[display(fmt = "{}", inner)]
pub struct TrackerManager {
    inner: Arc<InnerTrackerManager>,
}

impl TrackerManager {
    /// Creates a new `TrackerManager` instance.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The peer ID to associate with the manager.
    /// * `peer_port` - The port number on which the [Torrent] is listening for incoming peer connections.
    /// * `info_hash` - The info hash of the torrent being tracked by this manager.
    /// * `connection_timeout` - The timeout for tracker connections.
    ///
    /// # Returns
    ///
    /// A `TrackerManager` instance with initialized settings.
    pub fn new(
        peer_id: PeerId,
        peer_port: u16,
        info_hash: InfoHash,
        connection_timeout: Duration,
    ) -> Self {
        let inner = Arc::new(InnerTrackerManager {
            peer_id,
            peer_port,
            info_hash,
            trackers: Default::default(),
            peers: Default::default(),
            connection_timeout,
            bytes_remaining: RwLock::new(u64::MAX),
            bytes_completed: Default::default(),
            callbacks: MultiThreadedCallback::new(),
            cancellation_token: Default::default(),
        });

        let inner_main_loop = inner.clone();
        tokio::spawn(async move {
            inner_main_loop.start().await;
        });

        Self { inner }
    }

    /// Checks if a given tracker URL is known within this manager.
    pub async fn is_tracker_url_known(&self, url: &Url) -> bool {
        self.inner.is_tracker_url_known(url).await
    }

    /// Get the currently active trackers.
    /// This might return an empty list if no trackers have been added yet.
    pub async fn trackers(&self) -> Vec<Url> {
        let trackers = self.inner.trackers.read().await;
        trackers.iter().map(|e| e.url().clone()).collect()
    }

    /// Get the total number of active trackers.
    /// This might return 0 if no trackers have been added yet.
    pub async fn total_trackers(&self) -> usize {
        self.inner.trackers.read().await.len()
    }

    /// Get the currently known peers that have been discovered by the trackers.
    pub async fn discovered_peers(&self) -> Vec<SocketAddr> {
        self.inner.peers.read().await.clone()
    }

    /// Adds a new tracker to the manager.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the tracker to add.
    /// * `tier` - The tier of the tracker to add.
    ///
    /// # Returns
    ///
    /// Returns the created tracker handle on success, else the [TrackerError].
    pub async fn add_tracker_entry(&self, entry: TrackerEntry) -> Result<TrackerHandle> {
        self.inner.create_tracker_from_entry(entry).await
    }

    /// Adds a new tracker to the manager on a background task.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the tracker to add.
    /// * `tier` - The tier of the tracker to add.
    pub async fn add_tracker_async(&self, entry: TrackerEntry) {
        let inner = self.inner.clone();
        tokio::spawn(async move {
            select! {
                _ = inner.cancellation_token.cancelled() => return,
                _ = inner.create_tracker_from_entry(entry) => return,
            }
        });
    }

    /// Announces to all trackers with the specified info hash.
    /// In regard to [self.start_announcing], this will not start an automatic announcement loop
    /// and is more a one of.
    ///
    /// # Returns
    ///
    /// Returns the announcement response result.
    pub async fn announce_all(&self, event: AnnounceEvent) -> AnnouncementResult {
        let start_time = Instant::now();
        let result = self.inner.announce_all(event).await;
        let elapsed = start_time.elapsed();
        trace!(
            "Announced to all trackers in {}.{:03} seconds",
            elapsed.as_secs(),
            elapsed.subsec_millis()
        );
        result
    }

    /// Announce the given event to the specified tracker.
    pub async fn announce(
        &self,
        handle: TrackerHandle,
        event: AnnounceEvent,
    ) -> Result<AnnouncementResult> {
        self.inner.announce(handle, event).await
    }

    /// Announces to all the trackers with the specified info hash.
    /// This method will spawn the announcement task and return immediately.
    pub fn make_announcement_to_all(&self, event: AnnounceEvent) {
        let inner = self.inner.clone();
        tokio::spawn(async move {
            select! {
                _ = inner.cancellation_token.cancelled() => return,
                _ = inner.announce_all(event) => return,
            }
        });
    }

    /// Make a new announcement to the specified tracker for the given event.
    /// This method will spawn the announcement task and return immediately.
    pub fn make_announcement(&self, handle: TrackerHandle, event: AnnounceEvent) {
        let inner = self.inner.clone();
        tokio::spawn(async move {
            select! {
                _ = inner.cancellation_token.cancelled() => return,
                _ = inner.announce(handle, event) => return,
            }
        });
    }

    /// Scrape all active trackers.
    pub async fn scrape(&self) -> Result<ScrapeResult> {
        let mut result = ScrapeResult::default();
        let hashes = vec![self.inner.info_hash.clone()];

        if self.inner.trackers.read().await.len() == 0 {
            return Err(TrackerError::NoTrackers);
        }

        for tracker in self.inner.trackers.read().await.iter() {
            match tracker.scrape(&hashes).await {
                Ok(metrics) => {
                    for (hash, metrics) in metrics.files {
                        let file_metrics = result
                            .files
                            .entry(hash)
                            .or_insert(ScrapeFileMetrics::default());
                        file_metrics.complete += metrics.complete;
                        file_metrics.incomplete += metrics.incomplete;
                        file_metrics.downloaded += metrics.downloaded;
                    }
                }
                Err(e) => debug!("Tracker manager {} failed to scrape tracker, {}", self, e),
            }
        }

        Ok(result)
    }

    /// Inform the trackers about the remaining number of piece bytes that need to be downloaded by the torrent.
    pub async fn update_bytes_remaining(&self, remaining_bytes: usize) {
        trace!(
            "Torrent manager {} is updating remaining bytes to {}",
            self,
            remaining_bytes
        );
        *self.inner.bytes_remaining.write().await = remaining_bytes as u64;
    }

    /// Inform the trackers about the number of completed piece bytes by the torrent.
    pub async fn update_bytes_completed(&self, bytes_completed: usize) {
        trace!(
            "Tracker manager {} is updating bytes completed to {}",
            self,
            bytes_completed
        );
        *self.inner.bytes_completed.write().await = bytes_completed as u64;
    }
}

impl Callback<TrackerManagerEvent> for TrackerManager {
    fn subscribe(&self) -> Subscription<TrackerManagerEvent> {
        self.inner.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<TrackerManagerEvent>) {
        self.inner.callbacks.subscribe_with(subscriber)
    }
}

impl Drop for TrackerManager {
    fn drop(&mut self) {
        trace!("Dropping tracker manager {}", self.inner);
        self.inner.cancellation_token.cancel();
    }
}

#[derive(Debug, Display)]
#[display(fmt = "{} ({})", peer_id, info_hash)]
struct InnerTrackerManager {
    /// The unique peer id of the torrent
    peer_id: PeerId,
    /// The port the torrent is listening on to accept incoming connections
    peer_port: u16,
    /// The torrent info hash for which this tracker manager is responsible for
    info_hash: InfoHash,
    /// The active trackers
    trackers: RwLock<Vec<Tracker>>,
    /// The discovered peers from the trackers
    peers: RwLock<Vec<SocketAddr>>,
    /// The timeout of tracker connections
    connection_timeout: Duration,
    /// The number completed piece bytes
    bytes_completed: RwLock<u64>,
    /// The number of remaining piece bytes that need to be downloaded
    bytes_remaining: RwLock<u64>,
    callbacks: MultiThreadedCallback<TrackerManagerEvent>,
    cancellation_token: CancellationToken,
}

impl InnerTrackerManager {
    /// Start the main loop of the tracker manager.
    async fn start(&self) {
        let mut announcement_tick =
            time::interval(Duration::from_secs(DEFAULT_ANNOUNCEMENT_INTERVAL_SECONDS));

        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                _ = announcement_tick.tick() => self.do_automatic_announcements().await,
            }
        }

        self.announce_stopped().await;
        debug!("Tracker manager {} main loop has stopped", self);
    }

    /// Check if the given url is already registered/known.
    async fn is_tracker_url_known(&self, url: &Url) -> bool {
        let trackers = self.trackers.read().await;
        trackers.iter().any(|e| e.url() == url)
    }

    /// Try to create a new tracker for the given url.
    /// It returns the created tracker handle on success, else the [TrackerError].
    async fn create_tracker_from_entry(&self, entry: TrackerEntry) -> Result<TrackerHandle> {
        // if the url is already known, reject the request to create the tracker
        let url_already_exists = self.is_tracker_url_known(&entry.url).await;
        if url_already_exists {
            return Err(TrackerError::DuplicateUrl(entry.url));
        }

        match Tracker::builder()
            .url(entry.url)
            .tier(entry.tier)
            .timeout(self.connection_timeout.clone())
            .peer_id(self.peer_id.clone())
            .peer_port(self.peer_port)
            .build()
            .await
        {
            Ok(tracker) => self.add_tracker(tracker).await,
            Err(e) => {
                debug!("Failed to create new tracker, {}", e);
                Err(e)
            }
        }
    }

    /// Add the given tracker to the trackers pool.
    /// Returns a unique tracker handle for the added tracker.
    async fn add_tracker(&self, tracker: Tracker) -> Result<TrackerHandle> {
        let handle = tracker.handle();
        let tracker_info = tracker.to_string();

        {
            let mut mutex = self.trackers.write().await;
            mutex.push(tracker);
            debug!("Tracker {} has been added to {}", tracker_info, self);
        }

        self.send_event(TrackerManagerEvent::TrackerAdded(handle))
            .await;
        Ok(handle)
    }

    /// Add the discovered peer addresses to the manager.
    /// This will only add unique peer addresses and filter out any duplicate addresses that have already been discovered.
    ///
    /// It returns the total amount of added peer addresses that were added.
    async fn add_peers(&self, peers: &[SocketAddr]) -> usize {
        trace!("Discovered a total of {} peers, {:?}", peers.len(), peers);
        let mut mutex = self.peers.write().await;
        let mut unique_new_peer_addrs = Vec::new();

        for peer in peers.into_iter() {
            if !mutex.contains(peer) {
                mutex.push(peer.clone());
                unique_new_peer_addrs.push(peer.clone());
            }
        }

        debug!(
            "Discovered a total of {} new peers",
            unique_new_peer_addrs.len()
        );
        let total_peers = unique_new_peer_addrs.len();
        if total_peers > 0 {
            self.send_event(TrackerManagerEvent::PeersDiscovered(unique_new_peer_addrs))
                .await;
        }

        total_peers
    }

    async fn announce(
        &self,
        handle: TrackerHandle,
        event: AnnounceEvent,
    ) -> Result<AnnouncementResult> {
        let mutex = self.trackers.read().await;
        let tracker = mutex
            .iter()
            .find(|e| e.handle() == handle)
            .ok_or(TrackerError::InvalidHandle(handle))?;

        match self
            .announce_tracker(tracker, event)
            .await
            .map(|e| AnnouncementResult {
                total_leechers: e.leechers,
                total_seeders: e.seeders,
                peers: e.peers,
            }) {
            Ok(e) => {
                // update the discovered peers
                self.add_peers(e.peers.as_slice()).await;
                Ok(e)
            }
            Err(e) => Err(e),
        }
    }

    async fn announce_all(&self, event: AnnounceEvent) -> AnnouncementResult {
        let mut result = AnnouncementResult::default();
        let mut total_peers = 0;
        let mutex = self.trackers.read().await;

        // start announcing the given hash to each tracker simultaneously
        let futures: Vec<_> = mutex
            .iter()
            .map(|tracker| self.announce_tracker(tracker, event))
            .collect();

        // wait for all responses to complete
        let responses = future::join_all(futures).await;
        for response in responses {
            match response {
                Ok(response) => {
                    result.total_leechers += response.leechers;
                    result.total_seeders += response.seeders;
                    result.peers.extend_from_slice(response.peers.as_slice());

                    total_peers += self.add_peers(response.peers.as_slice()).await;
                }
                Err(e) => debug!(
                    "Failed to announce info hash {:?} to tracker, {}",
                    self.info_hash, e
                ),
            }
        }

        info!(
            "Discovered a total of {} peers for {}",
            total_peers, self.info_hash
        );
        result
    }

    async fn announce_stopped(&self) {
        let mut mutex = self.trackers.write().await;
        let mut futures = Vec::new();

        debug!(
            "Announcing stopped event to all trackers for {:?}",
            self.info_hash
        );
        for tracker in mutex.as_mut_slice() {
            futures.push(self.announce_tracker(tracker, AnnounceEvent::Stopped));
        }

        // wait for all responses to complete and filter on errors
        let responses: Vec<Result<AnnounceEntryResponse>> = future::join_all(futures)
            .await
            .into_iter()
            .filter(|e| e.is_err())
            .collect();
        for response in responses {
            if let Err(e) = response {
                debug!(
                    "Failed to make stopped announcement to tracker for info hash {:?}, {}",
                    self.info_hash, e
                );
            }
        }
    }

    async fn announce_tracker(
        &self,
        tracker: &Tracker,
        event: AnnounceEvent,
    ) -> Result<AnnounceEntryResponse> {
        trace!("Announcing event {} to tracker {}", event, tracker);
        let announce = Announcement {
            info_hash: self.info_hash.clone(),
            event,
            bytes_completed: *self.bytes_completed.read().await,
            bytes_remaining: *self.bytes_remaining.read().await,
        };

        match tracker.announce(announce).await {
            Ok(response) => {
                debug!(
                    "Tracker {} announcement found {} peers",
                    tracker,
                    response.peers.len()
                );
                Ok(response)
            }
            Err(e) => {
                warn!(
                    "Announcement of event {} failed for tracker {}, {:?}",
                    event, tracker, e
                );
                Err(e)
            }
        }
    }

    async fn send_event(&self, event: TrackerManagerEvent) {
        self.callbacks.invoke(event);
    }

    /// Performs automatic announcements to all trackers periodically.
    ///
    /// This method is called by the periodic task loop.
    ///
    /// # Arguments
    ///
    /// * `manager` - The `InnerTrackerManager` to perform announcements with.
    async fn do_automatic_announcements(&self) {
        let mut mutex = self.trackers.write().await;
        let now = Utc::now();

        for tracker in mutex.as_mut_slice() {
            let interval = tracker.announcement_interval().await;
            let last_announcement = tracker.last_announcement().await;
            let delta = now.signed_duration_since(last_announcement);

            if delta.num_seconds() >= interval as i64 {
                match self.announce_tracker(tracker, AnnounceEvent::Started).await {
                    Ok(_) => {}
                    Err(e) => warn!("Failed make an announcement for {}, {}", tracker, e),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::recv_timeout;

    use popcorn_fx_core::init_logger;
    use std::str::FromStr;
    use tokio::sync::mpsc::unbounded_channel;
    use url::Url;

    #[tokio::test]
    async fn test_add_tracker() {
        init_logger!();
        let url = Url::parse("udp://tracker.opentrackr.org:1337").unwrap();
        let peer_id = PeerId::new();
        let info_hash =
            InfoHash::from_str("urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7").unwrap();
        let entry = TrackerEntry { tier: 0, url };
        let manager = TrackerManager::new(peer_id, 6881, info_hash, Duration::from_secs(1));

        let result = manager.add_tracker_entry(entry).await;

        assert_eq!(
            None,
            result.err(),
            "expected the tracker to have been created"
        );
    }

    #[tokio::test]
    async fn test_tracker_manager_announce_all() {
        init_logger!();
        let url = Url::parse("udp://tracker.opentrackr.org:1337").unwrap();
        let peer_id = PeerId::new();
        let info_hash =
            InfoHash::from_str("urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7").unwrap();
        let entry = TrackerEntry { tier: 0, url };
        let manager = TrackerManager::new(peer_id, 6881, info_hash, Duration::from_secs(1));

        manager.add_tracker_entry(entry).await.unwrap();
        let result = manager.announce_all(AnnounceEvent::Started).await;

        assert_ne!(0, result.peers.len(), "expected peers to have been found");
    }

    #[tokio::test]
    async fn test_tracker_manager_scrape() {
        init_logger!();
        let url = Url::parse("udp://tracker.opentrackr.org:1337").unwrap();
        let peer_id = PeerId::new();
        let info_hash =
            InfoHash::from_str("urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7").unwrap();
        let entry = TrackerEntry { tier: 0, url };
        let manager = TrackerManager::new(peer_id, 6881, info_hash, Duration::from_secs(1));

        manager.add_tracker_entry(entry).await.unwrap();
        let result = manager
            .scrape()
            .await
            .expect("expected the scrape to succeed");

        assert_eq!(
            1,
            result.files.len(),
            "expected the scrape result to match the torrent"
        );
    }

    #[tokio::test]
    async fn test_add_callback() {
        init_logger!();
        let url = Url::parse("udp://tracker.opentrackr.org:1337").unwrap();
        let peer_id = PeerId::new();
        let info_hash =
            InfoHash::from_str("urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7").unwrap();
        let entry = TrackerEntry { tier: 0, url };
        let (tx, mut rx) = unbounded_channel();
        let manager = TrackerManager::new(peer_id, 6881, info_hash, Duration::from_secs(1));

        let mut receiver = manager.subscribe();
        tokio::spawn(async move {
            if let Some(event) = receiver.recv().await {
                tx.send((*event).clone()).unwrap();
            }
        });

        manager
            .add_tracker_entry(entry)
            .await
            .expect("expected the tracker to have been created");

        let result = recv_timeout!(
            &mut rx,
            Duration::from_millis(200),
            "expected to receive an event"
        );
        if let TrackerManagerEvent::TrackerAdded(_) = result {
        } else {
            assert!(
                false,
                "expected TrackerManagerEvent::TrackerAdded, got {:?} instead",
                result
            );
        }
    }

    #[tokio::test]
    async fn test_drop() {
        init_logger!();
        let url = Url::parse("udp://tracker.opentrackr.org:1337").unwrap();
        let peer_id = PeerId::new();
        let info_hash =
            InfoHash::from_str("urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7").unwrap();
        let manager = TrackerManager::new(peer_id, 6881, info_hash, Duration::from_secs(1));

        manager
            .add_tracker_async(TrackerEntry { tier: 0, url })
            .await;
        drop(manager);
    }
}
