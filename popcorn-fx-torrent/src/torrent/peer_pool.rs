use crate::torrent::peer::{Peer, PeerHandle, PeerState};
use crate::torrent::{TorrentHandle, TorrentPeer};
use derive_more::Display;
use itertools::Itertools;
use log::{debug, trace, warn};
use std::net::SocketAddr;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::{Mutex, RwLock};
use tokio::{select, time};

/// The failure reason when adding a peer failed.
#[derive(Debug, Error, PartialEq)]
pub enum Reason {
    #[error("peer already exists within the pool")]
    Duplicate,
    #[error("pool limit has been reached")]
    LimitReached,
}

/// The torrent peer pool manager for a single torrent.
/// This manager is responsible for managing the torrent peer information and actual active peers.
#[derive(Debug, Display)]
#[display(fmt = "{}", handle)]
pub struct PeerPool {
    /// The unique handle of the torrent
    handle: TorrentHandle,
    /// The currently active peers within the pool
    pub peers: RwLock<Vec<Box<dyn Peer>>>,
    /// The discovered torrent peers
    peer_list: Mutex<Vec<TorrentPeer>>,
    /// The maximum amount of peers allowed in the pool
    limit: Mutex<usize>,
}

impl PeerPool {
    /// Create a new peer pool for the given torrent handle.
    pub fn new(handle: TorrentHandle, pool_limit: usize) -> Self {
        Self {
            handle,
            peers: Default::default(),
            peer_list: Default::default(),
            limit: Mutex::new(pool_limit),
        }
    }

    /// Get the total amount of peer connections within the pool.
    pub async fn len(&self) -> usize {
        self.peers.read().await.len()
    }

    /// Get the limit of the peer pool.
    pub async fn pool_limit(&self) -> usize {
        *self.limit.lock().await
    }

    /// Add the given [TcpPeer] to this peer pool.
    /// The pool will check if the peer is unique before adding it to the pool, if it's a duplicate,
    /// the peer won't be added to the pool and the function will return [None].
    ///
    /// It returns a [Subscription] to receive peer events when the peer is added to the pool.
    pub async fn add_peer(&self, peer: Box<dyn Peer>) -> Result<(), Reason> {
        let mut peers = self.peers.write().await;
        let pool_limit = *self.limit.lock().await;

        // check if we've reached the pool limit
        if peers.len() >= pool_limit {
            debug!(
                "Peer pool {} is unable to add peer, pool limit reached",
                self
            );
            return Err(Reason::LimitReached);
        }

        // check if the peer already exists within the pool
        if peers.iter().any(|e| e.handle() == peer.handle()) {
            warn!("Peer pool {} detected duplicate peer {}", self, peer);
            return Err(Reason::Duplicate);
        }

        peers.push(peer);
        Ok(())
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

    /// Get the total amount of candidates for creating new connections.
    pub async fn num_connect_candidates(&self) -> usize {
        self.peer_list
            .lock()
            .await
            .iter()
            .filter(|peer| peer.is_connect_candidate())
            .count()
    }

    /// Add the given peer addresses to the pool's peer list.
    pub async fn add_peer_addresses(
        &self,
        addrs: Vec<SocketAddr>,
        torrent_addr: Option<SocketAddr>,
    ) {
        let peers = self.peers.read().await;
        let mut mutex = self.peer_list.lock().await;
        let addrs: Vec<_> = addrs
            .into_iter()
            // filter out duplicates
            .filter(|addr| !mutex.iter().any(|e| &e.addr == addr))
            // filter out any existing connections
            .filter(|addr| !peers.iter().any(|peer| peer.addr() == *addr))
            .map(|addr| {
                if let Some(torrent_addr) = torrent_addr {
                    TorrentPeer::new_with_rank(addr, &torrent_addr)
                } else {
                    TorrentPeer::new(addr)
                }
            })
            .collect();

        trace!(
            "Adding a total of {} new peer addrs for torrent {}",
            addrs.len(),
            self.handle
        );
        mutex.extend(addrs);
    }

    /// Updates the given peer addresses to be no longer in use.
    pub async fn peer_connections_closed(&self, addrs: Vec<SocketAddr>) {
        let mut mutex = self.peer_list.lock().await;

        for peer in mutex.iter_mut().filter(|e| addrs.contains(&e.addr)) {
            peer.is_in_use = false;
        }
    }

    /// Try to get the given amount of peer list addresses from the pool.
    /// If the peer list candidates are not enough, it will return the remaining available addresses.
    ///
    /// # Arguments
    ///
    /// * `len` - The total number of peer list address to retrieve.
    pub async fn new_connection_candidates(&self, len: usize) -> Vec<SocketAddr> {
        let mut peer_list = self.peer_list.lock().await;
        let pool_limit = *self.limit.lock().await;
        let remaining_slots = pool_limit - self.peers.read().await.len();
        let len = len.min(remaining_slots).min(peer_list.len());
        let mut result = Vec::new();

        for candidate in peer_list
            .iter_mut()
            .filter(|peer| peer.is_connect_candidate())
            .sorted()
            .take(len)
        {
            candidate.is_in_use = true;
            result.push(candidate.addr);
        }

        result
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
        *self.limit.lock().await = limit;
    }

    /// Remove any closed or invalid peers from the pool.
    /// The cleanup tries to close the peer connection within a timely manner if possible.
    pub async fn clean(&self) {
        let mut peers = self.peers.write().await;
        let mut handles_to_remove = vec![];

        for peer in peers.iter() {
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

        peers.retain(|e| !handles_to_remove.contains(&e.handle()));
        debug!("Cleaned a total of {} peers", handles_to_remove.len());
    }

    /// Shut down the peer pool, closing all peer connections.
    pub async fn shutdown(&self) {
        debug!("Shutting down peer pool for {}", self.handle);
        let mut peers = self.peers.write().await;

        // clear all known peer list addresses
        self.peer_list.lock().await.clear();

        self.set_pool_limit(0).await;
        for peer in std::mem::take(&mut *peers) {
            peer.close().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::init_logger;
    use crate::torrent::{TorrentConfig, TorrentFlags};
    use crate::{create_peer_pair, create_torrent};

    mod add_peer {
        use super::*;

        use crate::torrent::peer::tests::MockPeer;

        #[tokio::test]
        async fn test_add_peer() {
            init_logger!();
            let peer_handle = PeerHandle::new();
            let mut peer = MockPeer::new();
            peer.expect_handle().return_const(peer_handle);
            let pool = PeerPool::new(TorrentHandle::new(), 2);

            let result = pool.add_peer(Box::new(peer)).await;
            assert_eq!(Ok(()), result, "expected the peer to have been added");

            let result = pool.peers.read().await.len();
            assert_eq!(
                1, result,
                "expected the peer to have been present within the pool"
            );
        }

        #[tokio::test]
        async fn test_limit_reached() {
            init_logger!();
            let peer_handle1 = PeerHandle::new();
            let peer_handle2 = PeerHandle::new();
            let mut peer1 = MockPeer::new();
            peer1.expect_handle().return_const(peer_handle1);
            let mut peer2 = MockPeer::new();
            peer2.expect_handle().return_const(peer_handle2);
            let pool = PeerPool::new(TorrentHandle::new(), 1);

            let result = pool.add_peer(Box::new(peer1)).await;
            assert_eq!(Ok(()), result, "expected the peer to have been added");

            let result = pool.add_peer(Box::new(peer2)).await;
            assert_eq!(
                Err(Reason::LimitReached),
                result,
                "expected the peer to not have been added"
            );
        }
    }

    #[tokio::test]
    async fn test_peer_pool_add_available_peer_addrs() {
        init_logger!();
        let expected_result = vec![SocketAddr::from(([127, 0, 0, 1], 1900))];
        let pool = PeerPool::new(TorrentHandle::new(), 2);

        pool.add_peer_addresses(expected_result.clone(), None).await;
        let result = pool.num_connect_candidates().await;
        assert_eq!(1, result, "expected the address to have been added");

        let result = pool.new_connection_candidates(1).await;
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
        let pool = PeerPool::new(TorrentHandle::new(), 2);

        let _ = pool.add_peer(Box::new(peer1)).await;
        let _ = pool.add_peer(Box::new(peer2)).await;
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
