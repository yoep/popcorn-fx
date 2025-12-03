pub use errors::*;
pub use metrics::*;
pub use node::*;
pub use node_id::*;
pub use tracker::*;

mod compact;
mod errors;
mod krpc;
mod metrics;
mod node;
mod node_id;
mod observer;
mod peers;
mod routing_table;
mod tracker;
mod traversal;

const DEFAULT_ROUTING_NODE_SERVERS: fn() -> Vec<&'static str> = || {
    vec![
        "router.utorrent.com:6881",
        "router.bittorrent.com:6881",
        "dht.transmissionbt.com:6881",
        "dht.aelitis.com:6881",     // Vuze
        "dht.libtorrent.org:25401", // @arvidn's
        "dht.anacrolix.link:42069",
        "router.bittorrent.cloud:42069",
    ]
};

#[cfg(test)]
mod tests {
    /// Create a new DHT tracker server pair.
    #[macro_export]
    macro_rules! create_node_server_pair {
        () => {{
            use crate::create_node_server_pair;
            use crate::torrent::dht::node_id::NodeId;

            create_node_server_pair!(NodeId::new(), NodeId::new())
        }};
        ($node_id1:expr, $node_id2:expr) => {{
            use crate::torrent::dht::DhtTracker;

            let left_node = DhtTracker::builder()
                .node_id($node_id1)
                .build()
                .await
                .unwrap();
            let right_node = DhtTracker::builder()
                .node_id($node_id2)
                .build()
                .await
                .unwrap();

            (left_node, right_node)
        }};
    }
}
