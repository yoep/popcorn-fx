use crate::torrent::tracker::TrackerEntry;
use crate::torrent::{TorrentContext, TorrentOperation, TorrentOperationResult};
use async_trait::async_trait;
use log::{debug, warn};
use std::sync::Arc;
use tokio::sync::Mutex;

/// The torrent trackers sync operation is responsible for adding the known trackers to the torrent.
/// This operation waits for the trackersto have established a connection before continuing.
#[derive(Debug)]
pub struct TorrentTrackersSyncOperation {
    initialized: Mutex<bool>,
}

impl TorrentTrackersSyncOperation {
    pub fn new() -> Self {
        Self {
            initialized: Default::default(),
        }
    }

    async fn create_trackers(&self, torrent: &TorrentContext) -> bool {
        let metadata = torrent.metadata().await;
        let tiered_trackers = metadata.tiered_trackers();

        if tiered_trackers.is_empty() {
            warn!(
                "Unable to create tiered trackers for {}, no tiered trackers found in metadata",
                torrent
            );
            return false;
        }

        let futures: Vec<_> = tiered_trackers
            .into_iter()
            .map(|(tier, urls)| {
                urls.into_iter()
                    .map(|url| TrackerEntry { tier, url })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .map(|entry| torrent.add_tracker(entry))
            .collect();

        let added_trackers = futures::future::join_all(futures)
            .await
            .into_iter()
            .map(|e| e.is_ok())
            .count();
        debug!("Added {} trackers to {}", added_trackers, torrent);
        *self.initialized.lock().await = true;
        true
    }
}

#[async_trait]
impl TorrentOperation for TorrentTrackersSyncOperation {
    fn name(&self) -> &str {
        "connect trackers synchronized operation"
    }

    async fn execute(&self, torrent: &Arc<TorrentContext>) -> TorrentOperationResult {
        // add the known trackers to the torrent
        if !*self.initialized.lock().await {
            if !self.create_trackers(torrent).await {
                return TorrentOperationResult::Stop;
            }
        }

        TorrentOperationResult::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_torrent;
    use crate::torrent::{TorrentConfig, TorrentFlags};
    use popcorn_fx_core::init_logger;
    use tempfile::tempdir;

    #[test]
    fn test_execute() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "ubuntu-https.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![]
        );
        let context = torrent.instance().unwrap();
        let runtime = context.runtime();
        let operation = TorrentTrackersSyncOperation::new();

        runtime.block_on(async {
            let result = operation.execute(&context).await;
            assert_eq!(TorrentOperationResult::Continue, result);

            let result = context.announce_all().await;
            assert_ne!(
                0,
                result.peers.len(),
                "expected to have discovered some peers"
            );
        });
    }
}
