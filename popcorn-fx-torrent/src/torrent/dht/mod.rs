pub use errors::*;
pub use node::*;
pub use node_id::*;
pub use server::*;

mod compact;
mod errors;
mod krpc;
mod node;
mod node_id;
mod routing_table;
mod server;

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a new DHT node server pair.
    pub async fn create_node_server_pair() -> (NodeServer, NodeServer) {
        let node1 = NodeServer::builder().build().await.unwrap();
        let node2 = NodeServer::builder().build().await.unwrap();

        (node1, node2)
    }
}
