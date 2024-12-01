use crate::torrents::trackers::TrackerEntry;
use crate::torrents::{InnerTorrent, TorrentCommandEvent, TorrentOperation};
use async_trait::async_trait;
use derive_more::Display;
use log::{debug, warn};
use tokio::sync::Mutex;

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
    async fn create_trackers_cache(&self, torrent: &InnerTorrent) -> bool {
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
    async fn add_trackers_from_cache(&self, torrent: &InnerTorrent) {
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
    async fn execute<'a>(&self, torrent: &'a InnerTorrent) -> Option<&'a InnerTorrent> {
        // build the tiered trackers cache if needed
        if !*self.initialized.lock().await {
            // if we're unable to create the tiered trackers
            // then stop the operation chain as we're unable to continue
            if !self.create_trackers_cache(&torrent).await {
                return None;
            }
        }

        self.add_trackers_from_cache(&torrent).await;
        Some(torrent)
    }

    fn clone_boxed(&self) -> Box<dyn TorrentOperation> {
        Box::new(TorrentTrackersOperation::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torrents::fs::DefaultTorrentFileStorage;
    use crate::torrents::{Torrent, TorrentConfig, TorrentInfo};
    use popcorn_fx_core::testing::{init_logger, read_test_file_to_bytes};
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
        let torrent = Torrent::request()
            .metadata(torrent_info)
            .peer_listener_port(9090)
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
        let operation = TorrentTrackersOperation::new();

        let result = operation.execute(&*inner).await;

        assert_eq!(Some(&*inner), result);
    }
}
