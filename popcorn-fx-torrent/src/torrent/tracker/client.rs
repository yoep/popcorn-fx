use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::torrent::metrics::Metric;
use crate::torrent::peer::PeerId;
use crate::torrent::tracker::{
    AnnounceEntryResponse, AnnounceEvent, Announcement, Result, ScrapeFileMetrics, ScrapeResult,
    Tracker, TrackerClientMetrics, TrackerError, TrackerHandle, TrackerState,
};
use crate::torrent::{InfoHash, Metrics};
use derive_more::Display;
use futures::future;
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use log::{debug, info, trace, warn};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::{Mutex, RwLock, RwLockReadGuard};
use tokio::{select, time};
use tokio_util::sync::CancellationToken;
use url::Url;

const DEFAULT_ANNOUNCEMENT_INTERVAL: Duration = Duration::from_secs(30);
const STATS_INTERVAL: Duration = Duration::from_secs(1);

/// Aggregated announcement result returned by one or more trackers.
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
    /// Returns the total number of peers reported by the trackers.
    ///
    /// This is simply the length of [`Self::peers`].
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
    /// The tier of the tracker.
    ///
    /// Lower values indicate higher priority.
    pub tier: u8,
    /// The tracker url to connect to.
    pub url: Url,
}

/// The event that can be emitted by the tracker client.
#[derive(Debug, Clone)]
pub enum TrackerClientEvent {
    /// Emitted when new peers have been discovered for a torrent.
    ///
    /// Contains the [`InfoHash`] of the torrent and the list of newly discovered peer addresses.
    PeersDiscovered(InfoHash, Vec<SocketAddr>),
    /// Emitted when a new tracker has been added to the client.
    ///
    /// Contains the handle of the added tracker.
    TrackerAdded(TrackerHandle),
    /// Emitted when the tracker client's metric stats are updated.
    ///
    /// This is emitted periodically based on [`STATS_INTERVAL`].
    Stats(TrackerClientMetrics),
}

impl PartialEq for TrackerClientEvent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                TrackerClientEvent::PeersDiscovered(a, _),
                TrackerClientEvent::PeersDiscovered(b, _),
            ) => a == b,
            (TrackerClientEvent::TrackerAdded(a), TrackerClientEvent::TrackerAdded(b)) => a == b,
            (TrackerClientEvent::Stats(_), TrackerClientEvent::Stats(_)) => true,
            _ => false,
        }
    }
}

/// A tracker client that manages communication with one or more trackers
/// for a set of torrents.
///
/// It allows registering torrents, adding trackers, announcing events,
/// scraping statistics, and retrieving discovered peers.
#[derive(Debug, Display, Clone)]
#[display(fmt = "{}", inner)]
pub struct TrackerClient {
    inner: Arc<InnerClient>,
}

impl TrackerClient {
    /// Creates a new [`TrackerClient`] instance.
    ///
    /// # Arguments
    ///
    /// * `connection_timeout` - The timeout for tracker connections.
    ///
    /// # Returns
    ///
    /// A [`TrackerClient`] instance with its internal event loop spawned.
    pub fn new(connection_timeout: Duration) -> Self {
        let (command_sender, command_receiver) = unbounded_channel();
        let inner = Arc::new(InnerClient {
            handle: Default::default(),
            trackers: Default::default(),
            torrents: Default::default(),
            connection_timeout,
            command_sender,
            callbacks: MultiThreadedCallback::new(),
            metrics: Default::default(),
            cancellation_token: Default::default(),
        });

        let inner_main_loop = inner.clone();
        tokio::spawn(async move {
            inner_main_loop.start(command_receiver).await;
        });

        Self { inner }
    }

    /// Returns the aggregated metric stats of this tracker client.
    pub fn metrics(&self) -> &TrackerClientMetrics {
        &self.inner.metrics
    }

    /// Returns the tracker corresponding to the given handle, if any.
    pub async fn get(&self, handle: &TrackerHandle) -> Option<Tracker> {
        self.inner
            .trackers
            .read()
            .await
            .iter()
            .find(|e| &e.handle() == handle)
            .cloned()
    }

    /// Checks if a given tracker URL is known within this client.
    pub async fn is_tracker_url_known(&self, url: &Url) -> bool {
        self.inner.is_tracker_url_known(url).await
    }

    /// Returns the URLs of all trackers managed by this client.
    ///
    /// This might return an empty list if no trackers have been added yet.
    pub async fn tracker_urls(&self) -> Vec<Url> {
        let trackers = self.inner.trackers.read().await;
        trackers.iter().map(|e| e.url().clone()).collect()
    }

    /// Returns all trackers managed by this client.
    ///
    /// This might return an empty list if no trackers have been added yet.
    pub async fn trackers(&self) -> Vec<Tracker> {
        self.inner.trackers.read().await.clone()
    }

    /// Returns the number of trackers managed by this client.
    ///
    /// This might return `0` if no trackers have been added yet.
    pub async fn trackers_len(&self) -> usize {
        self.inner.trackers.read().await.len()
    }

    /// Returns the number of torrents tracked by this client.
    ///
    /// This might return `0` if no torrents have been added yet.
    pub async fn torrents_len(&self) -> usize {
        self.inner.torrents.lock().await.len()
    }

    /// Registers a new torrent with the tracker client so that it can discover peers.
    ///
    /// If a torrent with the same [`InfoHash`] is already registered,
    /// this call is effectively a no-op.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The peer ID of the client.
    /// * `peer_port` - The port on which the client is listening.
    /// * `info_hash` - The info hash of the torrent.
    /// * `metrics` - The metrics of the torrent.
    pub async fn add_torrent(
        &self,
        peer_id: PeerId,
        peer_port: u16,
        info_hash: InfoHash,
        metrics: Metrics,
    ) -> Result<()> {
        self.inner
            .add_torrent(peer_id, peer_port, info_hash, metrics)
            .await
    }

    /// Removes the given torrent [`InfoHash`] from the tracker client.
    ///
    /// This stops tracking the torrent and clears any discovered peers for it.
    ///
    /// # Arguments
    ///
    /// * `info_hash` - The info hash of the torrent.
    pub fn remove_torrent(&self, info_hash: &InfoHash) {
        let _ = self
            .inner
            .command_sender
            .send(TrackerClientCommand::RemoveTorrent(info_hash.clone()));
    }

    /// Returns the discovered peers for the given info hash.
    ///
    /// The info hash should first be registered through [`TrackerClient::add_torrent`].
    pub async fn discovered_peers(&self, info_hash: &InfoHash) -> Option<Vec<SocketAddr>> {
        self.inner
            .torrents
            .lock()
            .await
            .get(info_hash)
            .map(|e| e.peers.iter().cloned().collect())
    }

    /// Adds a new tracker to the client.
    ///
    /// # Arguments
    ///
    /// * `entry` - The tracker entry describing the URL and tier.
    ///
    /// # Returns
    ///
    /// Returns the created tracker handle on success, otherwise a [`TrackerError`].
    pub async fn add_tracker_entry(&self, entry: TrackerEntry) -> Result<TrackerHandle> {
        self.inner.create_tracker_from_entry(entry).await
    }

    /// Adds a new tracker to the client on a background task.
    ///
    /// This is the asynchronous "fire-and-forget" variant of
    /// [`TrackerClient::add_tracker_entry`]. Any error is only logged.
    ///
    /// # Arguments
    ///
    /// * `entry` - The tracker entry describing the URL and tier.
    pub async fn add_tracker_async(&self, entry: TrackerEntry) {
        let inner = self.inner.clone();
        tokio::spawn(async move {
            select! {
                _ = inner.cancellation_token.cancelled() => return,
                _ = inner.create_tracker_from_entry(entry) => return,
            }
        });
    }

    /// Announces an event for the given torrent to all trackers.
    ///
    /// # Returns
    ///
    /// Returns the aggregated announcement response result from all trackers.
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

    /// Announces the given event for a specific torrent to the specified tracker.
    ///
    /// # Returns
    ///
    /// Returns the announcement result from that tracker or a [`TrackerError`].
    pub async fn announce(
        &self,
        handle: TrackerHandle,
        info_hash: &InfoHash,
        event: AnnounceEvent,
    ) -> Result<AnnouncementResult> {
        self.inner.announce(handle, info_hash, event).await
    }

    /// Announces an event for the given torrent to all trackers.
    ///
    /// This method will spawn the announcement task and return immediately
    /// without waiting for the responses.
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

    /// Announces an event for the given torrent to the specified tracker.
    ///
    /// This method will spawn the announcement task and return immediately
    /// without waiting for the response.
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

    /// Scrapes all trackers for stats about the given [`InfoHash`].
    ///
    /// The results from all trackers are aggregated per file.
    ///
    /// # Errors
    ///
    /// Returns [`TrackerError::NoTrackers`] if no trackers are registered.
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

    /// Starts automatically announcing the torrent with the given info hash.
    ///
    /// This re-enables periodic announcements if they were previously stopped
    /// via [`TrackerClient::stop_announcing`].
    pub fn start_announcing(&self, info_hash: &InfoHash) {
        let _ = self
            .inner
            .command_sender
            .send(TrackerClientCommand::StartAnnouncing(info_hash.clone()));
    }

    /// Stops automatically announcing the torrent with the given info hash.
    ///
    /// This does not remove the torrent from the tracker client,
    /// but temporarily disables any new automatic announcements.
    /// Use [`TrackerClient::start_announcing`] to enable automatic announcements again.
    pub fn stop_announcing(&self, info_hash: &InfoHash) {
        let _ = self
            .inner
            .command_sender
            .send(TrackerClientCommand::StopAnnouncing(info_hash.clone()));
    }

    /// Closes the tracker client, resulting in termination of its operations.
    ///
    /// This cancels the internal event loop and closes all managed tracker connections.
    pub fn close(&self) {
        self.inner.cancellation_token.cancel();
    }
}

impl Callback<TrackerClientEvent> for TrackerClient {
    fn subscribe(&self) -> Subscription<TrackerClientEvent> {
        self.inner.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<TrackerClientEvent>) {
        self.inner.callbacks.subscribe_with(subscriber)
    }
}

#[derive(Debug, PartialEq)]
enum TrackerClientCommand {
    StartAnnouncing(InfoHash),
    StopAnnouncing(InfoHash),
    RemoveTorrent(InfoHash),
}

/// Inner implementation of the tracker client.
///
/// This type is reference-counted and shared by the public [`TrackerClient`].
#[derive(Debug, Display)]
#[display(fmt = "{}", handle)]
struct InnerClient {
    /// The unique handle of this client.
    handle: TrackerHandle,
    /// Active trackers managed by this client.
    trackers: RwLock<Vec<Tracker>>,
    /// The torrents being tracked by this client.
    torrents: Mutex<HashMap<InfoHash, TrackerTorrent>>,
    /// The timeout for tracker connections.
    connection_timeout: Duration,
    /// Command sender for handling asynchronous tasks in the client loop.
    command_sender: UnboundedSender<TrackerClientCommand>,
    /// Callback dispatcher used to notify subscribers of client events.
    callbacks: MultiThreadedCallback<TrackerClientEvent>,
    /// Aggregated tracker client metrics.
    metrics: TrackerClientMetrics,
    /// Cancellation token used to stop the client event loop and background tasks.
    cancellation_token: CancellationToken,
}

impl InnerClient {
    /// Starts the main event loop of the tracker client.
    ///
    /// This loop processes commands, performs automatic announcements, and updates stats
    /// until the cancellation token is triggered.
    async fn start(&self, mut command_receiver: UnboundedReceiver<TrackerClientCommand>) {
        let mut announcement_tick = time::interval(DEFAULT_ANNOUNCEMENT_INTERVAL);
        let mut stats_interval = time::interval(STATS_INTERVAL);

        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(command) = command_receiver.recv() => self.handle_command(command).await,
                _ = announcement_tick.tick() => self.do_automatic_announcements().await,
                _ = stats_interval.tick() => self.update_stats().await,
            }
        }

        self.announce_all_stopped().await;
        self.close().await;
        debug!("Tracker client {} main loop has stopped", self);
    }

    async fn handle_command(&self, command: TrackerClientCommand) {
        match command {
            TrackerClientCommand::StartAnnouncing(info_hash) => {
                self.update_torrent_announcing_state(info_hash, true).await
            }
            TrackerClientCommand::StopAnnouncing(info_hash) => {
                self.update_torrent_announcing_state(info_hash, false).await
            }
            TrackerClientCommand::RemoveTorrent(info_hash) => self.remove_torrent(info_hash).await,
        }
    }

    async fn update_torrent_announcing_state(&self, info_hash: InfoHash, is_announcing: bool) {
        let mut torrents = self.torrents.lock().await;

        if let Some(torrent) = torrents.get_mut(&info_hash) {
            torrent.is_announcing = is_announcing;
        }
    }

    /// Get all active trackers of the client.
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
        metrics: Metrics,
    ) -> Result<()> {
        if peer_port == 0 {
            return Err(TrackerError::InvalidPort(peer_port));
        }

        let mut torrents = self.torrents.lock().await;

        // check if the given info hash if unique within the registered torrents
        // if not, we ignore this registration
        if !torrents.contains_key(&info_hash) {
            let info_hash_txt = info_hash.to_string();
            torrents.insert(
                info_hash.clone(),
                TrackerTorrent {
                    peer_id,
                    peer_port,
                    metrics,
                    peers: Default::default(),
                    is_announcing: true,
                },
            );
            debug!("Tracker client {} added torrent {}", self, info_hash_txt);
        }

        Ok(())
    }

    async fn remove_torrent(&self, info_hash: InfoHash) {
        let mut torrents = self.torrents.lock().await;

        if let Some(_) = torrents.remove(&info_hash) {
            debug!("Tracker client {} removed torrent {}", self, info_hash);
        }
    }

    /// Checks if the given URL is already registered/known.
    async fn is_tracker_url_known(&self, url: &Url) -> bool {
        let trackers = self.trackers.read().await;
        trackers.iter().any(|e| e.url() == url)
    }

    /// Tries to create a new tracker for the given entry.
    ///
    /// The URL of the entry must not already be known. On success,
    /// the created tracker is added to the client.
    ///
    /// # Returns
    ///
    /// Returns the created tracker handle on success, otherwise a [`TrackerError`].
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

    /// Adds the given tracker to the tracker's pool.
    ///
    /// # Returns
    ///
    /// Returns a unique tracker handle for the added tracker.
    async fn add_tracker(&self, tracker: Tracker) -> Result<TrackerHandle> {
        let handle = tracker.handle();
        let tracker_info = tracker.to_string();

        {
            let mut mutex = self.trackers.write().await;
            mutex.push(tracker);
            debug!("Tracker {} has been added to {}", tracker_info, self);
        }

        self.send_event(TrackerClientEvent::TrackerAdded(handle));
        Ok(handle)
    }

    /// Adds one or more discovered peers to the tracker client.
    ///
    /// This will only add unique peer addresses and filters out any duplicate
    /// addresses that have already been discovered for the torrent.
    ///
    /// # Returns
    ///
    /// The number of newly added unique peer addresses.
    async fn add_peers(
        &self,
        info_hash: &InfoHash,
        torrent: &mut TrackerTorrent,
        peers: &[SocketAddr],
    ) -> usize {
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
            self.send_event(TrackerClientEvent::PeersDiscovered(
                info_hash.clone(),
                unique_new_peer_addrs,
            ));
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
        let torrent = torrents
            .get_mut(info_hash)
            .ok_or(TrackerError::InfoHashNotFound(info_hash.clone()))?;

        let result = self
            .announce_tracker(
                tracker,
                info_hash,
                event,
                torrent.peer_id,
                torrent.peer_port,
                torrent.metrics.completed_size.total(),
                torrent.metrics.bytes_remaining(),
            )
            .await
            .map(|e| AnnouncementResult {
                total_leechers: e.leechers,
                total_seeders: e.seeders,
                peers: e.peers,
            })?;
        self.add_peers(info_hash, torrent, result.peers.as_slice())
            .await;

        Ok(result)
    }

    async fn announce_all(&self, info_hash: &InfoHash, event: AnnounceEvent) -> AnnouncementResult {
        let mut result = AnnouncementResult::default();
        let mut total_peers = 0;
        let trackers = self.trackers.read().await;
        let mut torrents = self.torrents.lock().await;

        if let Some(torrent) = torrents.get_mut(info_hash) {
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
                        torrent.metrics.completed_size.total(),
                        torrent.metrics.bytes_remaining(),
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

                        total_peers += self
                            .add_peers(info_hash, torrent, response.peers.as_slice())
                            .await;
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

        for (info_hash, torrent) in torrents.iter() {
            futures.extend(trackers.iter().map(|tracker| {
                self.announce_tracker(
                    tracker,
                    &info_hash,
                    AnnounceEvent::Stopped,
                    torrent.peer_id,
                    torrent.peer_port,
                    torrent.metrics.completed_size.total(),
                    torrent.metrics.bytes_remaining(),
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

    fn send_event(&self, event: TrackerClientEvent) {
        self.callbacks.invoke(event);
    }

    /// Performs automatic announcements to all trackers periodically.
    ///
    /// This method is called by the periodic task loop and respects the
    /// per-tracker announcement interval and last announcement timestamp.
    async fn do_automatic_announcements(&self) {
        let trackers = self.trackers.read().await;
        let torrents = self.torrents.lock().await;
        let now = Instant::now();

        for (info_hash, torrent) in torrents.iter() {
            for tracker in self.active_trackers(&trackers).await {
                let interval = tracker.announcement_interval().await;
                let last_announcement = tracker.last_announcement().await;
                let delta = now - last_announcement;

                if delta.as_secs() >= interval {
                    if let Err(err) = self
                        .announce_tracker(
                            tracker,
                            &info_hash,
                            AnnounceEvent::Started,
                            torrent.peer_id,
                            torrent.peer_port,
                            torrent.metrics.completed_size.total(),
                            torrent.metrics.bytes_remaining(),
                        )
                        .await
                    {
                        warn!("Failed make an announcement for {}, {}", tracker, err);
                    }
                }
            }
        }
    }

    /// Updates metrics and emits a [`TrackerClientEvent::Stats`] event.
    ///
    /// This aggregates tracker metrics into the client metrics and ticks
    /// both tracker and client metric time windows.
    async fn update_stats(&self) {
        for tracker in self.trackers.read().await.iter() {
            let tracker_metrics = tracker.metrics();

            self.metrics.bytes_in.inc_by(tracker_metrics.bytes_in.get());
            self.metrics
                .bytes_out
                .inc_by(tracker_metrics.bytes_out.get());

            tracker.tick(STATS_INTERVAL);
        }

        self.send_event(TrackerClientEvent::Stats(self.metrics.snapshot()));
        self.metrics.tick(STATS_INTERVAL);
    }

    /// Closes all tracker connections managed by this client.
    async fn close(&self) {
        for tracker in self.trackers.write().await.iter() {
            tracker.close().await;
        }
    }
}

/// A torrent peer registered with the tracker.
#[derive(Debug, PartialEq)]
struct TrackerTorrent {
    /// The unique peer id of the torrent
    peer_id: PeerId,
    /// The port the torrent is listening on to accept incoming connections
    peer_port: u16,
    /// The discovered peers for this torrent by the tracker
    peers: HashSet<SocketAddr>,
    /// A reference to the torrent metrics
    metrics: Metrics,
    /// Indicates if the torrent peer should be announced to the trackers.
    is_announcing: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::timeout;
    use crate::{assert_timeout, init_logger};

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
                peers: Default::default(),
                metrics: Metrics::new(),
                is_announcing: true,
            };
            let manager = TrackerClient::new(Duration::from_secs(1));

            {
                let result = manager
                    .add_torrent(peer_id, peer_port, info_hash.clone(), Metrics::new())
                    .await;
                assert_eq!(Ok(()), result);

                let torrents = manager.inner.torrents.lock().await;
                assert_eq!(
                    1,
                    torrents.len(),
                    "expected the torrent to have been registered"
                );
                assert_eq!(&expected_result, torrents.get(&info_hash).unwrap());
            }

            {
                let _ = manager
                    .add_torrent(PeerId::new(), peer_port, info_hash, Metrics::new())
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
            let manager = TrackerClient::new(Duration::from_secs(1));

            let result = manager
                .add_torrent(PeerId::new(), 0, info_hash, Metrics::new())
                .await;

            assert_eq!(Err(TrackerError::InvalidPort(0)), result);
        }
    }

    #[tokio::test]
    async fn test_add_tracker() {
        init_logger!();
        let url = Url::parse("udp://tracker.opentrackr.org:1337").unwrap();
        let entry = TrackerEntry { tier: 0, url };
        let manager = TrackerClient::new(Duration::from_secs(1));

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
        let manager = TrackerClient::new(Duration::from_secs(1));

        // try to remove a non-existing torrent
        manager.remove_torrent(&info_hash);

        {
            manager
                .add_torrent(PeerId::new(), 6881, info_hash.clone(), Metrics::new())
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
            manager.remove_torrent(&info_hash);
            assert_timeout!(
                Duration::from_millis(500),
                manager.inner.torrents.lock().await.len() == 0,
                "expected the torrent to have been removed"
            );
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
        let manager = TrackerClient::new(Duration::from_secs(1));

        manager
            .add_torrent(peer_id, 6881, info_hash.clone(), Metrics::new())
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
        let manager = TrackerClient::new(Duration::from_secs(1));

        manager
            .add_torrent(peer_id, 6881, info_hash.clone(), Metrics::new())
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
    async fn test_discovered_peers() {
        init_logger!();
        let info_hash =
            InfoHash::from_str("urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7").unwrap();
        let peer_addr = SocketAddr::from(([127, 0, 0, 1], 6882));
        let manager = TrackerClient::new(Duration::from_secs(1));

        manager.inner.torrents.lock().await.insert(
            info_hash.clone(),
            TrackerTorrent {
                peer_id: PeerId::new(),
                peer_port: 6881,
                peers: vec![peer_addr.clone()].into_iter().collect(),
                metrics: Metrics::default(),
                is_announcing: true,
            },
        );

        let result = manager.discovered_peers(&info_hash).await;
        assert_eq!(
            true,
            result.is_some(),
            "expected the info hash to have been found"
        );
        let result = result.unwrap();
        assert_eq!(
            Some(&peer_addr),
            result.get(0),
            "expected a discovered peer to have been returned"
        );
    }

    #[tokio::test]
    async fn test_add_callback() {
        init_logger!();
        let url = Url::parse("udp://tracker.opentrackr.org:1337").unwrap();
        let entry = TrackerEntry { tier: 0, url };
        let (tx, mut rx) = unbounded_channel();
        let manager = TrackerClient::new(Duration::from_secs(1));

        let mut receiver = manager.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                if let TrackerClientEvent::TrackerAdded(_) = &*event {
                    tx.send((*event).clone()).unwrap();
                    break;
                }
            }
        });

        manager
            .add_tracker_entry(entry)
            .await
            .expect("expected the tracker to have been created");

        let result = timeout!(
            rx.recv(),
            Duration::from_millis(750),
            "expected to receive an event"
        )
        .unwrap();
        if let TrackerClientEvent::TrackerAdded(handle) = result {
            let result = manager
                .trackers()
                .await
                .into_iter()
                .any(|e| e.handle() == handle);
            assert!(
                result,
                "expected tracker {} to have been present within the managed",
                handle
            );
        } else {
            assert!(
                false,
                "expected TrackerManagerEvent::TrackerAdded, got {:?} instead",
                result
            );
        }
    }

    #[tokio::test]
    async fn test_start_stop_announcing() {
        init_logger!();
        let peer_id = PeerId::new();
        let peer_port = 6881;
        let info_hash =
            InfoHash::from_str("urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7").unwrap();
        let manager = TrackerClient::new(Duration::from_secs(1));

        let result = manager
            .add_torrent(peer_id, peer_port, info_hash.clone(), Metrics::new())
            .await;
        assert_eq!(Ok(()), result, "expected the torrent to have been added");

        manager.stop_announcing(&info_hash);
        assert_timeout!(
            Duration::from_millis(500),
            manager
                .inner
                .torrents
                .lock()
                .await
                .get(&info_hash)
                .unwrap()
                .is_announcing
                == false,
            "expected the torrent to be no longer announcing"
        );

        manager.start_announcing(&info_hash);
        assert_timeout!(
            Duration::from_millis(500),
            manager
                .inner
                .torrents
                .lock()
                .await
                .get(&info_hash)
                .unwrap()
                .is_announcing
                == true,
            "expected the torrent to be no longer announcing"
        );
    }

    #[tokio::test]
    async fn test_drop() {
        init_logger!();
        let url = Url::parse("udp://tracker.opentrackr.org:1337").unwrap();
        let manager = TrackerClient::new(Duration::from_secs(1));

        manager
            .add_tracker_async(TrackerEntry { tier: 0, url })
            .await;
        drop(manager);
    }
}
