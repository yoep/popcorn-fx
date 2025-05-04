use crc::{Crc, CRC_32_ISCSI};
use std::cmp::Ordering;
use std::net::{IpAddr, SocketAddr};

const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);

/// The torrent peer address information.
#[derive(Debug)]
pub(crate) struct TorrentPeer {
    /// The address of a remote peer.
    pub addr: SocketAddr,
    /// Indicates if this peer address is in use by the torrent.
    pub is_in_use: bool,
    /// Indicates if this peer has been identified as a seed.
    pub is_seed: bool,
    /// Indicates if this peer has been banned from establishing a connection.
    pub is_banned: bool,
    /// The peer priority rank.
    pub rank: PeerPriority,
}

impl TorrentPeer {
    /// Create a new torrent peer address information.
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            is_in_use: false,
            is_seed: false,
            is_banned: false,
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
            rank,
        }
    }

    /// Check if this peer is a candidate for establishing a new connection.
    ///
    /// # Returns
    ///
    /// It returns true when the peer is a candidate, else false.
    pub fn is_connect_candidate(&self) -> bool {
        !self.is_in_use && !self.is_banned
    }
}

impl PartialEq for TorrentPeer {
    fn eq(&self, other: &Self) -> bool {
        self.addr == other.addr
    }
}

impl PartialOrd for TorrentPeer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.rank.partial_cmp(&other.rank)
    }
}

impl Eq for TorrentPeer {}

impl Ord for TorrentPeer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
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

    fn none() -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[cfg(test)]
    mod torrent_peer {
        use super::*;

        #[test]
        fn test_is_connect_candidate() {
            let peer = TorrentPeer {
                addr: ([127, 0, 0, 1], 8090).into(),
                is_in_use: false,
                is_seed: false,
                is_banned: false,
                rank: PeerPriority::none(),
            };
            assert_eq!(
                true,
                peer.is_connect_candidate(),
                "expected the peer to be a candidate"
            );

            let peer = TorrentPeer {
                addr: ([127, 0, 0, 1], 8090).into(),
                is_in_use: true,
                is_seed: false,
                is_banned: false,
                rank: PeerPriority::none(),
            };
            assert_eq!(
                false,
                peer.is_connect_candidate(),
                "expected a in-use peer to not have been a candidate"
            );

            let peer = TorrentPeer {
                addr: ([127, 0, 0, 1], 8090).into(),
                is_in_use: false,
                is_seed: false,
                is_banned: true,
                rank: PeerPriority::none(),
            };
            assert_eq!(
                false,
                peer.is_connect_candidate(),
                "expected a banned peer to not have been a candidate"
            );
        }

        #[test]
        fn test_order() {
            let peer1 = TorrentPeer {
                addr: ([127, 0, 0, 1], 8090).into(),
                is_in_use: false,
                is_seed: false,
                is_banned: false,
                rank: PeerPriority(Some(30)),
            };
            let peer2 = TorrentPeer {
                addr: ([127, 0, 0, 1], 8090).into(),
                is_in_use: false,
                is_seed: false,
                is_banned: false,
                rank: PeerPriority(Some(10)),
            };

            assert_eq!(Ordering::Less, peer1.cmp(&peer2));
            assert_eq!(Ordering::Greater, peer2.cmp(&peer1));
        }
    }

    #[cfg(test)]
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
