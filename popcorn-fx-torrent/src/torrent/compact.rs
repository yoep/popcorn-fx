use crate::torrent::{Result, TorrentError};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use log::{trace, warn};
use serde::de::{Error, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::f32::consts::E;
use std::fmt::Formatter;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

/// A list of compact IPv4 addresses
pub type CompactIpv4Addrs = Vec<CompactIpv4Addr>;

struct CompactIpv4AddrsVisitor;

impl CompactIpv4AddrsVisitor {
    fn parse_bytes(bytes: &[u8]) -> CompactIpv4Addrs {
        let mut addrs = Vec::new();
        let addr_count = bytes.len() / 6;

        for i in 0..addr_count {
            let start = i * 6;
            let end = start + 6;

            match CompactIpv4AddrVisitor::parse_bytes(&bytes[start..end]) {
                Ok(addr) => addrs.push(addr),
                Err(e) => warn!("Failed to parse compact address, {}", e),
            }
        }

        trace!("Parsed {} addresses from compact ipv4", addrs.len());
        addrs
    }
}

impl<'de> serde::de::Visitor<'de> for CompactIpv4AddrsVisitor {
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

        Ok(Self::parse_bytes(value.as_ref()))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Self::parse_bytes(v))
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

/// A compact IPv4 address
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

impl TryInto<CompactIpv4Addr> for SocketAddr {
    type Error = TorrentError;

    fn try_into(self) -> Result<CompactIpv4Addr> {
        let ip_addr = self.ip();

        match ip_addr {
            IpAddr::V4(addr) => Ok(CompactIpv4Addr {
                ip: addr,
                port: self.port(),
            }),
            IpAddr::V6(_) => Err(TorrentError::AddressParse(
                "expected ipv4, but got ipv6 instead".to_string(),
            )),
        }
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

impl Serialize for CompactIpv4Addr {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let ip_bytes: [u8; 4] = self.ip.octets();
        let port_bytes = self.port.to_be_bytes();

        let mut bytes = Vec::with_capacity(6);
        bytes.extend_from_slice(&ip_bytes);
        bytes.extend_from_slice(&port_bytes);

        serializer.serialize_bytes(&bytes)
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
    fn parse_bytes(bytes: &[u8]) -> Result<CompactIpv4Addr> {
        if bytes.len() != 6 {
            return Err(TorrentError::AddressParse(
                "expected a byte slice of a compact ipv4 address".to_string(),
            ));
        }

        let ip = Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]);
        let port = u16::from_be_bytes([bytes[4], bytes[5]]);
        Ok(CompactIpv4Addr { ip, port })
    }
}

impl<'de> serde::de::Visitor<'de> for CompactIpv4AddrVisitor {
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
        let addr_count = bytes.len() / 18;

        for i in 0..addr_count {
            let start = i * 18;
            let end = start + 18;

            match CompactIpv6AddrVisitor::parse_bytes(&bytes[start..end]) {
                Ok(addr) => addrs.push(addr),
                Err(e) => warn!("Failed to parse compact address, {}", e),
            }
        }

        trace!("Parsed {} addresses from compact ipv6", addrs.len());
        addrs
    }
}

impl<'de> serde::de::Visitor<'de> for CompactIpv6AddrsVisitor {
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

        Ok(Self::parse_bytes(value.as_ref()))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
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

impl Serialize for CompactIpv6Addr {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let ip_bytes: [u8; 16] = self.ip.octets();
        let port_bytes = self.port.to_be_bytes();

        let mut bytes = Vec::with_capacity(18);
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
        if bytes.len() != 18 {
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

impl<'de> serde::de::Visitor<'de> for CompactIpv6AddrVisitor {
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

pub mod compact_ipv6 {
    use super::*;
    use serde::Deserializer;

    pub fn serialize<S>(
        addrs: &CompactIpv6Addrs,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
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

/// A compact representation of an ipv4 or ipv6 address without port.
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

    #[test]
    fn test_compact_ipv4_addr() {
        let expected_result = CompactIpv4Addr {
            ip: Ipv4Addr::new(127, 0, 0, 1),
            port: 6881,
        };
        let bytes = serde_bencode::to_bytes(&expected_result).unwrap();

        let result = serde_bencode::from_bytes::<CompactIpv4Addr>(&bytes).unwrap();

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_compact_ipv4_try_into() {
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
    fn test_compact_ipv4_mod() {
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
}
