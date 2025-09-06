use crate::torrent::peer::{
    ConnectionDirection, ConnectionProtocol, Peer, PeerClientInfo, PeerEvent, PeerHandle, PeerId,
    PeerState, PeerStats,
};
use crate::torrent::PieceIndex;
use async_trait::async_trait;
use bit_vec::BitVec;
use crc::{Crc, CRC_32_ISCSI};
use fx_callback::{Callback, Subscriber, Subscription};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Weak};
use tokio::sync::mpsc::unbounded_channel;

const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);

/// A lightweight, non-owning handle to a remote [`Peer`] managed by a torrent's pool.
///
/// As this is a weak reference to a [Peer], call [TorrentPeer::is_valid] to check if the reference is still valid.
#[derive(Debug)]
pub struct TorrentPeer {
    handle: PeerHandle,
    addr: SocketAddr,
    inner: Weak<dyn Peer>,
}

impl TorrentPeer {
    /// Create a weak handle from a strong reference to a peer.
    pub(crate) fn new(peer: &Arc<dyn Peer>) -> Self {
        Self {
            handle: peer.handle(),
            addr: peer.addr(),
            inner: Arc::downgrade(peer),
        }
    }

    /// Check if this torrent peer is still valid.
    /// It returns `false` when the underlying peer has been closed.
    pub fn is_valid(&self) -> bool {
        self.inner.strong_count() > 0
    }

    /// Try to get a strong reference to the underlying peer implementation.
    fn instance(&self) -> Option<Arc<dyn Peer>> {
        self.inner.upgrade()
    }
}

#[async_trait]
impl Peer for TorrentPeer {
    fn handle(&self) -> PeerHandle {
        self.handle
    }

    fn handle_as_ref(&self) -> &PeerHandle {
        &self.handle
    }

    fn client(&self) -> PeerClientInfo {
        if let Some(inner) = self.instance() {
            return inner.client();
        }

        PeerClientInfo {
            handle: self.handle,
            id: PeerId::new(),
            addr: self.addr.clone(),
            connection_type: ConnectionDirection::Inbound,
            connection_protocol: ConnectionProtocol::Other,
        }
    }

    fn addr(&self) -> SocketAddr {
        self.addr.clone()
    }

    fn addr_as_ref(&self) -> &SocketAddr {
        &self.addr
    }

    async fn state(&self) -> PeerState {
        if let Some(inner) = self.instance() {
            return inner.state().await;
        }

        PeerState::Closed
    }

    async fn stats(&self) -> PeerStats {
        if let Some(inner) = self.instance() {
            return inner.stats().await;
        }

        PeerStats::default()
    }

    async fn is_seed(&self) -> bool {
        if let Some(inner) = self.instance() {
            return inner.is_seed().await;
        }

        false
    }

    async fn remote_piece_bitfield(&self) -> BitVec {
        if let Some(inner) = self.instance() {
            return inner.remote_piece_bitfield().await;
        }

        BitVec::default()
    }

    fn notify_piece_availability(&self, pieces: Vec<PieceIndex>) {
        if let Some(inner) = self.instance() {
            inner.notify_piece_availability(pieces)
        }
    }

    async fn close(&self) {
        if let Some(inner) = self.instance() {
            inner.close().await
        }
    }
}

impl Callback<PeerEvent> for TorrentPeer {
    fn subscribe(&self) -> Subscription<PeerEvent> {
        if let Some(inner) = self.instance() {
            return inner.subscribe();
        }

        let (_, rx) = unbounded_channel();
        rx
    }

    fn subscribe_with(&self, subscriber: Subscriber<PeerEvent>) {
        if let Some(inner) = self.instance() {
            inner.subscribe_with(subscriber)
        }
    }
}

impl Display for TorrentPeer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(inner) = self.instance() {
            return write!(f, "{}", inner);
        }

        write!(f, "invalid torrent peer")
    }
}

/// The canonical peer priority calculated by 2 addresses.
/// See BEP40 for more information.
///
/// # Usage
///
/// ```rust,no_run
/// use std::net::SocketAddr;
/// use popcorn_fx_torrent::torrent::PeerPriority;
///
/// let left: SocketAddr = ([123, 213, 0, 1], 1234).into();
/// let right: SocketAddr = ([230, 32, 123, 23], 300).into();
///
/// PeerPriority::from((&left, &right))
/// ```
///
/// # Explanation
///
/// 1. if the IP addresses are identical, hash the ports in 16 bit network-order
///    binary representation, ordered lowest first.
/// 2. if the IPs are in the same /24, hash the IPs ordered, lowest first.
/// 3. if the IPs are in the ame /16, mask the IPs by 0xffffff55, hash them
///    ordered, lowest first.
/// 4. if IPs are not in the same /16, mask the IPs by 0xffff5555, hash them
///    ordered, lowest first.
#[derive(Debug, Clone, PartialEq)]
pub struct PeerPriority(Option<u32>);

impl PeerPriority {
    /// Create a new peer priority for the given socket addresses.
    ///
    /// The priority might be [None] if the ip versions don't match.
    pub fn new(left: &SocketAddr, right: &SocketAddr) -> Self {
        Self(Self::calculate_from(left, right))
    }

    /// The priority/rank of the peer.
    pub fn priority(&self) -> Option<u32> {
        self.0.clone()
    }

    /// Take the priority/rank of the peer, leaving [None] behind.
    pub fn take(&mut self) -> Option<u32> {
        self.0.take()
    }

    /// Try to calculate the peer priority.
    /// It returns [None] when the ip version doesn't match.
    fn calculate_from(left: &SocketAddr, right: &SocketAddr) -> Option<u32> {
        if left.is_ipv4() != right.is_ipv4() {
            return None; // cannot calculate the peer priority of different ip versions
        }

        if left.ip() == right.ip() {
            let (p1, p2) = if left.port() <= right.port() {
                (left.port().to_be_bytes(), right.port().to_be_bytes())
            } else {
                (right.port().to_be_bytes(), left.port().to_be_bytes())
            };
            return Some(Self::crc32_hash_pair(&p1, &p2));
        } else if left.is_ipv6() {
            let mut bytes = Self::ipv6_octets(left)?;
            let mut other_bytes = Self::ipv6_octets(right)?;

            let mut offset = 0xff;
            for i in 0..16 {
                if offset == 0xff && bytes[i] != other_bytes[i] {
                    offset = std::cmp::max(i + 1, 6);
                } else if i > offset {
                    bytes[i] &= 0x55;
                    other_bytes[i] &= 0x55;
                }
            }

            if left > right {
                return Some(Self::crc32_hash_pair(&other_bytes, &bytes));
            }

            return Some(Self::crc32_hash_pair(&bytes, &other_bytes));
        }

        const V4_MASKS: [[u8; 4]; 3] = [
            [0xff, 0xff, 0x55, 0x55],
            [0xff, 0xff, 0xff, 0x55],
            [0xff, 0xff, 0xff, 0xff],
        ];
        let effective_mask: &[u8; 4];

        let mut bytes = Self::ipv4_octets(left)?;
        let mut other_bytes = Self::ipv4_octets(right)?;

        // if the first 16 bytes don't match, use the default mask FF.FF.55.55,
        // if the first 16 bytes match, but not the first 24 bytes, use the mask FF.FF.FF.55,
        // if the first 24 bytes match, use the mask FF.FF.FF.FF
        if bytes[0..2] != other_bytes[0..2] {
            effective_mask = &V4_MASKS[0];
        } else if bytes[0..3] != other_bytes[0..3] {
            effective_mask = &V4_MASKS[1];
        } else {
            effective_mask = &V4_MASKS[2];
        }

        Self::apply_mask(&mut bytes, effective_mask);
        Self::apply_mask(&mut other_bytes, effective_mask);

        if left > right {
            return Some(Self::crc32_hash_pair(&other_bytes, &bytes));
        }

        Some(Self::crc32_hash_pair(&bytes, &other_bytes))
    }

    /// Create an empty peer priority.
    /// This priority has no underlying value.
    pub fn none() -> Self {
        Self(None)
    }

    /// Get the ipv4 address octets.
    fn ipv4_octets(addr: &SocketAddr) -> Option<[u8; 4]> {
        match addr.ip() {
            IpAddr::V4(addr) => Some(addr.octets()),
            _ => None,
        }
    }

    /// Get the ipv6 address octets.
    fn ipv6_octets(addr: &SocketAddr) -> Option<[u8; 16]> {
        match addr.ip() {
            IpAddr::V6(addr) => Some(addr.octets()),
            _ => None,
        }
    }

    fn apply_mask(bytes: &mut [u8], mask: &[u8]) {
        for (byte, &mask_byte) in bytes.iter_mut().zip(mask.iter()) {
            *byte &= mask_byte;
        }
    }

    fn crc32_hash_pair(left: &[u8], right: &[u8]) -> u32 {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(left);
        buffer.extend_from_slice(right);
        CRC32.checksum(&buffer)
    }
}

impl PartialOrd for PeerPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0.is_none() {
            return Some(Ordering::Greater);
        }
        if other.0.is_none() {
            return Some(Ordering::Less);
        }

        other.0.partial_cmp(&self.0)
    }
}

impl From<(&SocketAddr, &SocketAddr)> for PeerPriority {
    fn from(value: (&SocketAddr, &SocketAddr)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<u32> for PeerPriority {
    fn from(value: u32) -> Self {
        Self(Some(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    mod torrent_peer {
        use super::*;
        use crate::torrent::peer::tests::MockPeer;

        #[test]
        fn test_is_valid() {
            let mut peer = MockPeer::new();
            peer.expect_handle().return_const(PeerHandle::new());
            peer.expect_addr()
                .return_const(SocketAddr::from(([127, 0, 0, 1], 6881)));
            let peer = Arc::from(Box::new(peer) as Box<dyn Peer>);
            let torrent_peer = TorrentPeer::new(&peer);

            let result = torrent_peer.is_valid();
            assert_eq!(true, result, "expected the torrent peer to be valid");

            drop(peer);
            let result = torrent_peer.is_valid();
            assert_eq!(false, result, "expected the torrent peer to be invalid");
        }

        #[test]
        fn test_handle() {
            let handle = PeerHandle::new();
            let mut peer = MockPeer::new();
            peer.expect_handle().return_const(handle);
            peer.expect_addr()
                .return_const(SocketAddr::from(([127, 0, 0, 1], 6881)));
            let peer = Arc::from(Box::new(peer) as Box<dyn Peer>);
            let torrent_peer = TorrentPeer::new(&peer);

            let result = torrent_peer.handle();
            assert_eq!(handle, result, "expected the peer handle to match");

            drop(peer);
            let result = torrent_peer.handle();
            assert_eq!(handle, result, "expected the peer handle to match");
        }

        #[tokio::test]
        async fn test_state() {
            let mut peer = MockPeer::new();
            peer.expect_handle().return_const(PeerHandle::new());
            peer.expect_addr()
                .return_const(SocketAddr::from(([127, 0, 0, 1], 6881)));
            peer.expect_state().returning(|| PeerState::Downloading);
            let peer = Arc::from(Box::new(peer) as Box<dyn Peer>);
            let torrent_peer = TorrentPeer::new(&peer);

            let result = torrent_peer.state().await;
            assert_eq!(
                PeerState::Downloading,
                result,
                "expected the torrent state to be downloaded"
            );

            drop(peer);
            let result = torrent_peer.state().await;
            assert_eq!(
                PeerState::Closed,
                result,
                "expected the torrent state to be closed"
            );
        }
    }

    mod peer_priority {
        use super::*;

        #[test]
        fn test_different_ip_version() {
            let peer1: SocketAddr = ([123, 213, 0, 1], 5000).into();
            let peer2: SocketAddr = (
                [0x20d, 0x20c, 0x20b, 0x20a, 0x209, 0x208, 0x207, 0x206],
                4000,
            )
                .into();

            assert_eq!(None, PeerPriority::from((&peer1, &peer2)).0);
        }

        #[test]
        fn test_compare() {
            let peer1 = PeerPriority(Some(10));
            let peer2 = PeerPriority(Some(20));
            let peer3 = PeerPriority::none();

            // compare some priorities
            assert_eq!(Some(Ordering::Equal), peer1.partial_cmp(&peer1));
            assert_eq!(Some(Ordering::Greater), peer1.partial_cmp(&peer2));
            assert_eq!(Some(Ordering::Less), peer2.partial_cmp(&peer1));

            // compare none priorities
            assert_eq!(Some(Ordering::Greater), peer3.partial_cmp(&peer1));
            assert_eq!(Some(Ordering::Less), peer1.partial_cmp(&peer3));
        }

        #[cfg(test)]
        mod ipv4 {
            use super::*;

            #[test]
            fn test_peer_priority_same_ip_address() {
                let peer1: SocketAddr = ([230, 12, 123, 3], 1234).into();
                let peer2: SocketAddr = ([230, 12, 123, 3], 300).into();

                assert_eq!(
                    hash_buffer("012c04d2"),
                    PeerPriority::from((&peer1, &peer2)).0
                );
            }

            #[test]
            fn test_peer_priority_matching_24_prefix() {
                let peer1: SocketAddr = ([230, 12, 123, 1], 1234).into();
                let peer2: SocketAddr = ([230, 12, 123, 3], 300).into();

                assert_eq!(
                    hash_buffer("e60c7b01e60c7b03"),
                    PeerPriority::from((&peer1, &peer2)).0
                );
            }

            #[test]
            fn test_peer_priority_matching_24_prefix_same_port() {
                let peer1: SocketAddr = ([123, 213, 32, 10], 0).into();
                let peer2: SocketAddr = ([123, 213, 32, 234], 0).into();

                assert_eq!(Some(0x99568189), PeerPriority::from((&peer1, &peer2)).0);
            }

            #[test]
            fn test_peer_priority_matching_16_prefix() {
                let peer1: SocketAddr = ([230, 12, 23, 1], 1234).into();
                let peer2: SocketAddr = ([230, 12, 123, 3], 300).into();

                assert_eq!(
                    hash_buffer("e60c1701e60c7b01"),
                    PeerPriority::from((&peer1, &peer2)).0
                );
            }

            #[test]
            fn test_peer_priority_different_16_prefix() {
                let peer1: SocketAddr = ([230, 120, 23, 1], 1234).into();
                let peer2: SocketAddr = ([230, 12, 123, 3], 300).into();

                assert_eq!(
                    hash_buffer("e60c5101e6781501"),
                    PeerPriority::from((&peer1, &peer2)).0
                );
            }

            #[test]
            fn test_peer_priority_different_16_prefix_same_port() {
                let peer1: SocketAddr = ([123, 213, 32, 10], 0).into();
                let peer2: SocketAddr = ([98, 76, 54, 32], 0).into();

                assert_eq!(Some(0xec2d7224), PeerPriority::from((&peer1, &peer2)).0);
            }
        }

        #[cfg(test)]
        mod ipv6 {
            use super::*;
            use std::net::Ipv6Addr;

            #[test]
            fn test_peer_priority_same_address_different_port() {
                let peer1: SocketAddr = (
                    Ipv6Addr::from_str("ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff").unwrap(),
                    1234,
                )
                    .into();
                let peer2: SocketAddr = (
                    Ipv6Addr::from_str("ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff").unwrap(),
                    300,
                )
                    .into();

                assert_eq!(
                    hash_buffer("012c04d2"),
                    PeerPriority::from((&peer1, &peer2)).0
                );
                assert_eq!(
                    hash_buffer("012c04d2"),
                    PeerPriority::from((&peer2, &peer1)).0
                ); // order shouldn't matter
            }

            #[test]
            fn test_peer_priority_different_32_prefix() {
                let peer1: SocketAddr = (
                    Ipv6Addr::from_str("ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff").unwrap(),
                    1234,
                )
                    .into();
                let peer2: SocketAddr = (
                    Ipv6Addr::from_str("ffff:0fff:ffff:ffff:ffff:ffff:ffff:ffff").unwrap(),
                    300,
                )
                    .into();

                assert_eq!(Some(3916556436), PeerPriority::from((&peer1, &peer2)).0);
            }
        }

        fn hash_buffer(hex: &str) -> Option<u32> {
            if hex.len() % 2 != 0 {
                return None;
            }

            let buffer: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
                .collect::<Option<Vec<_>>>()?;
            Some(CRC32.checksum(&buffer))
        }
    }
}
