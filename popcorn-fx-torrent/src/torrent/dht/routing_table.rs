use crate::torrent::dht::{Node, NodeId, NodeState};
use itertools::Itertools;
use log::{debug, trace};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Instant;

#[derive(Debug)]
pub struct RoutingTable {
    /// The root node ID of the routing table.
    pub id: NodeId,
    /// The buckets of the routing table.
    pub buckets: BTreeMap<u8, Bucket>,
    /// The number of nodes that can be stored within a bucket.
    /// This is the "K" value as described in BEP5.
    pub bucket_size: usize,
    /// The router nodes of the routing table used only for searching.
    router_nodes: Vec<Node>,
}

impl RoutingTable {
    /// Create a new routing table for the given root node.
    pub fn new(id: NodeId, bucket_size: usize, router_nodes: Vec<Node>) -> Self {
        Self {
            id,
            buckets: Default::default(),
            bucket_size,
            router_nodes,
        }
    }

    /// Get the number of nodes within the routing table.
    pub fn len(&self) -> usize {
        self.buckets.iter().map(|(_, bucket)| bucket.len()).sum()
    }

    /// Get all bucket nodes within the routing table.
    /// Does not include any router nodes.
    pub fn nodes(&self) -> Vec<Node> {
        self.buckets
            .iter()
            .flat_map(|(_, bucket)| bucket.nodes.iter().cloned())
            .collect()
    }

    /// Get all nodes used during search operations.
    /// Includes both bucket and router nodes.
    pub fn search_nodes(&self) -> Vec<Node> {
        let mut nodes = self.nodes();
        nodes.extend(self.router_nodes.iter().cloned());
        nodes
    }

    /// Get all router nodes in the routing table.
    /// These nodes are only used for search and should not appear in responses.
    pub fn router_nodes(&self) -> &[Node] {
        &self.router_nodes
    }

    /// Try to find the node within the routing table for the given ID.
    pub fn find_node(&self, id: &NodeId) -> Option<&Node> {
        let distance = self.id.distance(id);
        self.buckets
            .get(&distance)
            .and_then(|bucket| bucket.nodes.iter().find(|node| &node.id == id))
    }

    /// Try to find the node within the routing table for the given ID.
    /// It returns a mutable reference to the stored node when found.
    pub fn find_node_mut(&mut self, id: &NodeId) -> Option<&mut Node> {
        let distance = self.id.distance(id);
        self.buckets
            .get_mut(&distance)
            .and_then(|bucket| bucket.nodes.iter_mut().find(|node| &node.id == id))
    }

    /// Try to find the node within the routing table for the given address.
    /// It returns a mutable reference to the stored node when found.
    pub fn find_node_by_addr_mut(&mut self, addr: &SocketAddr) -> Option<&mut Node> {
        self.buckets
            .iter_mut()
            .find_map(|(_, bucket)| bucket.find_by_addr_mut(addr))
    }

    /// Try to find all nodes within the bucket of the given target node ID.
    pub fn find_bucket_nodes(&self, id: &NodeId) -> &[Node] {
        let distance = self.id.distance(id);
        self.buckets
            .get(&distance)
            .map(|b| b.nodes.as_slice())
            .unwrap_or(Default::default())
    }

    /// Try to add the given node to the routing table.
    ///
    /// # Returns
    ///
    /// It returns the bucket id to which the node has been added, else [None].
    pub fn add_node(&mut self, node: Node) -> Option<u8> {
        if !Self::is_valid(&node) {
            debug!(
                "Routing table is ignoring node {}, node is invalid",
                node.addr
            );
            return None;
        }

        let distance = self.id.distance(&node.id);
        if distance == 0 {
            trace!("Routing table is ignoring node, node has same ID as the routing table");
            return None;
        }

        let bucket = self
            .buckets
            .entry(distance)
            .or_insert_with(|| Bucket::new(self.bucket_size));
        // check if the node already exists within the bucket
        if bucket.nodes.contains(&node) {
            return None;
        }

        // try to add the node within the bucket
        if bucket.add(node) {
            Some(distance)
        } else {
            None
        }
    }

    /// Add the given router node to the routing table.
    /// These will only be used during searches, but never returned in a response.
    ///
    /// # Returns
    ///
    /// Returns `true` when the router node has been added, else `false`.
    pub fn add_router_node(&mut self, node: Node) -> bool {
        // check if the router node is already known
        if self.contains_router_node(&node.addr) {
            return false;
        }

        self.router_nodes.push(node);
        true
    }

    /// Remove the router node to the routing table.
    ///
    /// # Returns
    ///
    /// Returns `true` when the router node has been removed, else `false`.
    pub fn remove_router_node(&mut self, node: &Node) -> bool {
        if let Some(position) = self.router_nodes.iter().position(|e| e.id == node.id) {
            self.router_nodes.remove(position);
            return true;
        }

        false
    }

    /// Refresh all buckets within the routing table.
    pub async fn refresh(&mut self) {
        trace!(
            "DHT routing table is updating {} buckets",
            self.buckets.len()
        );
        for (_, bucket) in self.buckets.iter_mut() {
            bucket.refresh().await;
        }
    }

    /// Check if the given address is already registered as a router node.
    fn contains_router_node(&self, addr: &SocketAddr) -> bool {
        self.router_nodes.iter().find(|e| e.addr == *addr).is_some()
    }

    /// Validate if the given node is valid.
    fn is_valid(node: &Node) -> bool {
        let addr = &node.addr;
        let is_unspecified = match addr.ip() {
            IpAddr::V4(ip) => ip == Ipv4Addr::UNSPECIFIED,
            IpAddr::V6(ip) => ip == Ipv6Addr::UNSPECIFIED,
        };

        !is_unspecified && addr.port() != 0
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
    fn add(&mut self, node: Node) -> bool {
        if self.nodes.iter().any(|n| n.id == node.id) {
            return false;
        }

        if self.nodes.len() < self.max_size {
            self.nodes.push(node);
            self.last_changed = Instant::now();
            return true;
        } else {
            if let Some(position) = self
                .nodes
                .iter()
                .filter(|e| e.state != NodeState::Good)
                .sorted_by(Self::sort_nodes_by_worse_state)
                .position(|_| true)
            {
                // remove the first bad node to free up some space
                let _ = self.nodes.remove(position);
                self.nodes.push(node);
                self.last_changed = Instant::now();
                return true;
            }
        }

        false
    }

    /// Try to find a node by the given address.
    fn find_by_addr_mut(&mut self, addr: &SocketAddr) -> Option<&mut Node> {
        self.nodes.iter_mut().find(|e| e.addr == *addr)
    }

    /// Refresh the nodes within this bucket.
    async fn refresh(&mut self) {
        // TODO
    }

    fn sort_nodes_by_worse_state(a: &&Node, b: &&Node) -> Ordering {
        let a_state = a.state as u8;
        let b_state = b.state as u8;

        b_state.cmp(&a_state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_logger;
    use std::net::SocketAddr;

    mod routing_table {
        use super::*;

        #[test]
        fn test_add_node_self() {
            init_logger!();
            let node_id = NodeId::new();
            let node = Node::new(node_id, (Ipv4Addr::LOCALHOST, 9000).into());
            let mut routing_table = RoutingTable::new(node_id, 2, Vec::with_capacity(0));

            let result = routing_table.add_node(node);
            assert_eq!(
                None, result,
                "expected the node ID of the routing table to not have been added"
            );

            let result = routing_table.len();
            assert_eq!(0, result, "expected the node to not have been stored");
        }

        #[test]
        fn test_add_node() {
            init_logger!();
            let node_id = NodeId::new();
            let node = Node::new(node_id, (Ipv4Addr::LOCALHOST, 10000).into());
            let mut routing_table = RoutingTable::new(NodeId::new(), 10, Vec::with_capacity(0));

            let result = routing_table.add_node(node);
            assert_ne!(None, result, "expected the node to have been added");

            let result = routing_table.len();
            assert_eq!(1, result, "expected the node to have been stored");
        }

        #[test]
        fn test_add_node_invalid() {
            init_logger!();
            let node_unspecified_addr =
                Node::new(NodeId::new(), (Ipv4Addr::UNSPECIFIED, 1000).into());
            let node_unspecified_port = Node::new(NodeId::new(), (Ipv4Addr::LOCALHOST, 0).into());
            let mut routing_table = RoutingTable::new(NodeId::new(), 10, Vec::with_capacity(0));

            let result = routing_table.add_node(node_unspecified_addr);
            assert_eq!(
                None, result,
                "expected unspecified node addr to not be added"
            );

            let result = routing_table.add_node(node_unspecified_port);
            assert_eq!(
                None, result,
                "expected unspecified node port to not be added"
            );
        }
    }

    mod bucket {
        use super::*;

        #[test]
        fn test_add_empty_bucket() {
            let node = Node::new(NodeId::new(), (Ipv4Addr::LOCALHOST, 8900).into());
            let mut bucket = Bucket::new(8);

            let result = bucket.add(node);

            assert_eq!(true, result, "expected the node to have been added");
        }

        #[test]
        fn test_add_bucket_full() {
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

            let result = bucket.add(node);

            assert_eq!(false, result, "expected the node tto not have been added");
        }

        #[test]
        fn test_add_bucket_full_with_bad_node() {
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

            let result = bucket.add(node);

            assert_eq!(true, result, "expected the node to have been added");
        }
    }

    fn create_node_with_state(addr: SocketAddr, state: NodeState) -> Node {
        let mut node = Node::new(NodeId::new(), addr);
        node.state = state;
        node
    }
}
