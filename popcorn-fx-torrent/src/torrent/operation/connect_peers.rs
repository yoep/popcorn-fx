use crate::torrent::peer::webseed::HttpPeer;
use crate::torrent::{
    TorrentCommandEvent, TorrentContext, TorrentOperation, TorrentOperationResult,
};
use async_trait::async_trait;
use futures::future;
use log::{debug, trace, warn};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};
use tokio::time;
use url::Url;

const BURST_DURATION: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct TorrentConnectPeersOperation {
    webseed_urls: Mutex<Option<Vec<Url>>>,
    /// The maximum amount of in-flight peer connections being established
    max_in_flight: Mutex<usize>,
    /// The semaphore to limit the number of in-flight peers for the operation
    permits: Arc<Semaphore>,
}

impl TorrentConnectPeersOperation {
    pub fn new() -> Self {
        Self {
            webseed_urls: Mutex::new(None),
            max_in_flight: Mutex::new(0),
            permits: Arc::new(Semaphore::new(0)),
        }
    }

    /// Create the webseed url cache from the torrent context.
    async fn create_webseed_urls(&self, torrent: &Arc<TorrentContext>) {
        // check if the webseed urls have already been created
        if self.webseed_urls.lock().await.is_some() {
            return;
        }

        let mut mutex = self.webseed_urls.lock().await;
        let metadata = torrent.metadata_lock().read().await;
        let urls = metadata
            .url_list
            .as_ref()
            .map(|list| {
                list.iter()
                    .flat_map(|url| Self::parse_url(torrent, url))
                    .collect()
            })
            .unwrap_or(Vec::new());

        *mutex = Some(urls);
    }

    /// Update the available in-flight permits from the latest torrent config.
    async fn update_in_flight_permits(&self, context: &Arc<TorrentContext>) {
        let mut max_in_flight = self.max_in_flight.lock().await;
        let config_peers_in_flight = context.config_lock().read().await.peers_in_flight;

        if config_peers_in_flight == *max_in_flight {
            return;
        }

        let change = config_peers_in_flight as i64 - *max_in_flight as i64;
        if change > 0 {
            self.permits.add_permits(change as usize);
        } else {
            self.permits.forget_permits((-change) as usize);
        }

        *max_in_flight = config_peers_in_flight;
    }

    /// Check if bursting the initial connections is allowed.
    async fn is_in_flight_burst_allowed(&self) -> bool {
        self.webseed_urls.lock().await.is_none()
    }

    /// Burst the initial connections.
    async fn burst(&self, context: &Arc<TorrentContext>) {
        let permits = self.permits.clone();
        let available_permits = permits.available_permits();
        let max_connections = context.config_lock().read().await.peers_upper_limit;
        let extra = max_connections - available_permits;

        trace!(
            "Torrent {} is bursting it's initial connections with {}",
            context,
            extra
        );
        permits.add_permits(extra);

        tokio::spawn(async move {
            time::sleep(BURST_DURATION).await;

            let mut to_forget = extra;

            // start decreasing the available permits back to it's original
            while to_forget > 0 {
                to_forget = to_forget - permits.forget_permits(to_forget);
                time::sleep(Duration::from_secs(1)).await;
            }
        });
    }

    /// Try to create additional peer connections
    async fn create_additional_peer_connections(
        &self,
        mut wanted_connections: usize,
        context: &Arc<TorrentContext>,
    ) {
        // try to create webseed peers
        if let Some(webseed_urls) = self.webseed_urls.lock().await.as_mut() {
            self.create_webseed_peers(webseed_urls, &mut wanted_connections, context)
                .await;
        }

        let available_permits = self.permits.available_permits();
        let len = wanted_connections.min(available_permits);
        let peer_pool = context.peer_pool();
        let peer_addrs = peer_pool.new_connection_candidates(len).await;

        match self.permits.clone().acquire_many_owned(len as u32).await {
            Ok(mut permits) => {
                debug!(
                    "Creating an additional {} (of wanted {}, remaining {} addresses) peer connections for {}",
                    peer_addrs.len(),
                    wanted_connections,
                    context.peer_pool().num_connect_candidates().await,
                    context
                );

                let peers: Vec<_> = peer_addrs
                    .into_iter()
                    .filter_map(|addr| {
                        if let Some(permit) = permits.split(1) {
                            Some((addr, permit))
                        } else {
                            None
                        }
                    })
                    .collect();

                for (addr, permit) in peers {
                    let runtime_context = context.clone();
                    tokio::spawn(async move {
                        Self::create_peer_with_dialers(runtime_context, addr, permit).await;
                    });
                }
            }
            Err(e) => warn!(
                "Torrent {} peer connections failed to acquire permits, {}",
                context, e
            ),
        }
    }

    async fn create_webseed_peers(
        &self,
        webseed_urls: &mut Vec<Url>,
        wanted_connections: &mut usize,
        context: &Arc<TorrentContext>,
    ) {
        let available_permits = self.permits.available_permits();
        let len = (*wanted_connections)
            .min(webseed_urls.len())
            .min(available_permits);

        match self.permits.clone().acquire_many_owned(len as u32).await {
            Ok(mut permits) => {
                let webseeds = webseed_urls
                    .drain(..len)
                    .filter_map(|url| {
                        if let Some(permit) = permits.split(1) {
                            Some((url, permit))
                        } else {
                            debug!("Torrent {} failed to acquire connection permit", context);
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                for (url, permit) in webseeds {
                    let runtime_context = context.clone();
                    tokio::spawn(async move {
                        Self::create_http_peer(runtime_context, url, permit).await;
                    });
                }

                *wanted_connections = *wanted_connections - len;
            }
            Err(e) => warn!(
                "Torrent {} peer connections failed to acquire permits, {}",
                context, e
            ),
        }
    }

    /// Try to establish the peer connection through the torrent peer dialers.
    /// This will dial the address for every dialer and create the connection of the first received successful peer connection.
    async fn create_peer_with_dialers(
        context: Arc<TorrentContext>,
        peer_addr: SocketAddr,
        permit: OwnedSemaphorePermit,
    ) {
        let protocol_extensions = context.protocol_extensions();
        let event_sender = context.event_sender();
        let peer_id = context.peer_id();
        let dialers = context.peer_dialers().clone();

        let handle_info = context.handle();
        let peer_connection_timeout = context.config_lock().read().await.peer_connection_timeout;

        debug!(
            "Torrent {} is trying to create new peer connection to {} through {} dialers",
            context,
            peer_addr,
            dialers.len()
        );
        let mut futures: Vec<_> = dialers
            .iter()
            .map(|dialer| {
                let extensions = context.extensions();

                dialer.dial(
                    peer_id,
                    peer_addr,
                    context.clone(),
                    protocol_extensions,
                    extensions,
                    peer_connection_timeout,
                )
            })
            .collect();
        if futures.is_empty() {
            warn!("Torrent {} has no active peer dialers", context);
            return;
        }

        loop {
            let (result, _, remaining) = future::select_all(futures).await;

            match result {
                Ok(peer) => {
                    drop(permit);
                    let _ = event_sender.send(TorrentCommandEvent::PeerConnected(peer));
                    break;
                }
                Err(e) => {
                    trace!(
                        "Torrent {} failed to connect with {}, {}",
                        handle_info,
                        peer_addr,
                        e
                    );
                }
            }

            if remaining.is_empty() {
                debug!(
                    "Torrent {} failed to connect to {}, none of the peer dialers succeeded",
                    handle_info, peer_addr
                );
                context
                    .peer_pool()
                    .peer_connections_closed(vec![peer_addr])
                    .await;
                break;
            }

            // replace the futures with the remaining uncompleted futures
            futures = remaining;
        }
    }

    /// Try to create a new HTTP (webseed) peer.
    async fn create_http_peer(
        context: Arc<TorrentContext>,
        url: Url,
        permit: OwnedSemaphorePermit,
    ) {
        let event_sender = context.event_sender();
        let handle_info = context.handle();

        debug!(
            "Torrent {} is trying to create webseed peer connection to {}",
            context, url
        );
        match HttpPeer::new(url, context.clone()) {
            Ok(peer) => {
                drop(permit);
                let _ = event_sender.send(TorrentCommandEvent::PeerConnected(Box::new(peer)));
            }
            Err(e) => {
                debug!(
                    "Failed to create http peer connection for torrent {}, {}",
                    handle_info, e
                );
                drop(permit);
            }
        }
    }

    fn parse_url(context: &Arc<TorrentContext>, url: &String) -> Option<Url> {
        Url::parse(url)
            .map_err(|e| {
                debug!("Torrent {} has invalid webseed url {}, {}", context, url, e);
                e
            })
            .ok()
    }
}

#[async_trait]
impl TorrentOperation for TorrentConnectPeersOperation {
    fn name(&self) -> &str {
        "create peer connections operation"
    }

    async fn execute(&self, torrent: &Arc<TorrentContext>) -> TorrentOperationResult {
        let wanted_connections = torrent.remaining_peer_connections_needed().await;
        if wanted_connections > 0 {
            let burst_connections = self.is_in_flight_burst_allowed().await;

            self.create_webseed_urls(torrent).await;
            self.update_in_flight_permits(torrent).await;

            // burst the initial connections if needed
            if burst_connections {
                self.burst(torrent).await;
            }

            self.create_additional_peer_connections(wanted_connections, torrent)
                .await;
        }

        TorrentOperationResult::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_torrent;
    use crate::init_logger;
    use crate::torrent::{TorrentConfig, TorrentFlags};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_create_webseed_urls() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let expected_result = vec![Url::parse("https://archive.org/download/").unwrap()];
        let uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce&ws=https%3A%2F%2Farchive.org%2Fdownload%2F";
        let torrent = create_torrent!(
            uri,
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![],
            vec![]
        );
        let context = torrent.instance().unwrap();
        let operation = TorrentConnectPeersOperation::new();

        operation.create_webseed_urls(&context).await;
        let result = operation.webseed_urls.lock().await;

        assert_eq!(Some(expected_result), *result);
    }

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
        let operation = TorrentConnectPeersOperation::new();

        let result = operation.execute(&context).await;

        assert_eq!(TorrentOperationResult::Continue, result);
    }

    #[tokio::test]
    async fn test_update_permits() {
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
        let operation = TorrentConnectPeersOperation::new();

        // update the permits from the torrent settings
        operation.update_in_flight_permits(&context).await;

        assert_ne!(
            0,
            *operation.max_in_flight.lock().await,
            "expected the max in flight to have been updated"
        );
    }
}
