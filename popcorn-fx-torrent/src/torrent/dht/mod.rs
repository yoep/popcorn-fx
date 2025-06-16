pub use errors::*;
pub use node::*;
pub use node_id::*;
pub use tracker::*;

mod compact;
mod errors;
mod krpc;
mod node;
mod node_id;
mod routing_table;
mod tracker;

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a new DHT node server pair.
    pub async fn create_node_server_pair() -> (DhtTracker, DhtTracker) {
        let node1 = DhtTracker::builder().build().await.unwrap();
        let node2 = DhtTracker::builder().build().await.unwrap();

        (node1, node2)
    }
}
