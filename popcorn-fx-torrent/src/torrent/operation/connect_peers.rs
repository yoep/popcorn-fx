use crate::torrent::errors::Result;
use crate::torrent::peer::extension::Extensions;
use crate::torrent::peer::webseed::HttpPeer;
use crate::torrent::peer::{PeerId, ProtocolExtensionFlags, TcpPeer};
use crate::torrent::{
    TorrentCommandEvent, TorrentContext, TorrentOperation, TorrentOperationResult,
};
use async_trait::async_trait;
use futures::future;
use log::{debug, trace, warn};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use url::Url;

#[derive(Debug)]
pub struct TorrentConnectPeersOperation {
    webseed_urls: Mutex<Option<Vec<Url>>>,
}

impl TorrentConnectPeersOperation {
    pub fn new() -> Self {
        Self {
            webseed_urls: Mutex::new(None),
        }
    }

    /// Create the webseed url cache from the torrent context.
    /// This should be done only once.
    async fn create_webseed_urls(&self, torrent: &Arc<TorrentContext>) {
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

    async fn create_additional_peer_connections(
        &self,
        wanted_connections: usize,
        context: &Arc<TorrentContext>,
    ) {
        // try to add webseed peers
        if let Some(webseed_urls) = self.webseed_urls.lock().await.as_mut() {
            let end = wanted_connections.min(webseed_urls.len());

            for url in webseed_urls.drain(..end) {
                self.create_http_peer(context, url).await;
            }
        }

        let peer_pool = context.peer_pool();
        let peer_addrs = peer_pool
            .take_available_peer_addrs(wanted_connections)
            .await;

        debug!(
            "Creating an additional {} (of wanted {}, remaining {} addresses) peer connections for {}",
            peer_addrs.len(),
            wanted_connections,
            context.peer_pool().available_peer_addrs_len().await,
            context
        );

        // try to add known peers
        let futures: Vec<_> = peer_addrs
            .into_iter()
            .map(|addr| self.create_peer_with_dialers(context, addr))
            .collect();

        future::join_all(futures).await;
    }

    async fn create_http_peer(&self, context: &Arc<TorrentContext>, url: Url) {
        if let Some(permit) = context.peer_pool().permit().await {
            let event_sender = context.event_sender();
            let runtime = context.runtime().clone();

            let runtime_context = context.clone();
            context.runtime().spawn(async move {
                let handle_info = runtime_context.handle();

                debug!(
                    "Torrent {} is trying to create webseed peer connection to {}",
                    runtime_context, url
                );
                match HttpPeer::new(url, runtime_context, runtime) {
                    Ok(peer) => {
                        drop(permit);
                        let _ =
                            event_sender.send(TorrentCommandEvent::PeerConnected(Box::new(peer)));
                    }
                    Err(e) => {
                        debug!(
                            "Failed to create http peer connection for torrent {}, {}",
                            handle_info, e
                        );
                        drop(permit);
                    }
                }
            });
        }
    }

    /// Try to establish the peer connection through the torrent peer dialers.
    /// This will dial the address for every dialer and create the connection of the first received successful peer connection.
    async fn create_peer_with_dialers(&self, context: &Arc<TorrentContext>, peer_addr: SocketAddr) {
        if let Some(permit) = context.peer_pool().permit().await {
            let protocol_extensions = context.protocol_extensions();
            let event_sender = context.event_sender();
            let peer_id = context.peer_id();
            let dialers = context.peer_dialers().clone();

            let runtime_context = context.clone();
            context.runtime().spawn(async move {
                let handle_info = runtime_context.handle();
                let peer_connection_timeout = runtime_context.config_lock().read().await.peer_connection_timeout;

                debug!(
                    "Torrent {} is trying to create new peer connection to {} through {} dialers",
                    runtime_context, peer_addr, dialers.len()
                );
                let mut futures : Vec<_> = dialers.iter()
                    .map(|dialer| {
                        let extensions = runtime_context.extensions();
                        
                        dialer.dial(
                            peer_id,
                            peer_addr,
                            runtime_context.clone(),
                            protocol_extensions,
                            extensions,
                            peer_connection_timeout
                        )
                    })
                    .collect();
                if futures.is_empty() {
                    warn!("Torrent {} has no active peer dialers", runtime_context);
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
                                handle_info, peer_addr, e
                            );
                        }
                    }

                    if remaining.is_empty() {
                        debug!("Torrent {} failed to connect to {}, none of the peer dialers succeeded", handle_info, peer_addr);
                        break;
                    }

                    // replace the futures with the remaining uncompleted futures
                    futures = remaining;
                }
            });
        } else {
            // put the address back into the peer pool as no permit was granted from making the connection
            context
                .peer_pool()
                .add_available_peer_addrs(vec![peer_addr])
                .await;
        }
    }

    fn parse_url(torrent: &Arc<TorrentContext>, url: &String) -> Option<Url> {
        Url::parse(url)
            .map_err(|e| {
                debug!("Torrent {} has invalid webseed url {}, {}", torrent, url, e);
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
            // check if the webseed urls need to be created
            let create_webseeds = self.webseed_urls.lock().await.is_none();
            if create_webseeds {
                self.create_webseed_urls(torrent).await;
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
    use crate::torrent::{TorrentConfig, TorrentFlags};
    use popcorn_fx_core::init_logger;
    use tempfile::tempdir;

    #[test]
    fn test_create_webseed_urls() {
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
            vec![]
        );
        let context = torrent.instance().unwrap();
        let runtime = context.runtime();
        let operation = TorrentConnectPeersOperation::new();

        runtime.block_on(operation.create_webseed_urls(&context));
        let result = runtime.block_on(operation.webseed_urls.lock());

        assert_eq!(Some(expected_result), *result);
    }

    #[test]
    fn test_execute() {
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
        let runtime = context.runtime();
        let operation = TorrentConnectPeersOperation::new();

        let result = runtime.block_on(operation.execute(&context));

        assert_eq!(TorrentOperationResult::Continue, result);
    }
}
