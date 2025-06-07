use crate::torrent::tracker::TrackerEntry;
use crate::torrent::{
    TorrentCommandEvent, TorrentContext, TorrentOperation, TorrentOperationResult, TorrentState,
};
use async_trait::async_trait;
use log::{debug, warn};
use std::sync::Arc;
use tokio::sync::Mutex;

/// The torrent trackers operation is responsible for adding the known trackers to the torrent.
/// This operation add the trackers in a "fire-and-forget" mode and only waits for one tracker connection to have been established.
#[derive(Debug)]
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
            torrent.send_command_event(TorrentCommandEvent::State(TorrentState::Error));
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
    fn name(&self) -> &str {
        "connect trackers operation"
    }

    async fn execute(&self, torrent: &Arc<TorrentContext>) -> TorrentOperationResult {
        // build the tiered trackers cache if needed
        if !*self.initialized.lock().await {
            // if we're unable to create the tiered trackers
            // then stop the operation chain as we're unable to continue
            if !self.create_trackers_cache(&torrent).await {
                return TorrentOperationResult::Stop;
            }
        }

        self.add_trackers_from_cache(&torrent).await;
        // check if the metadata is known or if there are active tracker connections
        // if not, we wait for at least one tracker connection
        if torrent.metadata_lock().read().await.info.is_some()
            || torrent.active_tracker_connections().await > 0
        {
            TorrentOperationResult::Continue
        } else {
            TorrentOperationResult::Stop
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::torrent::{TorrentConfig, TorrentEvent, TorrentFlags};
    use crate::{create_torrent, timeout};

    use fx_callback::Callback;
    use popcorn_fx_core::init_logger;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::sync::mpsc::unbounded_channel;

    #[tokio::test]
    async fn test_execute_metadata_info_unknown() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let uri = "magnet:?xt=urn:btih:2C6B6858D61DA9543D4231A71DB4B1C9264B0685&dn=Ubuntu%2022.04%20LTS&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let torrent = create_torrent!(
            uri,
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![],
            vec![]
        );
        let inner = torrent.instance().unwrap();
        let (tx, mut rx) = unbounded_channel();
        let operation = TorrentTrackersOperation::new();

        let mut receiver = torrent.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                if let TorrentEvent::TrackersChanged = *event {
                    tx.send(()).unwrap();
                    break;
                }
            }
        });

        let result = operation.execute(&inner).await;
        assert_eq!(TorrentOperationResult::Stop, result, "expected the chain to stop if the metadata is unknown and no tracker connections have yet been established");

        timeout!(
            rx.recv(),
            Duration::from_secs(2),
            "expected a tracker connection to have been established"
        )
        .unwrap();
        let result = operation.execute(&inner).await;
        assert_eq!(TorrentOperationResult::Continue, result);
    }

    #[tokio::test]
    async fn test_execute_metadata_info_known() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let uri = "debian-udp.torrent";
        let torrent = create_torrent!(
            uri,
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![],
            vec![]
        );
        let inner = torrent.instance().unwrap();
        let operation = TorrentTrackersOperation::new();

        let result = operation.execute(&inner).await;
        assert_eq!(
            TorrentOperationResult::Continue,
            result,
            "expected the chain to continue if the metadata info is known"
        );
    }
}
