use crate::torrents::trackers::TrackerEntry;
use crate::torrents::{TorrentCommandEvent, TorrentContext, TorrentOperation};
use async_trait::async_trait;
use derive_more::Display;
use log::{debug, warn};
use tokio::sync::Mutex;

/// The torrent trackers operation is responsible for adding the known trackers to the torrent.
/// This operation add the trackers in a "fire-and-forget" mode and only waits for one tracker connection to have been established.
#[derive(Debug, Display)]
#[display(fmt = "connect trackers operation")]
pub struct TorrentTrackersOperation {
    initialized: Mutex<bool>,
    cached_tiered_trackers: Mutex<Vec<TrackerEntry>>,
}

impl TorrentTrackersOperation {
    pub fn new() -> Self {
        Self {
            initialized: Default::default(),
            cached_tiered_trackers: Mutex::new(Vec::new()),
        }
    }

    /// Get the tiered trackers from the metadata of the torrent.
    /// Returns false if the tiered trackers could not be created.
    async fn create_trackers_cache(&self, torrent: &TorrentContext) -> bool {
        let metadata = torrent.metadata().await;
        let tiered_trackers = metadata.tiered_trackers();

        if tiered_trackers.is_empty() {
            warn!(
                "Unable to create tiered trackers for {}, no tiered trackers found in metadata",
                torrent
            );
            return false;
        }

        let tracker_entries = tiered_trackers
            .into_iter()
            .map(|(tier, trackers)| {
                trackers
                    .into_iter()
                    .map(|url| TrackerEntry { tier, url })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect();

        *self.cached_tiered_trackers.lock().await = tracker_entries;
        *self.initialized.lock().await = true;
        true
    }

    /// Try to add the trackers from the cache to the torrent.
    async fn add_trackers_from_cache(&self, torrent: &TorrentContext) {
        let mut mutex = self.cached_tiered_trackers.lock().await;
        let entries: Vec<_> = mutex.drain(..).collect();

        if entries.is_empty() {
            return;
        }

        let total_entries = entries.len();
        for entry in entries {
            torrent.send_command_event(TorrentCommandEvent::ConnectToTracker(entry));
        }

        debug!(
            "Queued a total of {} new trackers for {}",
            total_entries, torrent
        );
    }
}

#[async_trait]
impl TorrentOperation for TorrentTrackersOperation {
    async fn execute<'a>(&self, torrent: &'a TorrentContext) -> Option<&'a TorrentContext> {
        // build the tiered trackers cache if needed
        if !*self.initialized.lock().await {
            // if we're unable to create the tiered trackers
            // then stop the operation chain as we're unable to continue
            if !self.create_trackers_cache(&torrent).await {
                return None;
            }
        }

        self.add_trackers_from_cache(&torrent).await;
        // check if the metadata is known or if there are active tracker connections
        // if not, we wait for at least one tracker connection
        if torrent.metadata_lock().read().await.info.is_some()
            || torrent.active_tracker_connections().await > 0
        {
            Some(torrent)
        } else {
            None
        }
    }

    fn clone_boxed(&self) -> Box<dyn TorrentOperation> {
        Box::new(TorrentTrackersOperation::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torrents::fs::DefaultTorrentFileStorage;
    use crate::torrents::{Torrent, TorrentConfig, TorrentEvent, TorrentInfo};
    use popcorn_fx_core::available_port;
    use popcorn_fx_core::core::Callbacks;
    use popcorn_fx_core::testing::{init_logger, read_test_file_to_bytes};
    use std::sync::mpsc::channel;
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::runtime::Runtime;

    #[tokio::test]
    async fn test_execute() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent_info_data = read_test_file_to_bytes("ubuntu-https.torrent");
        let torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let port = available_port!(6881, 31000).unwrap();
        let torrent = Torrent::request()
            .metadata(torrent_info)
            .peer_listener_port(port)
            .config(
                TorrentConfig::builder()
                    .tracker_connection_timeout(Duration::from_secs(1))
                    .build(),
            )
            .storage(Box::new(DefaultTorrentFileStorage::new(temp_path)))
            .operations(vec![])
            .runtime(runtime.clone())
            .build()
            .unwrap();
        let inner = torrent.instance().unwrap();
        let (tx, rx) = channel();
        let operation = TorrentTrackersOperation::new();

        torrent.add_callback(Box::new(move |event| {
            if let TorrentEvent::TrackersChanged = event {
                tx.send(()).unwrap();
            }
        }));

        let result = operation.execute(&*inner).await;
        assert_eq!(None, result);

        let _ = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("expected a tracker connection to have been established");
        let result = operation.execute(&*inner).await;
        assert_eq!(Some(&*inner), result);
    }
}
