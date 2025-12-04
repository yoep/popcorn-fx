use crate::torrent::InfoHash;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::net::SocketAddr;
use std::time::Instant;

/// Stores peer information received by the DHT network.
#[derive(Debug)]
pub struct PeerStorage {
    peers: HashMap<InfoHash, HashSet<PeerEntry>>,
}

impl PeerStorage {
    pub fn new() -> Self {
        Self {
            peers: Default::default(),
        }
    }

    /// Returns a peers iterator for the given info hash.
    /// The iterator might be empty if no info has yet been received for the [InfoHash].
    pub fn peers(&self, info_hash: &InfoHash) -> impl Iterator<Item = &PeerEntry> {
        self.peers
            .get(info_hash)
            .map(|e| e.iter())
            .unwrap_or_default()
    }

    /// Updates the peer information for the given info hash.
    pub fn update_peer(&mut self, info_hash: InfoHash, addr: SocketAddr, seed: bool) {
        let entry = self.peers.entry(info_hash).or_default();
        entry.insert(PeerEntry::new(addr, seed));
    }
}

#[derive(Debug)]
pub struct PeerEntry {
    pub addr: SocketAddr,
    pub added: Instant,
    pub seed: bool,
}

impl PeerEntry {
    pub fn new(addr: SocketAddr, seed: bool) -> Self {
        Self {
            addr,
            added: Instant::now(),
            seed,
        }
    }
}

impl PartialEq for PeerEntry {
    fn eq(&self, other: &Self) -> bool {
        self.addr == other.addr
    }
}

impl Eq for PeerEntry {}

impl Hash for PeerEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.addr.hash(state);
    }
}
