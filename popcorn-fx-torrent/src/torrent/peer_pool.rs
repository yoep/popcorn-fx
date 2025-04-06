use crate::torrent::peer::{Peer, PeerHandle, PeerState};
use crate::torrent::TorrentHandle;
use log::{debug, trace, warn};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, OwnedSemaphorePermit, RwLock, Semaphore};
use tokio::{select, time};

/// A pool manager which manages the peers of a torrent
#[derive(Debug)]
pub struct PeerPool {
    /// The unique handle of the torrent
    handle: TorrentHandle,
    /// The currently active peers within the pool
    pub peers: RwLock<Vec<Box<dyn Peer>>>,
    /// The discovered peers addrs
    available_peer_addrs: Mutex<Vec<SocketAddr>>,
    /// The peer addrs which have been identified as seeds
    seed_peer_addrs: Mutex<Vec<SocketAddr>>,
    /// The maximum amount of peers allowed in the pool
    limit: Mutex<usize>,
    /// The semaphore to limit the number of active peers and in-flight peers for the pool
    permits: Arc<Semaphore>,
}

impl PeerPool {
    /// Create a new peer pool for the given torrent handle.
    ///
    /// The pool limit defines the maximum amount of peer connections that can be active for the torrent.
    /// The maximum in flight sets the maximum amount of peer connections that can be tried to be established at the same moment.
    pub fn new(handle: TorrentHandle, pool_limit: usize, max_in_flight: usize) -> Self {
        let max_in_flight = max_in_flight.min(pool_limit);

        Self {
            handle,
            peers: Default::default(),
            available_peer_addrs: Default::default(),
            seed_peer_addrs: Default::default(),
            limit: Mutex::new(pool_limit),
            permits: Arc::new(Semaphore::new(pool_limit.min(max_in_flight))),
        }
    }

    /// Check if at least one permit for a new connection is available.
    ///
    pub fn is_permit_available(&self) -> bool {
        self.permits.available_permits() > 0
    }

    /// Add the given [TcpPeer] to this peer pool.
    /// The pool will check if the peer is unique before adding it to the pool, if it's a duplicate,
    /// the peer won't be added to the pool and the function will return [None].
    ///
    /// It returns a [Subscription] to receive peer events when the peer is added to the pool.
    pub async fn add_peer(&self, peer: Box<dyn Peer>) -> bool {
        let mut mutex = self.peers.write().await;

        if mutex.iter().any(|e| e.handle() == peer.handle()) {
            warn!(
                "Duplicate peer {} detected for torrent {}",
                peer, self.handle
            );
            return false;
        }

        mutex.push(peer);
        true
    }

    /// Remove the [TcpPeer] from the pool by the given [PeerHandle].
    /// It returns the peer that has been removed from the pool.
    pub async fn remove_peer(&self, peer: PeerHandle) -> Option<Box<dyn Peer>> {
        let mut mutex = self.peers.write().await;
        mutex
            .iter()
            .position(|e| e.handle() == peer)
            .map(|position| mutex.remove(position))
    }

    /// Get the length of the currently available/known peer addresses.
    pub async fn available_peer_addrs_len(&self) -> usize {
        self.available_peer_addrs.lock().await.len()
    }

    /// Add the given peer addrs to the pool's available peer addrs.
    pub async fn add_available_peer_addrs(&self, addrs: Vec<SocketAddr>) {
        let peers = self.peers.read().await;
        let mut mutex = self.available_peer_addrs.lock().await;
        let addrs: Vec<_> = addrs
            .into_iter()
            // filter out duplicates
            .filter(|addr| !mutex.contains(addr))
            // filter out any existing connections
            .filter(|addr| !peers.iter().any(|peer| peer.addr() == *addr))
            .collect();

        trace!(
            "Adding a total of {} new peer addrs for torrent {}",
            addrs.len(),
            self.handle
        );
        mutex.extend(addrs);
    }

    /// Remove the given peer addrs from the pool's available peer addrs.
    pub async fn remove_available_peer_addrs(&self, addrs: Vec<SocketAddr>) {
        let mut mutex = self.available_peer_addrs.lock().await;
        mutex.retain(|e| !addrs.contains(e));
    }

    /// Add the given peer addrs to the pool's seed peer addrs.
    pub async fn add_seed_peer_addrs(&self, addrs: Vec<SocketAddr>) {
        let mut mutex = self.seed_peer_addrs.lock().await;
        let addrs: Vec<SocketAddr> = addrs.into_iter().filter(|e| !mutex.contains(e)).collect();
        mutex.extend(addrs);
    }

    pub async fn remove_seed_peer_addrs(&self, addrs: Vec<SocketAddr>) {
        let mut mutex = self.seed_peer_addrs.lock().await;
        mutex.retain(|e| !addrs.contains(e));
    }

    /// Try to get the given amount of available peer addrs from the pool.
    /// If the available peer addrs are not enough, it will return the remaining available addresses.
    pub async fn take_available_peer_addrs(&self, len: usize) -> Vec<SocketAddr> {
        let mut mutex = self.available_peer_addrs.lock().await;
        let remaining_permits = self.permits.available_permits();
        let len = len.min(remaining_permits).min(mutex.len());

        mutex.drain(0..len).collect()
    }

    /// Get the total amount of healthy peer connections from the pool.
    pub async fn active_peer_connections(&self) -> usize {
        let mut count = 0;

        for peer in self.peers.read().await.iter() {
            let state = peer.state().await;
            if state != PeerState::Closed && state != PeerState::Error {
                count += 1;
            }
        }

        count
    }

    /// Set a new maximum amount of peers allowed in the pool.
    pub async fn set_pool_limit(&self, limit: usize) {
        let mut mutex = self.limit.lock().await;
        let change = limit as i32 - *mutex as i32;

        if change > 0 {
            self.permits.add_permits(change as usize);
        } else {
            self.permits.forget_permits(change as usize);
        }

        *mutex = limit;
    }

    /// Try to get a permit for creating a new peer connection.
    /// This permit limits the amount of active peers in the pool and from overcommitment.
    pub async fn permit(&self) -> Option<OwnedSemaphorePermit> {
        if self.permits.available_permits() == 0 {
            return None;
        }

        self.permits.clone().acquire_owned().await.ok()
    }

    /// Remove any closed or invalid peers from the pool.
    /// The cleanup tries to close the peer connection within a timely manner if possible.
    pub async fn clean(&self) {
        let mut mutex = self.peers.write().await;
        let mut handles_to_remove = vec![];

        for peer in mutex.iter() {
            let state = peer.state().await;
            if state == PeerState::Closed || state == PeerState::Error {
                handles_to_remove.push(peer.handle());
                select! {
                    _ = time::sleep(Duration::from_secs(1)) => {
                        debug!("Failed to close peer {} connection, close timed out", peer);
                    },
                    _ = peer.close() => {},
                }
            }
        }

        mutex.retain(|e| !handles_to_remove.contains(&e.handle()));
        debug!("Cleaned a total of {} peers", handles_to_remove.len());
    }

    /// Shut down the peer pool, closing all peer connections.
    pub async fn shutdown(&self) {
        debug!("Shutting down peer pool for {}", self.handle);
        let mut peers = self.peers.write().await;

        // clear all known addrs
        self.available_peer_addrs.lock().await.clear();
        self.seed_peer_addrs.lock().await.clear();

        self.set_pool_limit(0).await;
        for peer in std::mem::take(&mut *peers) {
            peer.close().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torrent::{TorrentConfig, TorrentFlags};
    use crate::{create_peer_pair, create_torrent};
    use popcorn_fx_core::init_logger;

    #[test]
    fn test_peer_pool_new_max_inflight_larger_than_pool_limit() {
        init_logger!();
        let pool_limit = 2;
        let pool = PeerPool::new(TorrentHandle::new(), pool_limit, 10);

        let result = pool.permits.available_permits();

        assert_eq!(
            pool_limit, result,
            "expected the max in flight to not be larger than the pool limit"
        );
    }

    #[tokio::test]
    async fn test_peer_pool_is_permit_available() {
        init_logger!();
        let pool = PeerPool::new(TorrentHandle::new(), 2, 1);

        let permit = pool.permit().await;
        assert!(permit.is_some(), "expected a permit");

        let result = pool.is_permit_available();
        assert_eq!(
            false, result,
            "expected no additional permits to have been available"
        );

        drop(permit);
        let result = pool.is_permit_available();
        assert_eq!(true, result, "expected a permit to have been available");
    }

    #[tokio::test]
    async fn test_peer_pool_add_available_peer_addrs() {
        init_logger!();
        let expected_result = vec![SocketAddr::from(([127, 0, 0, 1], 1900))];
        let pool = PeerPool::new(TorrentHandle::new(), 2, 1);

        pool.add_available_peer_addrs(expected_result.clone()).await;
        let result = pool.available_peer_addrs_len().await;
        assert_eq!(1, result, "expected the address to have been added");

        let result = pool.take_available_peer_addrs(1).await;
        assert_eq!(expected_result, result);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_peer_pool_clean() {
        init_logger!();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![]
        );
        let (peer1, peer2) = create_peer_pair!(&torrent);
        let pool = PeerPool::new(TorrentHandle::new(), 2, 1);

        pool.add_peer(Box::new(peer1)).await;
        pool.add_peer(Box::new(peer2)).await;
        let result = pool.peers.read().await.len();
        assert_eq!(
            2, result,
            "expected the peers to have been added to the pool"
        );

        pool.peers.read().await.get(0).unwrap().close().await;
        pool.clean().await;

        let result = pool.peers.read().await.len();
        assert_ne!(2, result);
    }
}
