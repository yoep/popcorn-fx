use crate::torrent::peer::{Peer, PeerClientInfo, PeerHandle, PeerState};
use crate::torrent::{PeerPriority, TorrentHandle, TorrentPeer};
use derive_more::Display;
use itertools::Itertools;
use log::{debug, trace, warn};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::{Mutex, RwLock, RwLockWriteGuard};
use tokio::time;

const CONNECTION_FAILURE_THRESHOLD: usize = 3;

/// The failure reason when adding a peer failed.
#[derive(Debug, Error, PartialEq)]
pub enum AddReason {
    #[error("peer already exists within the pool")]
    Duplicate,
    #[error("pool limit has been reached")]
    LimitReached,
}

/// The reason why a peer has been closed.
#[derive(Debug, PartialEq)]
pub enum CloseReason {
    /// Establishing a connection to the remote peer failed.
    ConnectionFailed,
    /// The client has closed the connection with the remote peer.
    Client,
    /// The remote peer has closed the connection.
    Remote,
}

/// The torrent peer pool manager for a single torrent.
/// This manager is responsible for managing the torrent peer information and actual active peers.
#[derive(Debug, Display)]
#[display(fmt = "{}", handle)]
pub struct PeerPool {
    /// The unique handle of the torrent
    handle: TorrentHandle,
    /// The currently active peers within the pool
    pub peers: RwLock<HashMap<PeerHandle, Arc<dyn Peer>>>,
    /// The discovered torrent peers
    peer_list: Mutex<HashMap<SocketAddr, PeerInfo>>,
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

    /// Get a torrent peer for the given handle.
    ///
    /// It returns the torrent peer instance when found, else [None].
    pub async fn get(&self, handle: &PeerHandle) -> Option<TorrentPeer> {
        let peers = self.peers.read().await;

        peers.get(&handle).map(|peer| TorrentPeer::new(peer))
    }

    /// Add the given [TcpPeer] to this peer pool.
    /// The pool will check if the peer is unique before adding it to the pool, if it's a duplicate,
    /// the peer won't be added to the pool and the function will return [None].
    ///
    /// It returns a [Subscription] to receive peer events when the peer is added to the pool.
    pub async fn add_peer(&self, peer: Box<dyn Peer>) -> Result<(), AddReason> {
        let mut peers = self.peers.write().await;
        let pool_limit = *self.limit.lock().await;
        let handle = peer.handle();

        // check if we've reached the pool limit
        if peers.len() >= pool_limit {
            debug!(
                "Peer pool {} is unable to add peer {}, pool limit reached",
                self, handle
            );
            return Err(AddReason::LimitReached);
        }

        // check if the peer already exists within the pool
        if peers.contains_key(&handle) {
            warn!("Peer pool {} detected duplicate peer {}", self, peer);
            return Err(AddReason::Duplicate);
        }

        peers.insert(handle, Arc::from(peer));
        Ok(())
    }

    /// Remove a torrent [Peer] from the pool by the given [PeerHandle].
    ///
    /// It returns the peer info when the peer was found and removed, else [None].
    pub async fn remove_peer(&self, handle: &PeerHandle) -> Option<PeerClientInfo> {
        let mut peers = self.peers.write().await;
        self.internal_remove_peer(&mut peers, handle)
            .await
            .map(|e| e.client())
    }

    /// Get the total amount of candidates for creating new connections.
    pub async fn num_connect_candidates(&self) -> usize {
        self.peer_list
            .lock()
            .await
            .iter()
            .filter(|(_, peer)| peer.is_connect_candidate())
            .count()
    }

    /// Add the given peer addresses to the pool's peer list.
    pub async fn add_peer_addresses(
        &self,
        addrs: Vec<SocketAddr>,
        torrent_addr: Option<SocketAddr>,
    ) {
        let peers = self.peers.read().await;
        let mut peer_list = self.peer_list.lock().await;
        let addrs: Vec<_> = addrs
            .into_iter()
            // filter out duplicates
            .filter(|addr| !peer_list.contains_key(addr))
            // filter out any existing connections
            .filter(|addr| !peers.iter().any(|(_, peer)| peer.addr() == *addr))
            .map(|addr| {
                if let Some(torrent_addr) = torrent_addr {
                    (addr, PeerInfo::new_with_rank(addr, &torrent_addr))
                } else {
                    (addr, PeerInfo::new(addr))
                }
            })
            .collect();

        trace!(
            "Adding a total of {} new peer addrs for torrent {}",
            addrs.len(),
            self.handle
        );
        peer_list.extend(addrs);
    }

    /// Updates the given peer address to be no longer in use.
    pub async fn peer_connection_closed(&self, addr: &SocketAddr, reason: CloseReason) {
        let mut peer_list = self.peer_list.lock().await;

        if let Some(peer) = peer_list.get_mut(addr) {
            match reason {
                CloseReason::ConnectionFailed => {
                    peer.failure_count = peer.failure_count.saturating_add(1);
                }
                _ => {}
            }

            peer.is_in_use = false;
        }
    }

    /// Update the peer priority of the given address.
    pub async fn update_peer_rank(&self, addr: &SocketAddr, change: i32) {
        let mut peer_list = self.peer_list.lock().await;

        if let Some(peer) = peer_list.get_mut(addr) {
            let mut rank = peer.rank.take().unwrap_or(0);

            if change < 0 {
                rank = rank.saturating_sub(1);
            } else {
                rank = rank.saturating_add(1);
            }

            peer.rank = PeerPriority::from(rank);
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

        peer_list
            .iter_mut()
            .filter(|(_, peer)| peer.is_connect_candidate())
            .sorted()
            .take(len)
            .map(|(addr, peer)| {
                peer.is_in_use = true;
                addr.clone()
            })
            .collect()
    }

    /// Get the total amount of healthy peer connections from the pool.
    pub async fn active_peer_connections(&self) -> usize {
        let mut count = 0;

        for (_, peer) in self.peers.read().await.iter() {
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
        let mut total_cleaned_peers = 0;

        let futures: Vec<_> = peers
            .iter()
            .map(|(handle, peer)| async move { (*handle, peer.state().await) })
            .collect();

        for (handle, state) in futures::future::join_all(futures).await {
            if state == PeerState::Closed || state == PeerState::Error {
                if let Some(peer) = self.internal_remove_peer(&mut peers, &handle).await {
                    total_cleaned_peers += 1;

                    if let Err(_) = time::timeout(Duration::from_secs(1), peer.close()).await {
                        debug!(
                            "Torrent peer pool {} failed to close peer {} connection, close operation timed out",
                            self, peer
                        );
                    }
                }
            }
        }

        debug!("Cleaned a total of {} peers", total_cleaned_peers);
    }

    /// Shut down the peer pool, closing all peer connections.
    pub async fn shutdown(&self) {
        debug!("Shutting down peer pool for {}", self.handle);

        // clear all known peer list addresses
        self.peer_list.lock().await.clear();
        self.set_pool_limit(0).await;

        let peers_to_close: Vec<Arc<dyn Peer>> = {
            let mut lock = self.peers.write().await;
            lock.drain().map(|(_, e)| e).collect()
        };

        for peer in peers_to_close {
            peer.close().await;
        }
    }

    async fn internal_remove_peer(
        &self,
        peers: &mut RwLockWriteGuard<'_, HashMap<PeerHandle, Arc<dyn Peer>>>,
        handle: &PeerHandle,
    ) -> Option<Arc<dyn Peer>> {
        if let Some(peer) = peers.remove(handle) {
            let addr = peer.addr();

            if let Some(info) = self.peer_list.lock().await.get_mut(&addr) {
                info.is_seed = peer.is_seed().await;
                info.is_in_use = false;
            }

            Some(peer)
        } else {
            None
        }
    }
}

/// The address information of a peer for the torrent.
#[derive(Debug, Clone)]
struct PeerInfo {
    /// The address of a remote peer.
    addr: SocketAddr,
    /// Indicates if this peer address is in use by the torrent.
    is_in_use: bool,
    /// Indicates if this peer has been identified as a seed.
    is_seed: bool,
    /// Indicates if this peer has been banned from establishing a connection.
    is_banned: bool,
    /// The number of failures when trying to connect to the remote peer.
    failure_count: usize,
    /// The peer priority rank.
    rank: PeerPriority,
}

impl PeerInfo {
    /// Create a new torrent peer address information.
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            is_in_use: false,
            is_seed: false,
            is_banned: false,
            failure_count: 0,
            rank: PeerPriority::none(),
        }
    }

    /// Create a new torrent peer address information.
    /// This peer address contains a rank based against the current torrent listening address.
    pub fn new_with_rank(addr: SocketAddr, torrent_addr: &SocketAddr) -> Self {
        let rank = PeerPriority::from((torrent_addr, &addr));
        Self {
            addr,
            is_in_use: false,
            is_seed: false,
            is_banned: false,
            failure_count: 0,
            rank,
        }
    }

    /// Check if this peer is a candidate for establishing a new connection.
    ///
    /// # Returns
    ///
    /// It returns true when the peer is a candidate, else false.
    pub fn is_connect_candidate(&self) -> bool {
        !self.is_in_use && !self.is_banned && self.failure_count < CONNECTION_FAILURE_THRESHOLD
    }
}

impl PartialEq for PeerInfo {
    fn eq(&self, other: &Self) -> bool {
        self.addr == other.addr
    }
}

impl PartialOrd for PeerInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // always prefer known seeds over rank
        if self.is_seed && !other.is_seed {
            return Some(Ordering::Less);
        } else if other.is_seed && !self.is_seed {
            return Some(Ordering::Greater);
        }

        // always prefer lesser failed addresses above rank
        if self.failure_count != other.failure_count {
            return self.failure_count.partial_cmp(&other.failure_count);
        }

        self.rank.partial_cmp(&other.rank)
    }
}

impl Eq for PeerInfo {}

impl Ord for PeerInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::init_logger;

    mod peer_pool {
        use super::*;
        use crate::torrent::peer::tests::MockPeer;

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
                    Err(AddReason::LimitReached),
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
            let peer1_handle = PeerHandle::new();
            let mut peer1 = MockPeer::new();
            peer1.expect_handle().return_const(peer1_handle);
            peer1
                .expect_addr()
                .return_const(SocketAddr::from(([127, 0, 0, 1], 6881)));
            peer1.expect_state().returning(|| PeerState::Closed);
            peer1.expect_close().return_const(());
            let mut peer2 = MockPeer::new();
            peer2.expect_handle().return_const(PeerHandle::new());
            peer1
                .expect_addr()
                .return_const(SocketAddr::from(([127, 0, 0, 2], 6881)));
            peer2.expect_state().returning(|| PeerState::Idle);
            let pool = PeerPool::new(TorrentHandle::new(), 2);

            // add peers to the pool
            let _ = pool.add_peer(Box::new(peer1)).await;
            let _ = pool.add_peer(Box::new(peer2)).await;
            let result = pool.peers.read().await.len();
            assert_eq!(
                2, result,
                "expected the peers to have been added to the pool"
            );

            // clean the peer pool
            pool.clean().await;

            let result = pool.peers.read().await.len();
            assert_eq!(1, result, "expected the closed peer to have been removed");
        }
    }

    mod peer_info {
        use super::*;

        #[test]
        fn test_is_connect_candidate() {
            let peer = PeerInfo {
                addr: ([127, 0, 0, 1], 8090).into(),
                is_in_use: false,
                is_seed: false,
                is_banned: false,
                failure_count: 0,
                rank: PeerPriority::none(),
            };
            assert_eq!(
                true,
                peer.is_connect_candidate(),
                "expected the peer to be a candidate"
            );

            let peer = PeerInfo {
                addr: ([127, 0, 0, 1], 8090).into(),
                is_in_use: true,
                is_seed: false,
                is_banned: false,
                failure_count: 0,
                rank: PeerPriority::none(),
            };
            assert_eq!(
                false,
                peer.is_connect_candidate(),
                "expected a in-use peer to not have been a candidate"
            );

            let peer = PeerInfo {
                addr: ([127, 0, 0, 1], 8090).into(),
                is_in_use: false,
                is_seed: false,
                is_banned: true,
                failure_count: 0,
                rank: PeerPriority::none(),
            };
            assert_eq!(
                false,
                peer.is_connect_candidate(),
                "expected a banned peer to not have been a candidate"
            );
        }

        #[tokio::test]
        async fn test_update_peer_priority() {
            init_logger!();
            let peer_address = SocketAddr::from(([127, 0, 0, 3], 6881));
            let pool = PeerPool::new(TorrentHandle::new(), 2);

            // add the peer address to the pool
            pool.add_peer_addresses(vec![peer_address.clone()], None)
                .await;

            // decrease the peer address priority
            pool.update_peer_rank(&peer_address, -1).await;
            let mut result = pool
                .peer_list
                .lock()
                .await
                .get(&peer_address)
                .cloned()
                .unwrap();
            assert_eq!(Some(0), result.rank.take());

            // increase the peer address priority
            pool.update_peer_rank(&peer_address, 1).await;
            let mut result = pool
                .peer_list
                .lock()
                .await
                .get(&peer_address)
                .cloned()
                .unwrap();
            assert_eq!(Some(1), result.rank.take());
        }

        mod order {
            use super::*;

            #[test]
            fn test_rank() {
                let peer1 = PeerInfo {
                    addr: ([127, 0, 0, 1], 8090).into(),
                    is_in_use: false,
                    is_seed: false,
                    is_banned: false,
                    failure_count: 0,
                    rank: PeerPriority::from(30),
                };
                let peer2 = PeerInfo {
                    addr: ([127, 0, 0, 1], 8090).into(),
                    is_in_use: false,
                    is_seed: false,
                    is_banned: false,
                    failure_count: 0,
                    rank: PeerPriority::from(10),
                };

                assert_eq!(Ordering::Less, peer1.cmp(&peer2));
                assert_eq!(Ordering::Greater, peer2.cmp(&peer1));
            }

            #[test]
            fn test_seed() {
                let peer1 = PeerInfo {
                    addr: ([127, 0, 0, 1], 8090).into(),
                    is_in_use: false,
                    is_seed: false,
                    is_banned: false,
                    failure_count: 0,
                    rank: PeerPriority::none(),
                };
                let peer2 = PeerInfo {
                    addr: ([127, 0, 0, 1], 8090).into(),
                    is_in_use: false,
                    is_seed: true,
                    is_banned: false,
                    failure_count: 0,
                    rank: PeerPriority::none(),
                };

                assert_eq!(Ordering::Greater, peer1.cmp(&peer2));
                assert_eq!(Ordering::Less, peer2.cmp(&peer1));
            }

            #[test]
            fn test_failure_count() {
                let peer1 = PeerInfo {
                    addr: ([127, 0, 0, 1], 8090).into(),
                    is_in_use: false,
                    is_seed: false,
                    is_banned: false,
                    failure_count: 2,
                    rank: PeerPriority::none(),
                };
                let peer2 = PeerInfo {
                    addr: ([127, 0, 0, 1], 8090).into(),
                    is_in_use: false,
                    is_seed: false,
                    is_banned: false,
                    failure_count: 0,
                    rank: PeerPriority::none(),
                };

                assert_eq!(Ordering::Greater, peer1.cmp(&peer2));
                assert_eq!(Ordering::Less, peer2.cmp(&peer1));
            }
        }
    }
}
