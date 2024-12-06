use crate::torrent::peer::{Peer, PeerHandle, PeerState};
use crate::torrent::TorrentHandle;
use log::{debug, warn};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock, Semaphore};
use tokio::{select, time};

/// A pool manager which manages the peers of a torrent
#[derive(Debug)]
pub struct PeerPool {
    /// The unique handle of the torrent
    handle: TorrentHandle,
    /// The currently active peers within the pool
    pub peers: RwLock<Vec<Peer>>,
    /// The discovered peers addrs
    available_peer_addrs: RwLock<Vec<SocketAddr>>,
    /// The maximum amount of peers allowed in the pool
    limit: Mutex<usize>,
    /// The semaphore to limit the number of active peers and in-flight peers for the pool
    semaphore: Semaphore,
}

impl PeerPool {
    /// Create a new peer pool for the given torrent handle.
    pub fn new(handle: TorrentHandle, pool_limit: usize) -> Self {
        Self {
            handle,
            peers: Default::default(),
            available_peer_addrs: Default::default(),
            limit: Mutex::new(pool_limit),
            semaphore: Semaphore::new(pool_limit),
        }
    }

    /// Add the given peer to the pool.
    pub async fn add_peer(&self, peer: Peer) -> bool {
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

    /// Remove the peer from the pool.
    pub async fn remove_peer(&self, peer: PeerHandle) {
        let mut mutex = self.peers.write().await;
        mutex.retain(|e| e.handle() != peer);
    }

    /// Get the length of the currently available/known peer addresses.
    pub async fn available_peer_addrs_len(&self) -> usize {
        self.available_peer_addrs.read().await.len()
    }

    /// Add the given peer addrs to the pool's available peer addrs.
    pub async fn add_available_peer_addrs(&self, addrs: Vec<SocketAddr>) {
        let mut mutex = self.available_peer_addrs.write().await;
        let addrs: Vec<_> = addrs.into_iter().filter(|e| !mutex.contains(e)).collect();

        debug!(
            "Adding a total of {} new peer addrs for torrent {}",
            addrs.len(),
            self.handle
        );
        mutex.extend(addrs);
    }

    /// Try to get the given amount of available peer addrs from the pool.
    /// If the available peer addrs are not enough, it will return the remaining available addresses.
    pub async fn take_available_peer_addrs(&self, mut len: usize) -> Vec<SocketAddr> {
        let mut mutex = self.available_peer_addrs.write().await;

        if mutex.len() < len {
            len = mutex.len();
        }

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
            self.semaphore.add_permits(change as usize);
        } else {
            self.semaphore.forget_permits(change as usize);
        }

        *mutex = limit;
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
        let mut addrs = self.available_peer_addrs.write().await;

        addrs.clear();
        self.set_pool_limit(0).await;
        for peer in std::mem::take(&mut *peers) {
            peer.close().await;
        }
    }
}
