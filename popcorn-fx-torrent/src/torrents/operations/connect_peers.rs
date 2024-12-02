use crate::torrents::{InnerTorrent, TorrentCommandEvent, TorrentOperation};
use async_trait::async_trait;
use derive_more::Display;

#[derive(Debug, Display)]
#[display(fmt = "connect peers operation")]
pub struct TorrentPeersOperation {}

impl TorrentPeersOperation {
    pub fn new() -> Self {
        Self {}
    }

    async fn create_additional_peer_connections(
        &self,
        wanted_connections: usize,
        torrent: &InnerTorrent,
    ) {
        let peer_addrs = torrent
            .peer_pool()
            .take_available_peer_addrs(wanted_connections)
            .await;

        for addr in peer_addrs {
            torrent.send_command_event(TorrentCommandEvent::ConnectToPeer(addr));
        }
    }
}

#[async_trait]
impl TorrentOperation for TorrentPeersOperation {
    async fn execute<'a>(&self, torrent: &'a InnerTorrent) -> Option<&'a InnerTorrent> {
        let wanted_connections = torrent.remaining_peer_connections_needed().await;
        if wanted_connections > 0 {
            self.create_additional_peer_connections(wanted_connections, torrent)
                .await;
        }

        Some(torrent)
    }

    fn clone_boxed(&self) -> Box<dyn TorrentOperation> {
        Box::new(TorrentPeersOperation::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torrents::fs::DefaultTorrentFileStorage;
    use crate::torrents::{Torrent, TorrentConfig, TorrentFlags, TorrentInfo};
    use popcorn_fx_core::core::torrents::magnet::Magnet;
    use popcorn_fx_core::testing::init_logger;
    use std::str::FromStr;
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::runtime::Runtime;

    #[tokio::test]
    async fn test_execute() {
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
        let operation = TorrentPeersOperation::new();

        let result = operation.execute(&*inner).await;

        assert_eq!(Some(&*inner), result);
    }
}
