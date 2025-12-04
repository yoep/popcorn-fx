use crate::torrent::dht::{Node, NodeId, NodeState};
use crate::torrent::metrics::Metric;
use itertools::Itertools;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::{Duration, Instant};
use thiserror::Error;

/// The result type alias for routing table operations.
pub type Result<T> = std::result::Result<T, Reason>;

/// The reason why a routing table operation failed.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum Reason {
    #[error("node already exists")]
    Duplicate,
    #[error("node is invalid")]
    InvalidNode,
    #[error("bucket has reached its limit")]
    LimitReached,
}

/// The bucket index type alias.
pub type BucketIndex = u8;

#[derive(Debug)]
pub struct RoutingTable {
    /// The root node ID of the routing table.
    pub id: NodeId,
    /// The buckets of the routing table.
    pub buckets: BTreeMap<BucketIndex, Bucket>,
    /// The number of nodes that can be stored within a bucket.
    /// This is the "K" value as described in BEP5.
    pub bucket_size: usize,
}

impl RoutingTable {
    /// Create a new routing table for the given root node.
    pub fn new(id: NodeId, bucket_size: usize) -> Self {
        Self {
            id,
            buckets: Default::default(),
            bucket_size,
        }
    }

    /// Returns the amount of nodes within the routing table.
    pub fn len(&self) -> usize {
        self.buckets.iter().map(|(_, bucket)| bucket.len()).sum()
    }

    /// Returns an iterator over the nodes within the routing table.
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.buckets
            .iter()
            .flat_map(|(_, bucket)| bucket.nodes.as_slice())
    }

    /// Returns an iterator over the non-empty buckets of the routing table.
    pub fn buckets(&self) -> impl Iterator<Item = &Bucket> {
        self.buckets.values().filter(|bucket| bucket.len() > 0)
    }

    /// Returns the found node within the routing table, if found.
    pub fn find_node(&self, id: &NodeId) -> Option<&Node> {
        self.nodes().find(|node| node.id() == id)
    }

    /// Returns the closest nodes slice for the given node id.
    pub fn find_bucket_nodes(&self, id: &NodeId) -> &[Node] {
        let distance = self.id.distance(id);
        self.buckets
            .get(&distance)
            .map(|b| b.nodes.as_slice())
            .unwrap_or_default()
    }

    /// Try to add the given node to the routing table.
    ///
    /// # Returns
    ///
    /// It returns the bucket id to which the node has been added, else [None].
    pub async fn add_node(&mut self, node: Node) -> Result<BucketIndex> {
        if node.id() == &self.id || !Self::is_valid(&node) {
            return Err(Reason::InvalidNode);
        }

        let distance = self.id.distance(node.id());
        // if distance == 0 {
        //     trace!("Routing table is ignoring node, node has same ID as the routing table");
        //     return None;
        // }

        let bucket = self
            .buckets
            .entry(distance)
            .or_insert_with(|| Bucket::new(self.bucket_size));
        // check if the node already exists within the bucket
        if bucket.nodes.contains(&node) {
            return Err(Reason::Duplicate);
        }

        // try to add the node within the bucket
        bucket.add(node).await.map(|_| distance)
    }

    /// Call once per tick (typically once per second), providing a tick interval.
    pub fn tick(&self, interval: Duration) {
        for (_, bucket) in &self.buckets {
            for node in &bucket.nodes {
                node.metrics().tick(interval);
            }
        }
    }

    /// Validate if the given node is valid.
    fn is_valid(node: &Node) -> bool {
        // early fail if the node address is unknown
        if match node.addr().ip() {
            IpAddr::V4(ip) => ip == Ipv4Addr::UNSPECIFIED,
            IpAddr::V6(ip) => ip == Ipv6Addr::UNSPECIFIED,
        } {
            return false;
        }

        node.id().verify_id(&node.addr().ip()) && node.addr().port() != 0
    }
}

#[derive(Debug)]
pub struct Bucket {
    /// The nodes of the bucket
    pub nodes: Vec<Node>,
    /// The last time the bucket has been updated
    pub last_changed: Instant,
    /// The maximum size of the bucket
    max_size: usize,
}

impl Bucket {
    /// Create a new bucket with the given (max) size.
    fn new(size: usize) -> Self {
        Self {
            nodes: vec![],
            last_changed: Instant::now(),
            max_size: size,
        }
    }

    /// Get the number of nodes within the bucket.
    fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Add the given node to the bucket, without exceeding the bucket size.
    async fn add(&mut self, node: Node) -> Result<()> {
        if self.nodes.len() < self.max_size {
            self.nodes.push(node);
            self.last_changed = Instant::now();
            return Ok(());
        } else {
            let nodes = futures::future::join_all(
                self.nodes
                    .iter()
                    .map(|e| async move { (e.state().await, e) })
                    .collect_vec(),
            )
            .await;
            if let Some(position) = nodes
                .into_iter()
                .filter(|(state, _)| state != &NodeState::Good)
                .sorted_by(Self::sort_nodes_by_worse_state)
                .position(|_| true)
            {
                // remove the first bad node to free up some space
                let _ = self.nodes.remove(position);
                self.nodes.push(node);
                self.last_changed = Instant::now();
                return Ok(());
            }
        }

        Err(Reason::LimitReached)
    }

    fn sort_nodes_by_worse_state(a: &(NodeState, &Node), b: &(NodeState, &Node)) -> Ordering {
        let a_state = a.0 as u8;
        let b_state = b.0 as u8;

        b_state.cmp(&a_state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_logger;
    use std::net::Ipv4Addr;
    use std::net::SocketAddr;

    mod routing_table {
        use super::*;

        #[tokio::test]
        async fn test_add_node_self() {
            init_logger!();
            let node_id = NodeId::new();
            let node = Node::new(node_id, (Ipv4Addr::LOCALHOST, 9000).into());
            let mut routing_table = RoutingTable::new(node_id, 2);

            let result = routing_table.add_node(node).await;
            assert_eq!(
                Err(Reason::InvalidNode),
                result,
                "expected the node ID of the routing table to not have been added"
            );

            let result = routing_table.len();
            assert_eq!(0, result, "expected the node to not have been stored");
        }

        #[tokio::test]
        async fn test_add_node() {
            init_logger!();
            let node_id = NodeId::new();
            let node = Node::new(node_id, (Ipv4Addr::LOCALHOST, 10000).into());
            let mut routing_table = RoutingTable::new(NodeId::new(), 10);

            let result = routing_table.add_node(node).await;
            assert!(result.is_ok(), "expected Ok, but got {:?} instead", result);

            let result = routing_table.len();
            assert_eq!(1, result, "expected the node to have been stored");
        }

        #[tokio::test]
        async fn test_add_node_invalid() {
            init_logger!();
            let node_unspecified_addr =
                Node::new(NodeId::new(), (Ipv4Addr::UNSPECIFIED, 1000).into());
            let node_unspecified_port = Node::new(NodeId::new(), (Ipv4Addr::LOCALHOST, 0).into());
            let mut routing_table = RoutingTable::new(NodeId::new(), 10);

            let result = routing_table.add_node(node_unspecified_addr).await;
            assert_eq!(
                Err(Reason::InvalidNode),
                result,
                "expected unspecified node addr to not be added"
            );

            let result = routing_table.add_node(node_unspecified_port).await;
            assert_eq!(
                Err(Reason::InvalidNode),
                result,
                "expected unspecified node port to not be added"
            );
        }
    }

    mod bucket {
        use super::*;

        #[tokio::test]
        async fn test_add_empty_bucket() {
            let node = Node::new(NodeId::new(), (Ipv4Addr::LOCALHOST, 8900).into());
            let mut bucket = Bucket::new(8);

            let result = bucket.add(node).await;

            assert!(result.is_ok(), "expected Ok, but got {:?} instead", result);
        }

        #[tokio::test]
        async fn test_add_bucket_full() {
            let node = Node::new(NodeId::new(), (Ipv4Addr::LOCALHOST, 8900).into());
            let mut bucket = Bucket::new(2);
            bucket.nodes.push(create_node_with_state(
                ([198, 168, 0, 1], 8900).into(),
                NodeState::Good,
            ));
            bucket.nodes.push(create_node_with_state(
                ([198, 168, 0, 2], 8900).into(),
                NodeState::Good,
            ));

            let result = bucket.add(node).await;

            assert_eq!(
                Err(Reason::LimitReached),
                result,
                "expected the node to not have been added"
            );
        }

        #[tokio::test]
        async fn test_add_bucket_full_with_bad_node() {
            let node = Node::new(NodeId::new(), (Ipv4Addr::LOCALHOST, 8900).into());
            let mut bucket = Bucket::new(2);
            bucket.nodes.push(create_node_with_state(
                ([198, 168, 0, 1], 8900).into(),
                NodeState::Bad,
            ));
            bucket.nodes.push(create_node_with_state(
                ([198, 168, 0, 2], 8900).into(),
                NodeState::Good,
            ));

            let result = bucket.add(node).await;

            assert!(result.is_ok(), "expected Ok, but got {:?} instead", result);
        }
    }

    fn create_node_with_state(addr: SocketAddr, state: NodeState) -> Node {
        Node::new_with_state(NodeId::new(), addr, state)
    }
}
