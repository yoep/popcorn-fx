use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use itertools::Itertools;
use log::warn;
use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Formatter;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use thiserror::Error;

pub(crate) const COMPACT_IPV4_ADDR_LEN: usize = 6;
pub(crate) const COMPACT_IPV6_ADDR_LEN: usize = 18;

#[derive(Debug, Error)]
pub enum CompactError {
    #[error("invalid compact ip address byte slice")]
    InvalidLength,
    #[error("failed to parse compact ip address, {0}")]
    AddressParse(String),
}

pub type CompactResult<T> = Result<T, CompactError>;

/// A list of compact IPv4 addresses
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CompactIpv4Addrs(Vec<CompactIpv4Addr>);

impl CompactIpv4Addrs {
    /// Returns true if the addresses are empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over the compact ipv4 addresses.
    pub fn iter(&self) -> impl Iterator<Item = &CompactIpv4Addr> {
        self.0.iter()
    }

    /// Creates a consuming iterator, that is, one that moves each value out of the vector (from start to end). The vector cannot be used after calling this.
    pub fn into_iter(self) -> impl Iterator<Item = CompactIpv4Addr> {
        self.0.into_iter()
    }
}

impl Serialize for CompactIpv4Addrs {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes: Vec<u8> = self.0.iter().map(|e| e.as_bytes()).concat();
        serializer.serialize_bytes(bytes.as_slice())
    }
}

impl<'de> Deserialize<'de> for CompactIpv4Addrs {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CompactIpv4AddrsVisitor;
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
                E: serde::de::Error,
            {
                let value = BASE64_STANDARD
                    .decode(v)
                    .map_err(|e| serde::de::Error::custom(e.to_string()))?;

                CompactIpv4Addrs::try_from(value.as_slice())
                    .map_err(|e| serde::de::Error::custom(e.to_string()))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                CompactIpv4Addrs::try_from(v).map_err(|e| serde::de::Error::custom(e.to_string()))
            }

            fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut addrs = Vec::new();

                while let Ok(Some(addr)) = seq.next_element::<CompactIpv4Addr>() {
                    addrs.push(addr);
                }

                Ok(Self::Value::from(addrs))
            }
        }

        deserializer.deserialize_any(CompactIpv4AddrsVisitor {})
    }
}

impl From<Vec<CompactIpv4Addr>> for CompactIpv4Addrs {
    fn from(addrs: Vec<CompactIpv4Addr>) -> Self {
        Self(addrs)
    }
}

impl TryFrom<&[u8]> for CompactIpv4Addrs {
    type Error = CompactError;

    fn try_from(bytes: &[u8]) -> CompactResult<Self> {
        if bytes.len() % COMPACT_IPV4_ADDR_LEN != 0 {
            return Err(CompactError::InvalidLength);
        }

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

        Ok(Self(addrs))
    }
}

/// A compact IPv4 address of a torrent peer.
#[derive(Debug, Clone, PartialEq)]
pub struct CompactIpv4Addr {
    pub ip: Ipv4Addr,
    pub port: u16,
}

impl CompactIpv4Addr {
    pub fn as_bytes(&self) -> Vec<u8> {
        let ip: [u8; 4] = self.ip.octets();
        let port = self.port.to_be_bytes();
        let mut bytes = [0u8; COMPACT_IPV4_ADDR_LEN];

        bytes[0..4].copy_from_slice(&ip);
        bytes[4..].copy_from_slice(&port);

        bytes.to_vec()
    }
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
    type Error = CompactError;

    fn try_from(bytes: &[u8]) -> CompactResult<Self> {
        CompactIpv4AddrVisitor::parse_bytes(bytes)
    }
}

impl TryFrom<SocketAddr> for CompactIpv4Addr {
    type Error = CompactError;

    fn try_from(addr: SocketAddr) -> CompactResult<Self> {
        if let IpAddr::V4(ip) = addr.ip() {
            Ok(Self {
                ip,
                port: addr.port(),
            })
        } else {
            Err(CompactError::AddressParse(
                "IPv6 is not supported for CompactIpv4Addr".to_string(),
            ))
        }
    }
}

impl From<&CompactIpv4Addr> for [u8; COMPACT_IPV4_ADDR_LEN] {
    fn from(value: &CompactIpv4Addr) -> [u8; COMPACT_IPV4_ADDR_LEN] {
        let ip: [u8; 4] = value.ip.octets();
        let port = value.port.to_be_bytes();
        let mut bytes = [0u8; COMPACT_IPV4_ADDR_LEN];

        bytes[0..4].copy_from_slice(&ip);
        bytes[4..].copy_from_slice(&port);

        bytes
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
    fn parse_bytes(bytes: &[u8]) -> CompactResult<CompactIpv4Addr> {
        if bytes.len() != COMPACT_IPV4_ADDR_LEN {
            return Err(CompactError::AddressParse(
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
        E: serde::de::Error,
    {
        if v.len() != 6 {
            return Err(serde::de::Error::invalid_length(v.len(), &self));
        }

        Self::parse_bytes(v).map_err(|e| serde::de::Error::custom(e))
    }
}

/// A list of compact IPv6 addresses
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CompactIpv6Addrs(Vec<CompactIpv6Addr>);

impl CompactIpv6Addrs {
    /// Returns true if the addresses are empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over the ipv6 compact addresses.
    pub fn iter(&self) -> impl Iterator<Item = &CompactIpv6Addr> {
        self.0.iter()
    }

    /// Creates a consuming iterator, that is, one that moves each value out of the vector (from start to end).
    /// The compact addresses cannot be used after calling this.
    pub fn into_iter(self) -> impl Iterator<Item = CompactIpv6Addr> {
        self.0.into_iter()
    }
}

impl Serialize for CompactIpv6Addrs {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes: Vec<u8> = self.0.iter().map(|e| e.as_bytes()).concat();
        serializer.serialize_bytes(bytes.as_slice())
    }
}

impl<'de> Deserialize<'de> for CompactIpv6Addrs {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CompactIpv6AddrsVisitor;
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
                E: serde::de::Error,
            {
                let value = BASE64_STANDARD
                    .decode(v)
                    .map_err(|e| serde::de::Error::custom(e.to_string()))?;

                Self::Value::try_from(value.as_slice())
                    .map_err(|e| serde::de::Error::custom(e.to_string()))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Self::Value::try_from(v).map_err(|e| serde::de::Error::custom(e.to_string()))
            }

            fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut addrs = Vec::new();

                while let Ok(Some(addr)) = seq.next_element::<CompactIpv6Addr>() {
                    addrs.push(addr);
                }

                Ok(Self::Value::from(addrs))
            }
        }

        deserializer.deserialize_any(CompactIpv6AddrsVisitor)
    }
}

impl From<Vec<CompactIpv6Addr>> for CompactIpv6Addrs {
    fn from(addrs: Vec<CompactIpv6Addr>) -> Self {
        Self(addrs)
    }
}

impl TryFrom<&[u8]> for CompactIpv6Addrs {
    type Error = CompactError;

    fn try_from(bytes: &[u8]) -> std::result::Result<Self, Self::Error> {
        if bytes.len() % COMPACT_IPV6_ADDR_LEN != 0 {
            return Err(CompactError::InvalidLength);
        }

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

        Ok(Self(addrs))
    }
}

/// A compact IPv6 address
#[derive(Debug, Clone, PartialEq)]
pub struct CompactIpv6Addr {
    pub ip: Ipv6Addr,
    pub port: u16,
}

impl CompactIpv6Addr {
    /// Returns the bytes representing this compact ipv6 address.
    pub fn as_bytes(&self) -> Vec<u8> {
        let ip: [u8; 16] = self.ip.octets();
        let port = self.port.to_be_bytes();
        let mut bytes = [0u8; COMPACT_IPV6_ADDR_LEN];

        bytes[..16].copy_from_slice(&ip);
        bytes[16..].copy_from_slice(&port);

        bytes.to_vec()
    }
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
    type Error = CompactError;

    fn try_into(self) -> CompactResult<CompactIpv6Addr> {
        let ip_addr = self.ip();

        match ip_addr {
            IpAddr::V6(addr) => Ok(CompactIpv6Addr {
                ip: addr,
                port: self.port(),
            }),
            IpAddr::V4(_) => Err(CompactError::AddressParse(
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
    type Error = CompactError;

    fn try_from(bytes: &[u8]) -> CompactResult<Self> {
        CompactIpv6AddrVisitor::parse_bytes(bytes)
    }
}

impl From<&CompactIpv6Addr> for [u8; COMPACT_IPV6_ADDR_LEN] {
    fn from(value: &CompactIpv6Addr) -> [u8; COMPACT_IPV6_ADDR_LEN] {
        let ip: [u8; 16] = value.ip.octets();
        let port = value.port.to_be_bytes();
        let mut bytes = [0u8; COMPACT_IPV6_ADDR_LEN];

        bytes[..16].copy_from_slice(&ip);
        bytes[16..].copy_from_slice(&port);

        bytes
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
    fn parse_bytes(bytes: &[u8]) -> CompactResult<CompactIpv6Addr> {
        if bytes.len() != COMPACT_IPV6_ADDR_LEN {
            return Err(CompactError::AddressParse(
                "expected a byte slice of a compact ipv6 address".to_string(),
            ));
        }

        let ip_bytes: [u8; 16] = <[u8; 16]>::try_from(&bytes[0..16]).map_err(|_| {
            CompactError::AddressParse("failed to convert slice to [u8; 16]".to_string())
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
        E: serde::de::Error,
    {
        if v.len() != 18 {
            return Err(serde::de::Error::invalid_length(v.len(), &self));
        }

        Self::parse_bytes(v).map_err(|e| serde::de::Error::custom(e))
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
        E: serde::de::Error,
    {
        let addr: IpAddr;

        if bytes.len() == 4 {
            addr = IpAddr::V4(Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]));
        } else if bytes.len() == 16 {
            addr = <[u8; 16]>::try_from(&bytes[0..16])
                .map(|e| Ipv6Addr::from(e))
                .map(|e| IpAddr::V6(e))
                .map_err(|_| {
                    serde::de::Error::custom(CompactError::AddressParse(
                        "failed to convert slice to [u8; 16]".to_string(),
                    ))
                })?;
        } else {
            return Err(serde::de::Error::invalid_length(bytes.len(), &self));
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

impl CompactIpAddr {
    /// Returns the bytes representing this compact ip address.
    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            CompactIpAddr::IPv4(e) => e.as_bytes(),
            CompactIpAddr::IPv6(e) => e.as_bytes(),
        }
    }
}

impl Serialize for CompactIpAddr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
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
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CompactIpAddrVisitor;
        impl<'de> Visitor<'de> for CompactIpAddrVisitor {
            type Value = CompactIpAddr;

            fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
                write!(f, "expected a compact ip address as bytes")
            }

            fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match bytes.len() {
                    COMPACT_IPV4_ADDR_LEN => CompactIpv4Addr::try_from(bytes)
                        .map(CompactIpAddr::IPv4)
                        .map_err(|e| serde::de::Error::custom(e)),
                    COMPACT_IPV6_ADDR_LEN => CompactIpv6Addr::try_from(bytes)
                        .map(CompactIpAddr::IPv6)
                        .map_err(|e| serde::de::Error::custom(e)),
                    _ => Err(serde::de::Error::invalid_length(bytes.len(), &self)),
                }
            }
        }

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

impl From<&CompactIpAddr> for SocketAddr {
    fn from(value: &CompactIpAddr) -> Self {
        match value {
            CompactIpAddr::IPv4(addr) => addr.into(),
            CompactIpAddr::IPv6(addr) => addr.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestIpv4List {
        pub text: String,
        pub addrs: CompactIpv4Addrs,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestIpv6List {
        pub text: String,
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
            let result: CompactResult<CompactIpv4Addr> = socket.try_into();
            assert!(result.is_err(), "expected an error to be returned");

            let expected_result = CompactIpv4Addr {
                ip: Ipv4Addr::new(127, 0, 0, 1),
                port: 6881,
            };
            let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6881);
            let result = CompactIpv4Addr::try_from(socket).unwrap();
            assert_eq!(expected_result, result);
        }

        #[test]
        fn test_mod() {
            let expected_result = TestIpv4List {
                text: "test".to_string(),
                addrs: vec![CompactIpv4Addr {
                    ip: Ipv4Addr::new(127, 0, 0, 1),
                    port: 6881,
                }]
                .into(),
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
                }]
                .into(),
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
