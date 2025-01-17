use crate::torrent::{TorrentContext, TorrentOperation, TorrentState};
use crate::torrent::{TorrentFlags, TorrentOperationResult};
use async_trait::async_trait;
use log::trace;
use std::sync::Arc;
use tokio::sync::Mutex;

/// The torrent metadata operation is responsible for checking if the metadata for a torrent is present and if not, retrieving it from peers.
#[derive(Debug)]
pub struct TorrentMetadataOperation {
    info: Mutex<MetadataInfo>,
}

impl TorrentMetadataOperation {
    pub fn new() -> Self {
        Self {
            info: Mutex::new(MetadataInfo {
                requesting_metadata: false,
                metadata_present: false,
            }),
        }
    }

    async fn is_metadata_known(&self, torrent: &TorrentContext) -> bool {
        // check if we've already checked for the presence of the metadata before
        // if so, use the cached information
        if self.info.lock().await.metadata_present {
            return true;
        }

        if torrent.metadata().await.info.is_some() {
            self.info.lock().await.metadata_present = true;
            return true;
        }

        false
    }

    async fn is_metadata_retrieval_enabled(&self, torrent: &TorrentContext) -> bool {
        let options = torrent.options_owned().await;
        options.contains(TorrentFlags::Metadata)
    }
}

#[async_trait]
impl TorrentOperation for TorrentMetadataOperation {
    fn name(&self) -> &str {
        "retrieve metadata operation"
    }

    async fn execute(&self, torrent: &Arc<TorrentContext>) -> TorrentOperationResult {
        let is_metadata_known = self.is_metadata_known(&torrent).await;

        if is_metadata_known {
            return TorrentOperationResult::Continue;
        }

        // check if the metadata should be retrieved from peers
        if self.is_metadata_retrieval_enabled(&torrent).await
            && !self.info.lock().await.requesting_metadata
        {
            // update the state of the torrent
            torrent.update_state(TorrentState::RetrievingMetadata).await;

            // check if there have been any peers discovered yet
            // if not, we want to retrieve the peers from trackers
            if torrent.discovered_peers().await.len() == 0 {
                trace!("No peers discovered yet, requesting from trackers");
                torrent.make_announce_all().await;
            }

            self.info.lock().await.requesting_metadata = true;
        }

        TorrentOperationResult::Stop
    }
}

#[derive(Debug)]
struct MetadataInfo {
    requesting_metadata: bool,
    metadata_present: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_torrent;
    use crate::torrent::TorrentConfig;
    use popcorn_fx_core::init_logger;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_execute_metadata_known() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![]
        );
        let context = torrent.instance().unwrap();
        let operation = TorrentMetadataOperation::new();

        let result = operation.execute(&context).await;

        assert_eq!(TorrentOperationResult::Continue, result);
    }

    #[tokio::test]
    async fn test_execute_metadata_unknown_and_metadata_option_disabled() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let torrent = create_torrent!(
            uri,
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![]
        );
        let context = torrent.instance().unwrap();
        let operation = TorrentMetadataOperation::new();

        let result = operation.execute(&context).await;

        assert_eq!(TorrentOperationResult::Stop, result);
    }
}
