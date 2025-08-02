use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::torrent::peer::PeerId;
use crate::torrent::tracker::{
    AnnounceEntryResponse, Announcement, Result, ScrapeFileMetrics, ScrapeResult, Tracker,
    TrackerError, TrackerHandle, TrackerState,
};
use crate::torrent::InfoHash;
use derive_more::Display;
use futures::future;
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use log::{debug, info, trace, warn};
use tokio::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard};
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
    /// Invoked when new peers have been discovered for a torrent
    PeersDiscovered(InfoHash, Vec<SocketAddr>),
    /// Invoked when a new tracker has been added
    TrackerAdded(TrackerHandle),
}

/// Manages torrent trackers and handles periodic announcements.
///
/// The `TrackerManager` is responsible for managing a list of trackers, performing automatic announcements, and handling tracker updates.
#[derive(Debug, Display, Clone)]
#[display(fmt = "{}", inner)]
pub struct TrackerManager {
    inner: Arc<InnerTrackerManager>,
}

impl TrackerManager {
    /// Creates a new `TrackerManager` instance.
    ///
    /// # Arguments
    ///
    /// * `connection_timeout` - The timeout for tracker connections.
    ///
    /// # Returns
    ///
    /// A `TrackerManager` instance with initialized settings.
    pub fn new(connection_timeout: Duration) -> Self {
        let inner = Arc::new(InnerTrackerManager {
            handle: Default::default(),
            trackers: Default::default(),
            torrents: Default::default(),
            connection_timeout,
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

    /// Register a new torrent to the tracker to discover new peers.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The peer ID of the torrent.
    /// * `peer_port` - The port on which the torrent is listening.
    /// * `info_hash` - The info hash of the torrent.
    pub async fn add_torrent(
        &self,
        peer_id: PeerId,
        peer_port: u16,
        info_hash: InfoHash,
    ) -> Result<()> {
        self.inner.add_torrent(peer_id, peer_port, info_hash).await
    }

    /// Remove the given torrent info hash from the tracker.
    ///
    /// # Arguments
    ///
    /// * `info_hash` - The info hash of the torrent.
    pub async fn remove_torrent(&self, info_hash: &InfoHash) {
        self.inner.remove_torrent(info_hash).await
    }

    /// Get the discovered peers for the given info hash.
    /// The info hash should be first registered through the [TrackerManager::add_torrent].
    pub async fn discovered_peers(&self, info_hash: &InfoHash) -> Option<Vec<SocketAddr>> {
        let mut torrents = self.inner.torrents.lock().await;
        self.inner
            .find_torrent(info_hash, &mut torrents)
            .await
            .map(|e| e.peers.iter().cloned().collect())
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
    ///
    /// # Returns
    ///
    /// Returns the announcement response result.
    pub async fn announce_all(
        &self,
        info_hash: &InfoHash,
        event: AnnounceEvent,
    ) -> AnnouncementResult {
        let start_time = Instant::now();
        let result = self.inner.announce_all(info_hash, event).await;
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
        info_hash: &InfoHash,
        event: AnnounceEvent,
    ) -> Result<AnnouncementResult> {
        self.inner.announce(handle, info_hash, event).await
    }

    /// Announces to all the trackers with the specified info hash.
    /// This method will spawn the announcement task and return immediately.
    pub fn make_announcement_to_all(&self, info_hash: &InfoHash, event: AnnounceEvent) {
        let info_hash = info_hash.clone();
        let inner = self.inner.clone();
        tokio::spawn(async move {
            select! {
                _ = inner.cancellation_token.cancelled() => return,
                _ = inner.announce_all(&info_hash, event) => return,
            }
        });
    }

    /// Make a new announcement to the specified tracker for the given event.
    /// This method will spawn the announcement task and return immediately.
    pub fn make_announcement(
        &self,
        handle: TrackerHandle,
        info_hash: &InfoHash,
        event: AnnounceEvent,
    ) {
        let info_hash = info_hash.clone();
        let inner = self.inner.clone();
        tokio::spawn(async move {
            select! {
                _ = inner.cancellation_token.cancelled() => return,
                _ = inner.announce(handle, &info_hash, event) => return,
            }
        });
    }

    /// Scrape all trackers for peers of the given [InfoHash].
    pub async fn scrape(&self, info_hash: &InfoHash) -> Result<ScrapeResult> {
        let mut result = ScrapeResult::default();
        let hashes = vec![info_hash.clone()];

        let trackers = self.inner.trackers.read().await;
        if trackers.is_empty() {
            return Err(TrackerError::NoTrackers);
        }

        let scrape_results = future::join_all(
            self.inner
                .active_trackers(&trackers)
                .await
                .iter()
                .map(|tracker| tracker.scrape(&hashes)),
        )
        .await;

        for scrape_result in scrape_results.into_iter() {
            match scrape_result {
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
    pub async fn update_bytes_remaining(&self, info_hash: &InfoHash, remaining_bytes: usize) {
        trace!(
            "Torrent manager {} is updating remaining bytes to {}",
            self,
            remaining_bytes
        );
        let mut torrents = self.inner.torrents.lock().await;
        if let Some(torrent) = self.inner.find_torrent(info_hash, &mut torrents).await {
            torrent.bytes_remaining = remaining_bytes as u64;
        }
    }

    /// Inform the trackers about the number of completed piece bytes by the torrent.
    pub async fn update_bytes_completed(&self, info_hash: &InfoHash, bytes_completed: usize) {
        trace!(
            "Tracker manager {} is updating bytes completed to {}",
            self,
            bytes_completed
        );
        let mut torrents = self.inner.torrents.lock().await;
        if let Some(torrent) = self.inner.find_torrent(info_hash, &mut torrents).await {
            torrent.bytes_completed = bytes_completed as u64;
        }
    }

    /// Close the tracker manager resulting in a termination of its operations.
    pub fn close(&self) {
        self.inner.cancellation_token.cancel();
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

#[derive(Debug, Display)]
#[display(fmt = "{}", handle)]
struct InnerTrackerManager {
    /// The unique handle of this tracker
    handle: TrackerHandle,
    /// Active trackers being used by this tracker
    trackers: RwLock<Vec<Tracker>>,
    /// The torrent being tracked
    torrents: Mutex<Vec<TrackerTorrent>>,
    /// The timeout of tracker connections
    connection_timeout: Duration,
    /// The callbacks of the tracker
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

        self.announce_all_stopped().await;
        debug!("Tracker manager {} main loop has stopped", self);
    }

    /// Get all active trackers of the manager.
    async fn active_trackers<'a>(
        &self,
        trackers: &'a RwLockReadGuard<'_, Vec<Tracker>>,
    ) -> Vec<&'a Tracker> {
        future::join_all(
            trackers
                .iter()
                .map(|tracker| async move { (tracker, tracker.state().await) }),
        )
        .await
        .into_iter()
        .filter(|(_, state)| *state == TrackerState::Active)
        .map(|(tracker, _)| tracker)
        .collect()
    }

    async fn add_torrent(
        &self,
        peer_id: PeerId,
        peer_port: u16,
        info_hash: InfoHash,
    ) -> Result<()> {
        if peer_port == 0 {
            return Err(TrackerError::InvalidPort(peer_port));
        }

        let mut torrents = self.torrents.lock().await;

        // check if the given info hash if unique within the registered torrents
        // if not, we ignore this registration
        if !torrents.iter().any(|e| e.info_hash == info_hash) {
            let info_hash_txt = info_hash.to_string();
            torrents.push(TrackerTorrent {
                peer_id,
                peer_port,
                info_hash,
                peers: Default::default(),
                bytes_completed: Default::default(),
                bytes_remaining: u64::MAX,
            });
            debug!("Tracker manager {} added torrent {}", self, info_hash_txt);
        }

        Ok(())
    }

    async fn remove_torrent(&self, info_hash: &InfoHash) {
        let mut torrents = self.torrents.lock().await;

        if let Some(position) = torrents.iter().position(|e| &e.info_hash == info_hash) {
            let _ = torrents.remove(position);
            debug!("Tracker manager {} removed torrent {}", self, info_hash);
        }
    }

    async fn find_torrent<'a>(
        &self,
        info_hash: &InfoHash,
        torrents: &'a mut MutexGuard<'_, Vec<TrackerTorrent>>,
    ) -> Option<&'a mut TrackerTorrent> {
        torrents.iter_mut().find(|e| &e.info_hash == info_hash)
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
    async fn add_peers(&self, torrent: &mut TrackerTorrent, peers: &[SocketAddr]) -> usize {
        trace!("Discovered a total of {} peers, {:?}", peers.len(), peers);
        let mut unique_new_peer_addrs = Vec::new();

        for peer in peers.into_iter() {
            if !torrent.peers.contains(peer) {
                torrent.peers.insert(peer.clone());
                unique_new_peer_addrs.push(peer.clone());
            }
        }

        debug!(
            "Discovered a total of {} new peers",
            unique_new_peer_addrs.len()
        );
        let total_peers = unique_new_peer_addrs.len();
        if total_peers > 0 {
            self.send_event(TrackerManagerEvent::PeersDiscovered(
                torrent.info_hash.clone(),
                unique_new_peer_addrs,
            ))
            .await;
        }

        total_peers
    }

    async fn announce(
        &self,
        handle: TrackerHandle,
        info_hash: &InfoHash,
        event: AnnounceEvent,
    ) -> Result<AnnouncementResult> {
        let trackers = self.trackers.read().await;
        let mut torrents = self.torrents.lock().await;
        let tracker = trackers
            .iter()
            .find(|e| e.handle() == handle)
            .ok_or(TrackerError::InvalidHandle(handle))?;
        let torrent = self
            .find_torrent(info_hash, &mut torrents)
            .await
            .ok_or(TrackerError::InfoHashNotFound(info_hash.clone()))?;

        let result = self
            .announce_tracker(
                tracker,
                info_hash,
                event,
                torrent.peer_id,
                torrent.peer_port,
                torrent.bytes_completed,
                torrent.bytes_remaining,
            )
            .await
            .map(|e| AnnouncementResult {
                total_leechers: e.leechers,
                total_seeders: e.seeders,
                peers: e.peers,
            })?;
        self.add_peers(torrent, result.peers.as_slice()).await;

        Ok(result)
    }

    async fn announce_all(&self, info_hash: &InfoHash, event: AnnounceEvent) -> AnnouncementResult {
        let mut result = AnnouncementResult::default();
        let mut total_peers = 0;
        let trackers = self.trackers.read().await;
        let mut torrents = self.torrents.lock().await;

        if let Some(torrent) = self.find_torrent(info_hash, &mut torrents).await {
            // start announcing the given hash to each tracker simultaneously
            let futures: Vec<_> = self
                .active_trackers(&trackers)
                .await
                .iter()
                .map(|tracker| {
                    self.announce_tracker(
                        tracker,
                        info_hash,
                        event,
                        torrent.peer_id,
                        torrent.peer_port,
                        torrent.bytes_completed,
                        torrent.bytes_remaining,
                    )
                })
                .collect();

            // wait for all responses to complete
            let responses = future::join_all(futures).await;
            for response in responses {
                match response {
                    Ok(response) => {
                        result.total_leechers += response.leechers;
                        result.total_seeders += response.seeders;
                        result.peers.extend_from_slice(response.peers.as_slice());

                        total_peers += self.add_peers(torrent, response.peers.as_slice()).await;
                    }
                    Err(e) => debug!(
                        "Failed to announce info hash {:?} to tracker, {}",
                        info_hash, e
                    ),
                }
            }

            info!(
                "Discovered a total of {} peers for {}",
                total_peers, info_hash
            );
        } else {
            warn!(
                "Tracker {} failed to announce event, torrent {} info hash not found",
                self, info_hash
            );
        }

        result
    }

    async fn announce_all_stopped(&self) {
        let trackers = self.trackers.write().await;
        let torrents = self.torrents.lock().await;
        let mut futures = Vec::with_capacity(trackers.len() * torrents.len());

        for torrent in torrents.iter() {
            futures.extend(trackers.iter().map(|tracker| {
                self.announce_tracker(
                    tracker,
                    &torrent.info_hash,
                    AnnounceEvent::Stopped,
                    torrent.peer_id,
                    torrent.peer_port,
                    torrent.bytes_completed,
                    torrent.bytes_remaining,
                )
            }));
        }

        for response in future::join_all(futures).await {
            if let Err(e) = response {
                debug!("Failed announce stop event to tracker, {}", e);
            }
        }
    }

    async fn announce_tracker(
        &self,
        tracker: &Tracker,
        info_hash: &InfoHash,
        event: AnnounceEvent,
        peer_id: PeerId,
        peer_port: u16,
        bytes_completed: u64,
        bytes_remaining: u64,
    ) -> Result<AnnounceEntryResponse> {
        trace!("Announcing event {} to tracker {}", event, tracker);
        let announce = Announcement {
            info_hash: info_hash.clone(),
            peer_id,
            peer_port,
            event,
            bytes_completed,
            bytes_remaining,
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
        let trackers = self.trackers.read().await;
        let torrents = self.torrents.lock().await;
        let now = Instant::now();

        for torrent in torrents.iter() {
            for tracker in self.active_trackers(&trackers).await {
                let interval = tracker.announcement_interval().await;
                let last_announcement = tracker.last_announcement().await;
                let delta = now - last_announcement;

                if delta.as_secs() >= interval {
                    if let Err(err) = self
                        .announce_tracker(
                            tracker,
                            &torrent.info_hash,
                            AnnounceEvent::Started,
                            torrent.peer_id,
                            torrent.peer_port,
                            torrent.bytes_completed,
                            torrent.bytes_remaining,
                        )
                        .await
                    {
                        warn!("Failed make an announcement for {}, {}", tracker, err);
                    }
                }
            }
        }
    }
}

/// A torrent peer registered with the tracker.
#[derive(Debug, PartialEq)]
pub struct TrackerTorrent {
    /// The unique peer id of the torrent
    peer_id: PeerId,
    /// The port the torrent is listening on to accept incoming connections
    peer_port: u16,
    /// The torrent info hash for which this tracker manager is responsible for
    info_hash: InfoHash,
    /// The discovered peers for this torrent by the tracker
    peers: HashSet<SocketAddr>,
    /// The number completed piece bytes by the torrent
    bytes_completed: u64,
    /// The number of remaining piece bytes that need to be downloaded by the torrent
    bytes_remaining: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::init_logger;
    use crate::timeout;

    use std::str::FromStr;
    use tokio::sync::mpsc::unbounded_channel;
    use url::Url;

    mod add_torrent {
        use super::*;

        #[tokio::test]
        async fn test_valid_torrent() {
            init_logger!();
            let peer_id = PeerId::new();
            let peer_port = 6881;
            let info_hash =
                InfoHash::from_str("urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7").unwrap();
            let expected_result = TrackerTorrent {
                peer_id,
                peer_port,
                info_hash: info_hash.clone(),
                peers: Default::default(),
                bytes_completed: 0,
                bytes_remaining: u64::MAX,
            };
            let manager = TrackerManager::new(Duration::from_secs(1));

            {
                let result = manager
                    .add_torrent(peer_id, peer_port, info_hash.clone())
                    .await;
                assert_eq!(Ok(()), result);

                let torrents = manager.inner.torrents.lock().await;
                assert_eq!(
                    1,
                    torrents.len(),
                    "expected the torrent to have been registered"
                );
                assert_eq!(&expected_result, torrents.get(0).unwrap());
            }

            {
                let _ = manager
                    .add_torrent(PeerId::new(), peer_port, info_hash)
                    .await;
                let result = manager.inner.torrents.lock().await;
                assert_eq!(
                    1,
                    result.len(),
                    "expected the torrent to not have been added as duplicate"
                );
            }
        }

        #[tokio::test]
        async fn test_invalid_port() {
            init_logger!();
            let info_hash =
                InfoHash::from_str("urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7").unwrap();
            let manager = TrackerManager::new(Duration::from_secs(1));

            let result = manager.add_torrent(PeerId::new(), 0, info_hash).await;

            assert_eq!(Err(TrackerError::InvalidPort(0)), result);
        }
    }

    #[tokio::test]
    async fn test_add_tracker() {
        init_logger!();
        let url = Url::parse("udp://tracker.opentrackr.org:1337").unwrap();
        let entry = TrackerEntry { tier: 0, url };
        let manager = TrackerManager::new(Duration::from_secs(1));

        let result = manager.add_tracker_entry(entry).await;

        assert_eq!(
            None,
            result.err(),
            "expected the tracker to have been created"
        );
    }

    #[tokio::test]
    async fn test_remove_torrent() {
        init_logger!();
        let info_hash =
            InfoHash::from_str("urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7").unwrap();
        let manager = TrackerManager::new(Duration::from_secs(1));

        // try to remove a non-existing torrent
        manager.remove_torrent(&info_hash).await;

        {
            manager
                .add_torrent(PeerId::new(), 6881, info_hash.clone())
                .await
                .unwrap();
            let result = manager.inner.torrents.lock().await;
            assert_eq!(
                1,
                result.len(),
                "expected the torrent to have been registered"
            );
        }

        {
            manager.remove_torrent(&info_hash).await;
            let result = manager.inner.torrents.lock().await;
            assert_eq!(0, result.len(), "expected the torrent to have been removed");
        }
    }

    #[tokio::test]
    async fn test_tracker_manager_announce_all() {
        init_logger!();
        let url = Url::parse("udp://tracker.opentrackr.org:1337").unwrap();
        let peer_id = PeerId::new();
        let info_hash =
            InfoHash::from_str("urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7").unwrap();
        let entry = TrackerEntry { tier: 0, url };
        let manager = TrackerManager::new(Duration::from_secs(1));

        manager
            .add_torrent(peer_id, 6881, info_hash.clone())
            .await
            .unwrap();

        manager.add_tracker_entry(entry).await.unwrap();
        let result = manager
            .announce_all(&info_hash, AnnounceEvent::Started)
            .await;

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
        let manager = TrackerManager::new(Duration::from_secs(1));

        manager
            .add_torrent(peer_id, 6881, info_hash.clone())
            .await
            .unwrap();

        manager.add_tracker_entry(entry).await.unwrap();
        let result = manager
            .scrape(&info_hash)
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
        let entry = TrackerEntry { tier: 0, url };
        let (tx, mut rx) = unbounded_channel();
        let manager = TrackerManager::new(Duration::from_secs(1));

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

        let result = timeout!(
            rx.recv(),
            Duration::from_millis(200),
            "expected to receive an event"
        )
        .unwrap();
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
        let manager = TrackerManager::new(Duration::from_secs(1));

        manager
            .add_tracker_async(TrackerEntry { tier: 0, url })
            .await;
        drop(manager);
    }
}
