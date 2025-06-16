use crate::torrent::errors::Result;
use crate::torrent::TorrentError;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use log::{trace, warn};
use serde::de::{Error, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Formatter;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

const COMPACT_IPV4_ADDR_LEN: usize = 6;
const COMPACT_IPV6_ADDR_LEN: usize = 18;

/// A list of compact IPv4 addresses
pub type CompactIpv4Addrs = Vec<CompactIpv4Addr>;

struct CompactIpv4AddrsVisitor;

impl CompactIpv4AddrsVisitor {
    /// Parse multiple compact IPv4 addresses from the given bytes.
    fn parse_addrs_bytes(bytes: &[u8]) -> CompactIpv4Addrs {
        let mut addrs = Vec::new();
        let addr_count = bytes.len() / COMPACT_IPV4_ADDR_LEN;

        for i in 0..addr_count {
            let start = i * COMPACT_IPV4_ADDR_LEN;
            let end = start + COMPACT_IPV4_ADDR_LEN;

            match CompactIpv4AddrVisitor::parse_bytes(&bytes[start..end]) {
                Ok(addr) => addrs.push(addr),
                Err(e) => warn!("Failed to parse compact address, {}", e),
            }
        }

        trace!("Parsed {} addresses from compact ipv4", addrs.len());
        addrs
    }
}

impl<'de> Visitor<'de> for CompactIpv4AddrsVisitor {
    type Value = CompactIpv4Addrs;

    fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "expected a string, sequence or byte array of compact ipv4 addresses"
        )
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: Error,
    {
        let value = BASE64_STANDARD
            .decode(v)
            .map_err(|e| Error::custom(e.to_string()))?;

        Ok(Self::parse_addrs_bytes(value.as_ref()))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Self::parse_addrs_bytes(v))
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut addrs = Vec::new();

        while let Ok(Some(addr)) = seq.next_element::<CompactIpv4Addr>() {
            addrs.push(addr);
        }

        Ok(addrs)
    }
}

/// A compact IPv4 address of a torrent peer.
#[derive(Debug, Clone, PartialEq)]
pub struct CompactIpv4Addr {
    pub ip: Ipv4Addr,
    pub port: u16,
}

impl Into<SocketAddr> for CompactIpv4Addr {
    fn into(self) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(self.ip), self.port)
    }
}

impl Into<IpAddr> for CompactIpv4Addr {
    fn into(self) -> IpAddr {
        IpAddr::V4(self.ip)
    }
}

impl From<&CompactIpv4Addr> for IpAddr {
    fn from(addr: &CompactIpv4Addr) -> Self {
        IpAddr::V4(addr.ip)
    }
}

impl From<&CompactIpv4Addr> for SocketAddr {
    fn from(value: &CompactIpv4Addr) -> Self {
        SocketAddr::new(IpAddr::V4(value.ip), value.port)
    }
}

impl TryFrom<&[u8]> for CompactIpv4Addr {
    type Error = TorrentError;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        CompactIpv4AddrVisitor::parse_bytes(bytes)
    }
}

impl TryFrom<SocketAddr> for CompactIpv4Addr {
    type Error = TorrentError;

    fn try_from(addr: SocketAddr) -> Result<Self> {
        if let IpAddr::V4(ip) = addr.ip() {
            Ok(Self {
                ip,
                port: addr.port(),
            })
        } else {
            Err(TorrentError::AddressParse(
                "IPv6 is not supported for CompactIpv4Addr".to_string(),
            ))
        }
    }
}

impl From<&CompactIpv4Addr> for [u8; COMPACT_IPV4_ADDR_LEN] {
    fn from(value: &CompactIpv4Addr) -> [u8; COMPACT_IPV4_ADDR_LEN] {
        let ip: [u8; 4] = value.ip.octets();
        let port = value.port.to_be_bytes();

        [ip[0], ip[1], ip[2], ip[3], port[0], port[1]]
    }
}

impl Serialize for CompactIpv4Addr {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes: [u8; 6] = self.into();
        serializer.serialize_bytes(bytes.as_slice())
    }
}

impl<'de> Deserialize<'de> for CompactIpv4Addr {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(CompactIpv4AddrVisitor {})
    }
}

struct CompactIpv4AddrVisitor;

impl CompactIpv4AddrVisitor {
    /// Parse a single compact IPv4 address from the given bytes.
    fn parse_bytes(bytes: &[u8]) -> Result<CompactIpv4Addr> {
        if bytes.len() != COMPACT_IPV4_ADDR_LEN {
            return Err(TorrentError::AddressParse(
                "expected a byte slice of a compact ipv4 address".to_string(),
            ));
        }

        let ip = Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]);
        let port = u16::from_be_bytes([bytes[4], bytes[5]]);
        Ok(CompactIpv4Addr { ip, port })
    }
}

impl<'de> Visitor<'de> for CompactIpv4AddrVisitor {
    type Value = CompactIpv4Addr;

    fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "expected a byte slice of a compact ipv4 address")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
    where
        E: Error,
    {
        if v.len() != 6 {
            return Err(Error::invalid_length(v.len(), &self));
        }

        Self::parse_bytes(v).map_err(|e| Error::custom(e))
    }
}

pub mod compact_ipv4 {
    use super::*;
    use crate::torrent::CompactIpv4Addrs;
    use serde::Deserializer;

    pub fn serialize<S>(
        addrs: &CompactIpv4Addrs,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde::Serialize::serialize(&addrs, serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<CompactIpv4Addrs, D::Error>
    where
        D: Deserializer<'de>,
    {
        D::deserialize_any(deserializer, CompactIpv4AddrsVisitor {})
    }
}

/// A list of compact IPv6 addresses
pub type CompactIpv6Addrs = Vec<CompactIpv6Addr>;

struct CompactIpv6AddrsVisitor;

impl CompactIpv6AddrsVisitor {
    fn parse_bytes(bytes: &[u8]) -> CompactIpv6Addrs {
        let mut addrs = Vec::new();
        let addr_count = bytes.len() / COMPACT_IPV6_ADDR_LEN;

        for i in 0..addr_count {
            let start = i * COMPACT_IPV6_ADDR_LEN;
            let end = start + COMPACT_IPV6_ADDR_LEN;

            match CompactIpv6AddrVisitor::parse_bytes(&bytes[start..end]) {
                Ok(addr) => addrs.push(addr),
                Err(e) => warn!("Failed to parse compact address, {}", e),
            }
        }

        trace!("Parsed {} addresses from compact ipv6", addrs.len());
        addrs
    }
}

impl<'de> Visitor<'de> for CompactIpv6AddrsVisitor {
    type Value = CompactIpv6Addrs;

    fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "expected a string, sequence or byte array of compact ipv6 addresses"
        )
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: Error,
    {
        let value = BASE64_STANDARD
            .decode(v)
            .map_err(|e| Error::custom(e.to_string()))?;

        Ok(Self::parse_bytes(value.as_ref()))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Self::parse_bytes(v))
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut addrs = Vec::new();

        while let Ok(Some(addr)) = seq.next_element::<CompactIpv6Addr>() {
            addrs.push(addr);
        }

        Ok(addrs)
    }
}

/// A compact IPv6 address
#[derive(Debug, Clone, PartialEq)]
pub struct CompactIpv6Addr {
    pub ip: Ipv6Addr,
    pub port: u16,
}

impl Into<SocketAddr> for CompactIpv6Addr {
    fn into(self) -> SocketAddr {
        SocketAddr::new(IpAddr::V6(self.ip), self.port)
    }
}

impl Into<IpAddr> for CompactIpv6Addr {
    fn into(self) -> IpAddr {
        IpAddr::V6(self.ip)
    }
}

impl TryInto<CompactIpv6Addr> for SocketAddr {
    type Error = TorrentError;

    fn try_into(self) -> Result<CompactIpv6Addr> {
        let ip_addr = self.ip();

        match ip_addr {
            IpAddr::V6(addr) => Ok(CompactIpv6Addr {
                ip: addr,
                port: self.port(),
            }),
            IpAddr::V4(_) => Err(TorrentError::AddressParse(
                "expected ipv6, but got ipv4 instead".to_string(),
            )),
        }
    }
}

impl From<&CompactIpv6Addr> for IpAddr {
    fn from(addr: &CompactIpv6Addr) -> Self {
        IpAddr::V6(addr.ip)
    }
}

impl From<&CompactIpv6Addr> for SocketAddr {
    fn from(value: &CompactIpv6Addr) -> Self {
        SocketAddr::new(IpAddr::V6(value.ip), value.port)
    }
}

impl TryFrom<&[u8]> for CompactIpv6Addr {
    type Error = TorrentError;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        CompactIpv6AddrVisitor::parse_bytes(bytes)
    }
}

impl Serialize for CompactIpv6Addr {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let ip_bytes: [u8; 16] = self.ip.octets();
        let port_bytes = self.port.to_be_bytes();

        let mut bytes = Vec::with_capacity(COMPACT_IPV6_ADDR_LEN);
        bytes.extend_from_slice(&ip_bytes);
        bytes.extend_from_slice(&port_bytes);

        serializer.serialize_bytes(&bytes)
    }
}

impl<'de> Deserialize<'de> for CompactIpv6Addr {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(CompactIpv6AddrVisitor {})
    }
}

struct CompactIpv6AddrVisitor;

impl CompactIpv6AddrVisitor {
    fn parse_bytes(bytes: &[u8]) -> Result<CompactIpv6Addr> {
        if bytes.len() != COMPACT_IPV6_ADDR_LEN {
            return Err(TorrentError::AddressParse(
                "expected a byte slice of a compact ipv6 address".to_string(),
            ));
        }

        let ip_bytes: [u8; 16] = <[u8; 16]>::try_from(&bytes[0..16]).map_err(|_| {
            TorrentError::AddressParse("failed to convert slice to [u8; 16]".to_string())
        })?;
        let ip = Ipv6Addr::from(ip_bytes);
        let port = u16::from_be_bytes([bytes[16], bytes[17]]);
        Ok(CompactIpv6Addr { ip, port })
    }
}

impl<'de> Visitor<'de> for CompactIpv6AddrVisitor {
    type Value = CompactIpv6Addr;

    fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "expected a byte slice of a compact ipv6 address")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
    where
        E: Error,
    {
        if v.len() != 18 {
            return Err(Error::invalid_length(v.len(), &self));
        }

        Self::parse_bytes(v).map_err(|e| Error::custom(e))
    }
}

pub mod compact_ipv6 {
    use super::*;
    use serde::Deserializer;

    pub fn serialize<S>(
        addrs: &CompactIpv6Addrs,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde::Serialize::serialize(&addrs, serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<CompactIpv6Addrs, D::Error>
    where
        D: Deserializer<'de>,
    {
        D::deserialize_any(deserializer, CompactIpv6AddrsVisitor)
    }
}

/// A compact representation of an IPv44 or IPv66 address without port.
#[derive(Debug, Clone, PartialEq)]
pub struct CompactIp {
    pub ip: IpAddr,
}

impl From<&SocketAddr> for CompactIp {
    fn from(value: &SocketAddr) -> Self {
        Self { ip: value.ip() }
    }
}

impl Serialize for CompactIp {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes: Vec<u8> = match &self.ip {
            IpAddr::V4(addr) => addr.octets().to_vec(),
            IpAddr::V6(addr) => addr.octets().to_vec(),
        };

        serializer.serialize_bytes(&bytes)
    }
}

impl<'de> Deserialize<'de> for CompactIp {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(CompactIpVisitor)
    }
}

struct CompactIpVisitor;

impl<'de> Visitor<'de> for CompactIpVisitor {
    type Value = CompactIp;

    fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "expected a compact ip address as bytes")
    }

    fn visit_bytes<E>(self, bytes: &[u8]) -> std::result::Result<Self::Value, E>
    where
        E: Error,
    {
        let addr: IpAddr;

        if bytes.len() == 4 {
            addr = IpAddr::V4(Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]));
        } else if bytes.len() == 16 {
            addr = <[u8; 16]>::try_from(&bytes[0..16])
                .map(|e| Ipv6Addr::from(e))
                .map(|e| IpAddr::V6(e))
                .map_err(|_| {
                    Error::custom(TorrentError::AddressParse(
                        "failed to convert slice to [u8; 16]".to_string(),
                    ))
                })?;
        } else {
            return Err(Error::invalid_length(bytes.len(), &self));
        }

        Ok(CompactIp { ip: addr })
    }
}

/// A compact IP address which can either be represented as IPv4 or IPv6.
#[derive(Debug, Clone, PartialEq)]
pub enum CompactIpAddr {
    IPv4(CompactIpv4Addr),
    IPv6(CompactIpv6Addr),
}

impl Serialize for CompactIpAddr {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            CompactIpAddr::IPv4(e) => e.serialize(serializer),
            CompactIpAddr::IPv6(e) => e.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for CompactIpAddr {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(CompactIpAddrVisitor)
    }
}

impl From<CompactIpv4Addr> for CompactIpAddr {
    fn from(value: CompactIpv4Addr) -> Self {
        Self::IPv4(value)
    }
}

impl From<CompactIpv6Addr> for CompactIpAddr {
    fn from(value: CompactIpv6Addr) -> Self {
        Self::IPv6(value)
    }
}

impl From<SocketAddr> for CompactIpAddr {
    fn from(value: SocketAddr) -> Self {
        if value.is_ipv4() {
            match value.try_into() {
                Ok(addr) => CompactIpAddr::IPv4(addr),
                Err(e) => unreachable!("The socket address should be IPv4 compatible, {}", e),
            }
        } else {
            match value.try_into() {
                Ok(addr) => CompactIpAddr::IPv6(addr),
                Err(e) => unreachable!("The socket address should be IPv6 compatible, {}", e),
            }
        }
    }
}

struct CompactIpAddrVisitor;

impl<'de> Visitor<'de> for CompactIpAddrVisitor {
    type Value = CompactIpAddr;

    fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "expected a compact ip address as bytes")
    }

    fn visit_bytes<E>(self, bytes: &[u8]) -> std::result::Result<Self::Value, E>
    where
        E: Error,
    {
        match bytes.len() {
            COMPACT_IPV4_ADDR_LEN => CompactIpv4Addr::try_from(bytes)
                .map(CompactIpAddr::IPv4)
                .map_err(|e| Error::custom(e)),
            COMPACT_IPV6_ADDR_LEN => CompactIpv6Addr::try_from(bytes)
                .map(CompactIpAddr::IPv6)
                .map_err(|e| Error::custom(e)),
            _ => Err(Error::invalid_length(bytes.len(), &self)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestIpv4List {
        pub text: String,
        #[serde(with = "compact_ipv4")]
        pub addrs: CompactIpv4Addrs,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestIpv6List {
        pub text: String,
        #[serde(with = "compact_ipv6")]
        pub addrs: CompactIpv6Addrs,
    }

    #[cfg(test)]
    mod tests_compact_ipv4 {
        use super::*;

        #[test]
        fn test_addr() {
            let expected_result = CompactIpv4Addr {
                ip: Ipv4Addr::new(127, 0, 0, 1),
                port: 6881,
            };
            let bytes = serde_bencode::to_bytes(&expected_result).unwrap();

            let result = serde_bencode::from_bytes::<CompactIpv4Addr>(&bytes).unwrap();

            assert_eq!(expected_result, result);
        }

        #[test]
        fn test_try_into_compact_addr() {
            let socket = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 6881);
            let result: Result<CompactIpv4Addr> = socket.try_into();
            assert!(result.is_err(), "expected an error to be returned");

            let expected_result = CompactIpv4Addr {
                ip: Ipv4Addr::new(127, 0, 0, 1),
                port: 6881,
            };
            let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6881);
            let result: Result<CompactIpv4Addr> = socket.try_into();
            assert_eq!(Ok(expected_result), result);
        }

        #[test]
        fn test_mod() {
            let expected_result = TestIpv4List {
                text: "test".to_string(),
                addrs: vec![CompactIpv4Addr {
                    ip: Ipv4Addr::new(127, 0, 0, 1),
                    port: 6881,
                }],
            };
            let bytes = serde_bencode::to_bytes(&expected_result).unwrap();

            let result = serde_bencode::from_bytes::<TestIpv4List>(&bytes).unwrap();

            assert_eq!(expected_result, result);
        }

        #[test]
        fn test_byte_slice_from_compact() {
            let compact_addr = CompactIpv4Addr {
                ip: Ipv4Addr::new(127, 0, 0, 1),
                port: 6881,
            };
            let expected_result: [u8; 6] = [127, 0, 0, 1, 26, 225];

            let result: [u8; 6] = (&compact_addr).into();

            assert_eq!(expected_result, result);
        }
    }

    #[cfg(test)]
    mod tests_compact_ipv6 {
        use super::*;

        #[test]
        fn test_compact_ipv6_addr() {
            let expected_result = CompactIpv6Addr {
                ip: Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1),
                port: 9090,
            };
            let bytes = serde_bencode::to_bytes(&expected_result).unwrap();

            let result = serde_bencode::from_bytes::<CompactIpv6Addr>(&bytes).unwrap();

            assert_eq!(expected_result, result);
        }

        #[test]
        fn test_compact_ipv6_mod() {
            let expected_result = TestIpv6List {
                text: "test".to_string(),
                addrs: vec![CompactIpv6Addr {
                    ip: Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1),
                    port: 9090,
                }],
            };
            let bytes = serde_bencode::to_bytes(&expected_result).unwrap();

            let result = serde_bencode::from_bytes::<TestIpv6List>(&bytes).unwrap();

            assert_eq!(expected_result, result);
        }
    }

    #[test]
    fn test_compact_ip() {
        let expected_result = CompactIp {
            ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        };
        let bytes = serde_bencode::to_bytes(&expected_result).unwrap();
        let result = serde_bencode::from_bytes(&bytes).unwrap();
        assert_eq!(expected_result, result);

        let expected_result = CompactIp {
            ip: IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 16)),
        };
        let bytes = serde_bencode::to_bytes(&expected_result).unwrap();
        let result = serde_bencode::from_bytes(&bytes).unwrap();
        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_compact_ip_addr() {
        let expected_result = CompactIpAddr::IPv4(CompactIpv4Addr {
            ip: Ipv4Addr::new(127, 0, 0, 1),
            port: 9090,
        });
        let bytes = serde_bencode::to_bytes(&expected_result).unwrap();
        let result = serde_bencode::from_bytes::<CompactIpAddr>(&bytes).unwrap();
        assert_eq!(expected_result, result);

        let expected_result = CompactIpAddr::IPv6(CompactIpv6Addr {
            ip: Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 16),
            port: 20021,
        });
        let bytes = serde_bencode::to_bytes(&expected_result).unwrap();
        let result = serde_bencode::from_bytes::<CompactIpAddr>(&bytes).unwrap();
        assert_eq!(expected_result, result);
    }
}
