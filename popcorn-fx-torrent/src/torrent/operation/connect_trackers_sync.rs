use crate::torrent::tracker::TrackerEntry;
use crate::torrent::{TorrentCommandEvent, TorrentContext, TorrentOperation};
use async_trait::async_trait;
use derive_more::Display;
use log::{debug, warn};
use tokio::sync::Mutex;

/// The torrent trackers sync operation is responsible for adding the known trackers to the torrent.
/// This operation waits for the trackersto have established a connection before continuing.
#[derive(Debug, Display)]
#[display(fmt = "connect trackers synchronized operation")]
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
            .map(|(entry)| torrent.add_tracker(entry))
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
    async fn execute<'a>(&self, torrent: &'a TorrentContext) -> Option<&'a TorrentContext> {
        // add the known trackers to the torrent
        if !*self.initialized.lock().await {
            if !self.create_trackers(torrent).await {
                return None;
            }
        }

        Some(torrent)
    }

    fn clone_boxed(&self) -> Box<dyn TorrentOperation> {
        Box::new(TorrentTrackersSyncOperation::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torrent::fs::DefaultTorrentFileStorage;
    use crate::torrent::{Torrent, TorrentConfig, TorrentEvent, TorrentInfo};
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
        let operation = TorrentTrackersSyncOperation::new();

        let result = operation.execute(&*inner).await;
        assert_eq!(Some(&*inner), result);
    }
}
