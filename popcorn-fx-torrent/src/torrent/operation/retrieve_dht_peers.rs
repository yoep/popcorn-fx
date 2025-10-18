use crate::torrent::{TorrentContext, TorrentOperation, TorrentOperationResult};
use async_trait::async_trait;
use log::{debug, trace};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const RETRIEVE_INTERVAL: Duration = Duration::from_secs(90);
const RETRIEVE_TIMEOUT: Duration = Duration::from_secs(3);

/// Retrieve potential peer addresses for the torrent through the DHT network.
#[derive(Debug)]
pub struct TorrentDhtPeersOperation {
    last_executed: Mutex<Option<Instant>>,
}

impl TorrentDhtPeersOperation {
    pub fn new() -> Self {
        Self {
            last_executed: Default::default(),
        }
    }

    async fn retrieve_peers(&self, context: &Arc<TorrentContext>) {
        if let Some(dht) = context.dht() {
            let info_hash = context.metadata_lock().read().await.info_hash.clone();
            match dht.get_peers(&info_hash, RETRIEVE_TIMEOUT).await {
                Ok(peers) => {
                    debug!("Torrent {} discovered {} DHT peers", context, peers.len());
                    context.add_peer_addresses(peers).await;
                }
                Err(err) => {
                    debug!("Torrent {} failed to retrieve peers, {}", context, err);
                }
            }
        } else {
            trace!(
                "Torrent {} is unable to retrieve DHT peers, no DHT tracker available",
                context
            );
        }

        *self.last_executed.lock().await = Some(Instant::now());
    }
}

#[async_trait]
impl TorrentOperation for TorrentDhtPeersOperation {
    fn name(&self) -> &str {
        "retrieve DHT peers operation"
    }

    async fn execute(&self, torrent: &Arc<TorrentContext>) -> TorrentOperationResult {
        let elapsed = if let Some(last_executed) = self.last_executed.lock().await.as_ref() {
            last_executed.elapsed()
        } else {
            Duration::from_secs(60 * 60)
        };

        if elapsed >= RETRIEVE_INTERVAL {
            self.retrieve_peers(torrent).await;
        }

        TorrentOperationResult::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torrent::dht::DhtTracker;
    use crate::torrent::storage::MemoryStorage;
    use crate::{create_torrent, init_logger};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_execute() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let torrent = create_torrent!(
            uri,
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![],
            vec![],
            |_| Box::new(MemoryStorage::new()),
            DhtTracker::builder()
                .default_routing_nodes()
                .build()
                .await
                .unwrap()
        );
        let context = torrent.instance().unwrap();
        let operation = TorrentDhtPeersOperation::new();

        // execute the operation
        let result = operation.execute(&context).await;
        assert_eq!(TorrentOperationResult::Continue, result);

        // check if the last_executed has been set
        let result = operation.last_executed.lock().await;
        assert_ne!(None, *result, "expected `last_executed` to have been set");
    }
}
