use crate::torrent::dht::{Node, NodeId, NodeState};
use itertools::Itertools;
use log::trace;
use std::cmp::Ordering;
use std::collections::BTreeMap;
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

    /// Get the number of nodes within the routing table.
    pub fn len(&self) -> usize {
        self.buckets.iter().map(|(_, bucket)| bucket.len()).sum()
    }

    /// Try to find the node within the routing table for the given ID.
    pub fn find_node(&self, id: &NodeId) -> Option<&Node> {
        let distance = self.id.distance(id);
        self.buckets
            .get(&distance)
            .and_then(|bucket| bucket.nodes.iter().find(|node| &node.id == id))
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
    pub fn add_node(&mut self, node: Node) -> bool {
        let distance = self.id.distance(&node.id);
        if distance == 0 {
            trace!("Routing table is ignoring node, node has same ID as the routing table");
            return false;
        }

        let bucket = self
            .buckets
            .entry(distance)
            .or_insert_with(|| Bucket::new(self.bucket_size));
        // check if the node already exists within the bucket
        if bucket.nodes.contains(&node) {
            return false;
        }

        // try to add the node within the bucket
        bucket.add(node)
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
            let node = Node::new(node_id, ([127, 0, 0, 1], 9000).into());
            let mut routing_table = RoutingTable::new(node_id, 2);

            let result = routing_table.add_node(node);

            assert_eq!(
                false, result,
                "expected the node ID of the routing table to not have been added"
            )
        }
    }

    mod bucket {
        use super::*;

        #[test]
        fn test_add_empty_bucket() {
            let node = Node::new(NodeId::new(), ([127, 0, 0, 1], 8900).into());
            let mut bucket = Bucket::new(8);

            let result = bucket.add(node);

            assert_eq!(true, result, "expected the node to have been added");
        }

        #[test]
        fn test_add_bucket_full() {
            let node = Node::new(NodeId::new(), ([127, 0, 0, 1], 8900).into());
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
            let node = Node::new(NodeId::new(), ([127, 0, 0, 1], 8900).into());
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
