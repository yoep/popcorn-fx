use std::fmt::{Debug, Formatter};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::torrents::peers::PeerId;
use crate::torrents::trackers::{
    AnnounceEntryResponse, Result, Tracker, TrackerError, TrackerHandle,
};
use crate::torrents::InfoHash;
use chrono::Utc;
use derive_more::Display;
use futures::future;
use log::{debug, info, trace, warn};
use popcorn_fx_core::core::block_in_place;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Sender;
use tokio::sync::{RwLock, RwLockReadGuard};
use tokio::{select, time};
use tokio_util::sync::CancellationToken;
use url::Url;

/// Kinds of tracker announces. This is typically indicated as the ``&event=``
/// HTTP query string parameter to HTTP trackers.
#[repr(u8)]
#[derive(Debug, Display, Clone)]
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
    /// * `info_hash` - The info hash of the torrent being tracked by this manager.
    /// * `timeout` - The timeout for tracker connections.
    /// * `event_sender` - The event sender to use for emitting events.
    /// * `runtime` - The runtime environment for spawning asynchronous tasks.
    ///
    /// # Returns
    ///
    /// A `TrackerManager` instance with initialized settings.
    pub fn new(
        peer_id: PeerId,
        peer_port: u16,
        info_hash: InfoHash,
        timeout: Duration,
        event_sender: Sender<TrackerManagerEvent>,
        runtime: Arc<Runtime>,
    ) -> Self {
        let inner = Arc::new(InnerTrackerManager::new(
            peer_id,
            peer_port,
            info_hash,
            timeout,
            event_sender,
        ));
        let cancellation_token = CancellationToken::new();

        Self {
            inner,
            cancellation_token,
            runtime,
        }
    }

    /// Checks if a given tracker URL is known within this manager.
    pub async fn is_tracker_url_known(&self, url: &Url) -> bool {
        let trackers = self.inner.trackers.read().await;
        trackers.iter().any(|e| e.url() == url)
    }

    /// Get the currently active trackers.
    /// This might return an empty list if no trackers have been added yet.
    pub async fn trackers(&self) -> Vec<Url> {
        let trackers = self.inner.trackers.read().await;
        trackers.iter().map(|e| e.url().clone()).collect()
    }

    /// Get the currently known peers that have been discovered by the trackers.
    pub async fn discovered_peers(&self) -> Vec<SocketAddr> {
        self.inner.peers.read().await.clone()
    }

    /// Adds a new tracker to the manager.
    ///
    /// # Arguments
    ///
    /// * `tracker` - The new tracker to add.
    ///
    /// # Returns
    ///
    /// Returns the created tracker handle on success, else the [TrackerError].
    pub async fn add_tracker(&self, url: &Url, tier: u8) -> Result<TrackerHandle> {
        let url_already_exists: bool;

        // check if the given url is already known for a tracker
        {
            let trackers = self.inner.trackers.read().await;
            url_already_exists = trackers.iter().any(|e| e.url() == url);
        }

        // if the url is already known, reject the request to create the tracker
        if url_already_exists {
            return Err(TrackerError::DuplicateUrl(url.clone()));
        }

        match Tracker::builder()
            .url(url.clone())
            .tier(tier)
            .timeout(self.inner.connection_timeout.clone())
            .peer_id(self.inner.peer_id.clone())
            .build()
            .await
        {
            Ok(tracker) => self.inner.add_tracker(tracker).await,
            Err(e) => {
                debug!("Failed to create new tracker {}: {}", url, e);
                Err(e)
            }
        }
    }

    /// Starts announcing to all trackers with the specified info hash.
    /// This will start automatic announcement loops for each known tracker within this manager.
    pub fn start_announcing(&self) {
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
            inner.announce_all().await;
        });
    }

    /// Announces to all trackers with the specified info hash.
    /// In regard to [self.start_announcing], this will not start an automatic announcement loop
    /// and is more a one of.
    ///
    /// # Returns
    ///
    /// Returns the announcement response result.
    pub async fn announce_all(&self) -> Announcement {
        let start_time = Instant::now();
        let result = self.inner.announce_all().await;
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
    ) -> Result<Announcement> {
        self.inner.announce(handle, event).await
    }

    /// Make a new announcement to the specified tracker for the given event.
    /// This method will spawn the announcement task and return immediately.
    pub async fn make_announcement(&self, handle: TrackerHandle, event: AnnounceEvent) {
        let inner = self.inner.clone();
        self.runtime.spawn(async move {
            let _ = inner.announce(handle, event).await;
        });
    }

    /// Announce to all trackers that the given torrent info hash has stopped.
    ///
    /// # Arguments
    ///
    /// * `info_hash` - The torrent info hash to announce
    pub async fn announce_stopped(&self) {
        self.cancellation_token.cancel();
        self.inner.announce_stopped().await;
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

        for tracker in mutex.as_mut_slice() {
            let interval = tracker.announcement_interval().await;
            let last_announcement = tracker.last_announcement().await;
            let delta = now.signed_duration_since(last_announcement);

            if delta.num_seconds() >= interval as i64 {
                match manager
                    .announce_tracker(tracker, AnnounceEvent::Started)
                    .await
                {
                    Ok(_) => {}
                    Err(e) => warn!("Failed make an announcement for {}, {}", tracker, e),
                }
            }
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
    /// The torrent info hash for which this tracker manager is responsible for
    info_hash: InfoHash,
    trackers: RwLock<Vec<Tracker>>,
    /// The discovered peers from the trackers
    peers: RwLock<Vec<SocketAddr>>,
    connection_timeout: Duration,
    event_sender: Sender<TrackerManagerEvent>,
}

impl InnerTrackerManager {
    fn new(
        peer_id: PeerId,
        peer_port: u16,
        info_hash: InfoHash,
        connection_timeout: Duration,
        event_sender: Sender<TrackerManagerEvent>,
    ) -> Self {
        Self {
            peer_id,
            peer_port,
            info_hash,
            trackers: Default::default(),
            peers: Default::default(),
            connection_timeout,
            event_sender,
        }
    }

    /// Get the info hash of the torrent being tracked.
    async fn info_hash(&self) -> InfoHash {
        self.info_hash.clone()
    }

    /// Add the given tracker to the trackers pool.
    /// Returns a unique tracker handle for the added tracker.
    async fn add_tracker(&self, tracker: Tracker) -> Result<TrackerHandle> {
        let handle = tracker.handle();
        let tracker_info = tracker.to_string();

        {
            let mut mutex = self.trackers.write().await;
            mutex.push(tracker);
            debug!("Tracker {} has been added", tracker_info);
        }

        self.send_event(TrackerManagerEvent::TrackerAdded(handle))
            .await;
        Ok(handle)
    }

    async fn add_peers(&self, peers: &[SocketAddr]) {
        trace!("Discovered a total of {} peers, {:?}", peers.len(), peers);
        let mut mutex = self.peers.write().await;
        let mut new_peers = Vec::new();

        for peer in peers.into_iter() {
            if !mutex.contains(&peer) {
                mutex.push(peer.clone());
                new_peers.push(peer.clone());
            }
        }

        debug!("Discovered a total of {} new peers", new_peers.len());
        if new_peers.len() > 0 {
            self.send_event(TrackerManagerEvent::PeersDiscovered(new_peers))
                .await;
        }
    }

    async fn announce(&self, handle: TrackerHandle, event: AnnounceEvent) -> Result<Announcement> {
        let mutex = self.trackers.read().await;
        let tracker = mutex
            .iter()
            .find(|e| e.handle() == handle)
            .ok_or(TrackerError::InvalidHandle(handle))?;

        match self
            .announce_tracker(tracker, event)
            .await
            .map(|e| Announcement {
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

    async fn announce_all(&self) -> Announcement {
        let mut result = Announcement::default();
        let mut mutex = self.trackers.write().await;
        let mut futures = Vec::new();
        let mut total_peers = 0;

        // start announcing the given hash to each tracker simultaneously
        for tracker in mutex.as_mut_slice() {
            futures.push(self.announce_tracker(tracker, AnnounceEvent::Started));
        }

        // wait for all responses to complete
        let responses = future::join_all(futures).await;
        for response in responses {
            match response {
                Ok(response) => {
                    result.total_leechers += response.leechers;
                    result.total_seeders += response.seeders;
                    result.peers.extend_from_slice(response.peers.as_slice());

                    let found_peers = response.peers.len();
                    self.add_peers(response.peers.as_slice()).await;
                    total_peers += found_peers;
                }
                Err(e) => debug!(
                    "Failed to announce info hash {:?} to tracker, {}",
                    self.info_hash, e
                ),
            }
        }

        info!("Found a total of {} peers", total_peers);
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
        match tracker
            .announce(self.info_hash.clone(), event.clone())
            .await
        {
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
        trace!("Sending event {:?}", event);
        if let Err(e) = self.event_sender.send(event).await {
            warn!(
                "Failed to send tracker manager event for peer {}, {}",
                self.peer_id, e
            );
        }
    }
}

impl Drop for InnerTrackerManager {
    fn drop(&mut self) {
        block_in_place(self.announce_stopped());
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use tokio::sync::oneshot::channel;
    use url::Url;

    use popcorn_fx_core::testing::init_logger;

    use super::*;

    #[test]
    fn test_add_tracker() {
        init_logger();
        let url = Url::parse("udp://tracker.opentrackr.org:1337").unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let peer_id = PeerId::new();
        let info_hash =
            InfoHash::from_str("urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7").unwrap();
        let (tx, _rx) = tokio::sync::mpsc::channel(1);
        let manager = TrackerManager::new(
            peer_id,
            6881,
            info_hash,
            Duration::from_secs(1),
            tx,
            runtime.clone(),
        );

        let result = runtime.block_on(manager.add_tracker(&url, 0));

        assert_eq!(
            None,
            result.err(),
            "expected the tracker to have been created"
        );
    }

    #[test]
    fn test_announce_all() {
        init_logger();
        let url = Url::parse("udp://tracker.opentrackr.org:1337").unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let peer_id = PeerId::new();
        let info_hash =
            InfoHash::from_str("urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7").unwrap();
        let (tx, _rx) = tokio::sync::mpsc::channel(1);
        let manager = TrackerManager::new(
            peer_id,
            6881,
            info_hash,
            Duration::from_secs(1),
            tx,
            runtime.clone(),
        );

        runtime.block_on(manager.add_tracker(&url, 0)).unwrap();
        let result = runtime.block_on(manager.announce_all());

        assert_ne!(0, result.peers.len(), "expected peers to have been found");
    }
}
