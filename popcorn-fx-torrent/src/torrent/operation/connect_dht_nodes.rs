use crate::torrent::{TorrentContext, TorrentOperation, TorrentOperationResult};
use async_trait::async_trait;
use log::{debug, trace, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug)]
pub struct TorrentConnectDhtNodesOperation {
    inner: Arc<InnerConnectDhtNodesOperation>,
}

impl TorrentConnectDhtNodesOperation {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(InnerConnectDhtNodesOperation {
                initialized: Default::default(),
                running: Default::default(),
            }),
        }
    }
}

#[async_trait]
impl TorrentOperation for TorrentConnectDhtNodesOperation {
    fn name(&self) -> &str {
        "connect torrent DHT nodes operation"
    }

    async fn execute(&self, torrent: &Arc<TorrentContext>) -> TorrentOperationResult {
        if self.inner.should_connect_to_dht_nodes() {
            self.inner.running.store(true, Ordering::Relaxed);

            let context = torrent.clone();
            let inner = self.inner.clone();
            tokio::spawn(async move {
                inner.connect_dht_nodes(context).await;
            });
        }

        if self.inner.is_connecting() {
            TorrentOperationResult::Stop
        } else {
            TorrentOperationResult::Continue
        }
    }
}

#[derive(Debug)]
struct InnerConnectDhtNodesOperation {
    initialized: AtomicBool,
    running: AtomicBool,
}

impl InnerConnectDhtNodesOperation {
    /// Check if the operation is currently connecting to DHT nodes.
    fn is_connecting(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Check if the operation should connect to the DHT nodes of the torrent.
    fn should_connect_to_dht_nodes(&self) -> bool {
        !self.initialized.load(Ordering::Relaxed) && !self.running.load(Ordering::Relaxed)
    }

    async fn connect_dht_nodes(&self, context: Arc<TorrentContext>) {
        let metadata = context.metadata_lock().read().await;

        if let Some(dht) = context.dht() {
            if let Some(nodes) = metadata.nodes.as_ref() {
                let mut futures: Vec<_> = vec![];

                for node in nodes.iter() {
                    match node.socket_addr() {
                        Ok(addr) => {
                            futures.push(dht.ping(addr));
                        }
                        Err(e) => {
                            warn!("Torrent {} contains invalid DHT node, {}", context, e);
                        }
                    }
                }

                let pinged_nodes = futures::future::join_all(futures)
                    .await
                    .into_iter()
                    .filter(|e| e.is_ok())
                    .count();
                debug!(
                    "Torrent {} pinged a total of {} DHT nodes",
                    context, pinged_nodes
                );
            } else {
                debug!("Torrent {} does not have any DHT nodes", context);
            }
        } else {
            trace!(
                "Torrent {} is unable to connect to DHT network, no DHT tracker available",
                context
            );
        }

        self.initialized.store(true, Ordering::Relaxed);
        self.running.store(false, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::torrent::TorrentError;
    use crate::torrent::{TorrentConfig, TorrentFlags};
    use crate::{create_torrent, init_logger};

    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::select;
    use tokio::time;

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
            vec![]
        );
        let context = torrent.instance().unwrap();
        let operation = TorrentConnectDhtNodesOperation::new();

        let result = operation.execute(&context).await;
        assert_eq!(
            TorrentOperationResult::Stop,
            result,
            "expected DHT nodes to be initializing"
        );

        let result = select! {
            _ = time::sleep(Duration::from_secs(10)) => Err(TorrentError::Timeout),
            _ = async {
                while operation.inner.running.load(Ordering::Relaxed) {
                    time::sleep(Duration::from_millis(20)).await;
                }
            } => Ok(())
        };
        assert_eq!(Ok(()), result, "expected DHT nodes to be initialized");

        let result = operation.execute(&context).await;
        assert_eq!(
            TorrentOperationResult::Continue,
            result,
            "expected the DHT nodes to have been initialized"
        );
        assert_eq!(
            true,
            operation.inner.initialized.load(Ordering::Relaxed),
            "expected the operation to have been initialized"
        );
        assert_eq!(
            false,
            operation.inner.running.load(Ordering::Relaxed),
            "expected the operation to have been completed"
        );
    }
}
