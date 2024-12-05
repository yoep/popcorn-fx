use crate::torrents::{TorrentCommandEvent, TorrentFlags};
use crate::torrents::{TorrentContext, TorrentOperation, TorrentState};
use async_trait::async_trait;
use derive_more::Display;
use log::trace;
use tokio::sync::Mutex;

/// The torrent metadata operation is responsible for checking if the metadata for a torrent is present and if not, retrieving it from peers.
#[derive(Debug, Display)]
#[display(fmt = "retrieve metadata operation")]
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
    async fn execute<'a>(&self, torrent: &'a TorrentContext) -> Option<&'a TorrentContext> {
        let is_metadata_known = self.is_metadata_known(&torrent).await;

        if is_metadata_known {
            return Some(torrent);
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

        None
    }

    fn clone_boxed(&self) -> Box<dyn TorrentOperation> {
        Box::new(TorrentMetadataOperation::new())
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
    use crate::torrents::fs::DefaultTorrentFileStorage;
    use crate::torrents::{Torrent, TorrentConfig, TorrentInfo};
    use popcorn_fx_core::core::torrents::magnet::Magnet;
    use popcorn_fx_core::testing::{init_logger, read_test_file_to_bytes};
    use std::str::FromStr;
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::runtime::Runtime;

    #[tokio::test]
    async fn test_execute_metadata_known() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent_info_data = read_test_file_to_bytes("debian-udp.torrent");
        let torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let torrent = Torrent::request()
            .metadata(torrent_info)
            .peer_listener_port(6881)
            .config(
                TorrentConfig::builder()
                    .peer_connection_timeout(Duration::from_secs(1))
                    .tracker_connection_timeout(Duration::from_secs(1))
                    .build(),
            )
            .storage(Box::new(DefaultTorrentFileStorage::new(temp_path)))
            .runtime(runtime.clone())
            .build()
            .unwrap();
        let operation = TorrentMetadataOperation::new();
        let inner = torrent.instance().unwrap();

        let result = operation.execute(&*inner).await;

        assert_eq!(Some(&*inner), result);
    }

    #[tokio::test]
    async fn test_execute_metadata_unknown_and_metadata_option_disabled() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let magnet = Magnet::from_str(uri).unwrap();
        let torrent_info = TorrentInfo::try_from(magnet).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let torrent = Torrent::request()
            .metadata(torrent_info)
            .options(TorrentFlags::None)
            .peer_listener_port(6881)
            .config(
                TorrentConfig::builder()
                    .peer_connection_timeout(Duration::from_secs(1))
                    .tracker_connection_timeout(Duration::from_secs(1))
                    .build(),
            )
            .storage(Box::new(DefaultTorrentFileStorage::new(temp_path)))
            .runtime(runtime.clone())
            .build()
            .unwrap();
        let inner = torrent.instance().unwrap();
        let operation = TorrentMetadataOperation::new();

        let result = operation.execute(&*inner).await;

        assert_eq!(None, result);
    }
}
