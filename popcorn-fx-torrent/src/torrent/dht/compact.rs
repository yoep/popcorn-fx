use crate::torrent::dht::{Error, Node, NodeId, Result};
use crate::torrent::{CompactIpv4Addr, CompactIpv6Addr};
use log::{debug, warn};
use std::fmt::Display;

const IPV4_NODE_SIZE: usize = 26;
const IPV6_NODE_SIZE: usize = 38;

/// A list of compact IPv4 nodes.
pub struct CompactIPv4Nodes(Vec<CompactIPv4Node>);

impl CompactIPv4Nodes {
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

impl From<&CompactIPv4Nodes> for Vec<u8> {
    fn from(value: &CompactIPv4Nodes) -> Vec<u8> {
        let mut buffer = vec![0u8; IPV4_NODE_SIZE * value.0.len()];

        for (i, node) in value.0.iter().enumerate() {
            let start = i * IPV4_NODE_SIZE;
            let end = start + IPV4_NODE_SIZE;
            let bytes: [u8; IPV4_NODE_SIZE] = node.into();

            buffer[start..end].clone_from_slice(&bytes);
        }

        buffer
    }
}

impl From<&[Node]> for CompactIPv4Nodes {
    fn from(nodes: &[Node]) -> Self {
        let mut compact_nodes = vec![];

        for node in nodes {
            match CompactIPv4Node::try_from(node) {
                Ok(e) => compact_nodes.push(e),
                Err(e) => debug!("Failed to parse compact IPv4 node, {}", e),
            }
        }

        Self(compact_nodes)
    }
}

impl Display for CompactIPv4Nodes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes: Vec<u8> = self.into();
        write!(f, "{}", String::from_utf8_lossy(&bytes))
    }
}

/// The compact representation of a node with an IPv4 address.
#[derive(Debug, Clone, PartialEq)]
pub struct CompactIPv4Node {
    pub addr: CompactIpv4Addr,
    pub id: NodeId,
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

        let addr =
            CompactIpv4Addr::try_from(&bytes[..6]).map_err(|e| Error::Parse(e.to_string()))?;
        let id = NodeId::try_from(&bytes[6..]).map_err(|e| Error::Parse(e.to_string()))?;

        Ok(Self { addr, id })
    }
}

impl From<&CompactIPv4Node> for [u8; IPV4_NODE_SIZE] {
    fn from(value: &CompactIPv4Node) -> [u8; IPV4_NODE_SIZE] {
        let mut buffer = [0u8; IPV4_NODE_SIZE];
        let addr: [u8; 6] = (&value.addr).into();

        buffer[0..6].copy_from_slice(&addr);
        buffer[6..].copy_from_slice(value.id.as_node_slice());

        buffer
    }
}

impl TryFrom<&Node> for CompactIPv4Node {
    type Error = Error;

    fn try_from(value: &Node) -> Result<Self> {
        Ok(Self {
            addr: value.addr.try_into().map_err(|_| Error::InvalidAddr)?,
            id: value.id,
        })
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
    pub addr: CompactIpv6Addr,
    pub id: NodeId,
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

        let addr =
            CompactIpv6Addr::try_from(&bytes[..18]).map_err(|e| Error::Parse(e.to_string()))?;
        let id = NodeId::try_from(&bytes[18..]).map_err(|e| Error::Parse(e.to_string()))?;

        Ok(Self { addr, id })
    }
}

impl From<&CompactIPv6Node> for [u8; IPV6_NODE_SIZE] {
    fn from(value: &CompactIPv6Node) -> [u8; IPV6_NODE_SIZE] {
        let mut buffer = [0u8; IPV6_NODE_SIZE];
        let addr: [u8; 18] = (&value.addr).into();

        buffer[0..6].copy_from_slice(&addr);
        buffer[6..].copy_from_slice(value.id.as_node_slice());

        buffer
    }
}

/// A list of compact IPv6 nodes.
pub struct CompactIPv6Nodes(Vec<CompactIPv6Node>);

impl CompactIPv6Nodes {
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

impl From<&CompactIPv6Nodes> for Vec<u8> {
    fn from(value: &CompactIPv6Nodes) -> Vec<u8> {
        let mut buffer = vec![0u8; IPV6_NODE_SIZE * value.0.len()];

        for (i, node) in value.0.iter().enumerate() {
            let start = i * IPV6_NODE_SIZE;
            let end = start + IPV6_NODE_SIZE;
            let bytes: [u8; IPV6_NODE_SIZE] = node.into();

            buffer[start..end].clone_from_slice(&bytes);
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

        #[test]
        fn test_try_from_compact_nodes() {
            let nodes = CompactIPv4Nodes(vec![
                CompactIPv4Node {
                    addr: CompactIpv4Addr {
                        ip: [127, 0, 0, 1].into(),
                        port: 5000,
                    },
                    id: NodeId::new(),
                },
                CompactIPv4Node {
                    addr: CompactIpv4Addr {
                        ip: [127, 0, 0, 2].into(),
                        port: 6080,
                    },
                    id: NodeId::new(),
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
    }
}
