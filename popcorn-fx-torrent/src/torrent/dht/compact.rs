use crate::torrent::dht::{Error, Node, NodeId, Result};
use crate::torrent::{CompactIpv4Addr, CompactIpv6Addr};
use itertools::Itertools;
use log::warn;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};

const IPV4_NODE_SIZE: usize = 26;
const IPV6_NODE_SIZE: usize = 38;

pub trait CompactIpNode {
    /// Returns the underlying compact node address as a byte slice.
    fn as_bytes(&self) -> Vec<u8>;
}

pub trait CompactIpNodes {
    /// Returns the underlying compact address nodes as a byte slice.
    fn as_bytes(&self) -> Vec<u8>;
}

/// A list of compact IPv4 nodes.
#[derive(Debug, Default, PartialEq)]
pub struct CompactIPv4Nodes(Vec<CompactIPv4Node>);

impl CompactIPv4Nodes {
    /// Check if the node vector is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of elements in the vector, also referred to as its 'length'.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Creates a consuming iterator, that is, one that moves each value out of the vector (from start to end).
    /// The vector cannot be used after calling this.
    pub fn into_iter(self) -> impl Iterator<Item = CompactIPv4Node> {
        self.0.into_iter()
    }

    /// The slice of compact IPv4 nodes.
    pub fn as_slice(&self) -> &[CompactIPv4Node] {
        self.0.as_slice()
    }
}

impl CompactIpNodes for CompactIPv4Nodes {
    fn as_bytes(&self) -> Vec<u8> {
        self.0.iter().map(CompactIpNode::as_bytes).concat()
    }
}

impl Serialize for CompactIPv4Nodes {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(self.as_bytes().as_slice())
    }
}

impl<'de> Deserialize<'de> for CompactIPv4Nodes {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CompactIPv4NodesVisitor;
        impl<'de> Visitor<'de> for CompactIPv4NodesVisitor {
            type Value = CompactIPv4Nodes;

            fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
                write!(f, "expected a byte slice")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                CompactIPv4Nodes::try_from(v).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_bytes(CompactIPv4NodesVisitor)
    }
}

impl TryFrom<&[u8]> for CompactIPv4Nodes {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        if bytes.len() % IPV4_NODE_SIZE != 0 {
            return Err(Error::Parse(
                "expected bytes matching a list of compact IPv4 nodes".to_string(),
            ));
        }

        let node_count = bytes.len() / IPV4_NODE_SIZE;
        let mut nodes = Vec::new();

        for i in 0..node_count {
            let start = i * IPV4_NODE_SIZE;
            let end = start + IPV4_NODE_SIZE;

            match CompactIPv4Node::try_from(&bytes[start..end]) {
                Ok(node) => nodes.push(node),
                Err(e) => warn!("Failed to parse compact IPv4 node, {}", e),
            }
        }

        Ok(Self(nodes))
    }
}

impl From<Vec<CompactIPv4Node>> for CompactIPv4Nodes {
    fn from(value: Vec<CompactIPv4Node>) -> Self {
        Self(value)
    }
}

impl From<&CompactIPv4Nodes> for Vec<u8> {
    fn from(value: &CompactIPv4Nodes) -> Self {
        let mut buffer = vec![0u8; IPV4_NODE_SIZE * value.0.len()];

        for (i, node) in value.0.iter().enumerate() {
            let start = i * IPV4_NODE_SIZE;
            let end = start + IPV4_NODE_SIZE;

            buffer[start..end].clone_from_slice(node.as_bytes().as_slice());
        }

        buffer
    }
}

impl Display for CompactIPv4Nodes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let bytes: Vec<u8> = self.into();
        write!(f, "{}", String::from_utf8_lossy(&bytes))
    }
}

/// The compact representation of a node with an IPv4 address.
#[derive(Debug, Clone, PartialEq)]
pub struct CompactIPv4Node {
    pub id: NodeId,
    pub addr: CompactIpv4Addr,
}

impl CompactIpNode for CompactIPv4Node {
    fn as_bytes(&self) -> Vec<u8> {
        Vec::<u8>::from(self)
    }
}

impl Serialize for CompactIPv4Node {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(self.as_bytes().as_slice())
    }
}

impl<'de> Deserialize<'de> for CompactIPv4Node {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CompactIPv4NodeVisitor;
        impl<'de> Visitor<'de> for CompactIPv4NodeVisitor {
            type Value = CompactIPv4Node;

            fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
                write!(f, "expected a byte slice")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                CompactIPv4Node::try_from(v).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_bytes(CompactIPv4NodeVisitor)
    }
}

impl TryFrom<&[u8]> for CompactIPv4Node {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != IPV4_NODE_SIZE {
            return Err(Error::Parse(format!(
                "expected {} bytes, but got {} instead",
                IPV4_NODE_SIZE,
                bytes.len()
            )));
        }

        let id = NodeId::try_from(&bytes[..20]).map_err(|e| Error::Parse(e.to_string()))?;
        let addr =
            CompactIpv4Addr::try_from(&bytes[20..]).map_err(|e| Error::Parse(e.to_string()))?;

        Ok(Self { addr, id })
    }
}

impl TryFrom<&Node> for CompactIPv4Node {
    type Error = Error;

    fn try_from(value: &Node) -> Result<Self> {
        Ok(Self {
            addr: value
                .addr()
                .clone()
                .try_into()
                .map_err(|_| Error::InvalidAddr)?,
            id: *value.id(),
        })
    }
}

impl From<&CompactIPv4Node> for Vec<u8> {
    fn from(value: &CompactIPv4Node) -> Self {
        let mut buffer = [0u8; IPV4_NODE_SIZE];
        let addr: [u8; 6] = (&value.addr).into();

        buffer[..20].copy_from_slice(value.id.as_node_slice());
        buffer[20..].copy_from_slice(&addr);

        buffer.to_vec()
    }
}

impl From<CompactIPv4Node> for Node {
    fn from(value: CompactIPv4Node) -> Self {
        Self::new(value.id, value.addr.into())
    }
}

/// The compact representation of a node with an IPv6 address.
#[derive(Debug, Clone, PartialEq)]
pub struct CompactIPv6Node {
    pub id: NodeId,
    pub addr: CompactIpv6Addr,
}

impl CompactIpNode for CompactIPv6Node {
    fn as_bytes(&self) -> Vec<u8> {
        Vec::<u8>::from(self)
    }
}

impl TryFrom<&[u8]> for CompactIPv6Node {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != IPV6_NODE_SIZE {
            return Err(Error::Parse(format!(
                "expected {} bytes, but got {} instead",
                IPV6_NODE_SIZE,
                bytes.len()
            )));
        }

        let id = NodeId::try_from(&bytes[..20]).map_err(|e| Error::Parse(e.to_string()))?;
        let addr =
            CompactIpv6Addr::try_from(&bytes[20..]).map_err(|e| Error::Parse(e.to_string()))?;

        Ok(Self { addr, id })
    }
}

impl From<&CompactIPv6Node> for Vec<u8> {
    fn from(value: &CompactIPv6Node) -> Self {
        let mut buffer = [0u8; IPV6_NODE_SIZE];
        let addr: [u8; 18] = (&value.addr).into();

        buffer[..20].copy_from_slice(value.id.as_node_slice());
        buffer[20..].copy_from_slice(&addr);

        buffer.to_vec()
    }
}

/// A list of compact IPv6 nodes.
#[derive(Debug, Default, PartialEq)]
pub struct CompactIPv6Nodes(Vec<CompactIPv6Node>);

impl CompactIPv6Nodes {
    /// Check if the node vector is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Creates a consuming iterator, that is, one that moves each value out of the vector (from start to end).
    /// The vector cannot be used after calling this.
    pub fn into_iter(self) -> impl Iterator<Item = CompactIPv6Node> {
        self.0.into_iter()
    }

    /// The slice of compact IPv6 nodes.
    pub fn as_slice(&self) -> &[CompactIPv6Node] {
        self.0.as_slice()
    }
}

impl CompactIpNodes for CompactIPv6Nodes {
    fn as_bytes(&self) -> Vec<u8> {
        self.0.iter().map(CompactIpNode::as_bytes).concat()
    }
}

impl Serialize for CompactIPv6Nodes {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(self.as_bytes().as_slice())
    }
}

impl<'de> Deserialize<'de> for CompactIPv6Nodes {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CompactIPv6NodesVisitor;
        impl<'de> Visitor<'de> for CompactIPv6NodesVisitor {
            type Value = CompactIPv6Nodes;

            fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
                write!(f, "expected a byte slice")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                CompactIPv6Nodes::try_from(v).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_bytes(CompactIPv6NodesVisitor)
    }
}

impl TryFrom<&[u8]> for CompactIPv6Nodes {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        if bytes.len() % IPV6_NODE_SIZE != 0 {
            return Err(Error::Parse(
                "expected bytes matching a list of compact IPv6 nodes".to_string(),
            ));
        }

        let node_count = bytes.len() / IPV6_NODE_SIZE;
        let mut nodes = Vec::new();

        for i in 0..node_count {
            let start = i * IPV6_NODE_SIZE;
            let end = start + IPV6_NODE_SIZE;

            match CompactIPv6Node::try_from(&bytes[start..end]) {
                Ok(node) => nodes.push(node),
                Err(e) => warn!("Failed to parse compact IPv6 node, {}", e),
            }
        }

        Ok(Self(nodes))
    }
}

impl From<Vec<CompactIPv6Node>> for CompactIPv6Nodes {
    fn from(value: Vec<CompactIPv6Node>) -> Self {
        Self(value)
    }
}

impl From<&CompactIPv6Nodes> for Vec<u8> {
    fn from(value: &CompactIPv6Nodes) -> Vec<u8> {
        let mut buffer = vec![0u8; IPV6_NODE_SIZE * value.0.len()];

        for (i, node) in value.0.iter().enumerate() {
            let start = i * IPV6_NODE_SIZE;
            let end = start + IPV6_NODE_SIZE;
            let bytes = node.as_bytes();

            buffer[start..end].copy_from_slice(&bytes);
        }

        buffer
    }
}

impl From<CompactIPv6Node> for Node {
    fn from(value: CompactIPv6Node) -> Self {
        Self::new(value.id, value.addr.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod compact_ipv4 {
        use super::*;

        use std::net::Ipv4Addr;

        #[test]
        fn test_as_bytes() {
            let ip: Ipv4Addr = [127, 0, 0, 1].into();
            let port = 5000;
            let node = CompactIPv4Node {
                id: NodeId::new(),
                addr: CompactIpv4Addr {
                    ip: ip.clone(),
                    port,
                },
            };
            let expected_result = {
                let mut buffer = [0u8; IPV4_NODE_SIZE];
                buffer[..20].copy_from_slice(node.id.as_node_slice());
                buffer[20..24].copy_from_slice(&[127, 0, 0, 1]);
                buffer[24..].copy_from_slice(port.to_be_bytes().as_slice());
                buffer.to_vec()
            };

            let results = node.as_bytes();

            assert_eq!(results.len(), IPV4_NODE_SIZE);
            assert_eq!(expected_result, results, "expected the bytes to match");
        }

        #[test]
        fn test_from_bytes() {
            let id = NodeId::new();
            let ip: Ipv4Addr = [180, 190, 0, 13].into();
            let port: u16 = 9886;
            let bytes = as_bytes(id, ip, port);

            let result = CompactIPv4Node::try_from(&bytes[..]).expect("expected a compact node");

            assert_eq!(id, result.id, "expected the node ID to match");
            assert_eq!(ip, result.addr.ip, "expected the IP address to match");
            assert_eq!(port, result.addr.port, "expected the port to match");
        }

        #[test]
        fn test_deserialize_node() {
            let id = NodeId::new();
            let ip: Ipv4Addr = [170, 15, 20, 0].into();
            let port: u16 = 8769;
            let expected_result = CompactIPv4Node {
                id,
                addr: CompactIpv4Addr { ip, port },
            };
            let bytes = serde_bencode::to_bytes(&expected_result).unwrap();

            let result = serde_bencode::from_bytes::<CompactIPv4Node>(bytes.as_slice()).unwrap();

            assert_eq!(
                expected_result, result,
                "expected the deserialized node to match"
            );
        }

        #[test]
        fn test_try_from_compact_nodes() {
            let nodes = CompactIPv4Nodes(vec![
                CompactIPv4Node {
                    id: NodeId::new(),
                    addr: CompactIpv4Addr {
                        ip: [127, 0, 0, 1].into(),
                        port: 5000,
                    },
                },
                CompactIPv4Node {
                    id: NodeId::new(),
                    addr: CompactIpv4Addr {
                        ip: [127, 0, 0, 2].into(),
                        port: 6080,
                    },
                },
            ]);
            let bytes = Vec::<u8>::from(&nodes);

            let result = CompactIPv4Nodes::try_from(bytes.as_slice()).unwrap();

            assert_eq!(
                nodes.len(),
                result.len(),
                "expected the number of nodes to match"
            );
            assert_eq!(nodes.0[0], result.0[0]);
            assert_eq!(nodes.0[1], result.0[1]);
        }

        fn as_bytes(id: NodeId, ip: Ipv4Addr, port: u16) -> Vec<u8> {
            let mut buffer = [0u8; IPV4_NODE_SIZE];
            buffer[..20].copy_from_slice(id.as_node_slice());
            buffer[20..24].copy_from_slice(ip.octets().as_slice());
            buffer[24..].copy_from_slice(port.to_be_bytes().as_slice());
            buffer.to_vec()
        }
    }

    mod compact_ipv6 {
        use super::*;
        use std::net::Ipv6Addr;

        #[test]
        fn test_as_bytes() {
            let ip = Ipv6Addr::LOCALHOST;
            let port = 8661;
            let node = CompactIPv6Node {
                id: NodeId::new(),
                addr: CompactIpv6Addr { ip, port },
            };
            let expected_result = {
                let mut buffer = [0u8; IPV6_NODE_SIZE];
                buffer[..20].copy_from_slice(node.id.as_node_slice());
                buffer[20..36].copy_from_slice(&ip.octets());
                buffer[36..].copy_from_slice(port.to_be_bytes().as_slice());
                buffer.to_vec()
            };

            let results = node.as_bytes();

            assert_eq!(results.len(), IPV6_NODE_SIZE);
            assert_eq!(expected_result, results, "expected the bytes to match");
        }
    }
}
