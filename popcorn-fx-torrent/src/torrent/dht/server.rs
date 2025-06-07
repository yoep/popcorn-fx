use crate::torrent::dht::compact::CompactIPv4Nodes;
use crate::torrent::dht::krpc::{
    FindNodeRequest, FindNodeResponse, Message, MessagePayload, PingMessage, QueryMessage,
    ResponseMessage,
};
use crate::torrent::dht::routing_table::RoutingTable;
use crate::torrent::dht::{Error, Node, NodeId, Result};
use derive_more::Display;
use log::{debug, info, trace, warn};
use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot::{Receiver, Sender};
use tokio::sync::{oneshot, Mutex, MutexGuard};
use tokio::time::{interval, timeout};
use tokio::{select, time};
use tokio_util::sync::CancellationToken;

/// The maximum size of a single UDP packet.
const MAX_PACKET_SIZE: usize = 65_535;
const REFRESH_INTERVAL_MINUTES: u64 = 15;
const RESPONSE_TIMEOUT_AFTER_SECS: u64 = 8;
pub const DEFAULT_BOOTSTRAP_SERVERS: fn() -> Vec<&'static str> = || {
    vec![
        "router.utorrent.com:6881",
        "router.bittorrent.com:6881",
        "dht.transmissionbt.com:6881",
        "dht.aelitis.com:6881",     // Vuze
        "router.silotis.us:6881",   // IPv6
        "dht.libtorrent.org:25401", // @arvidn's
        "dht.anacrolix.link:42069",
        "router.bittorrent.cloud:42069",
    ]
};

/// A server instance of a DHT node.
/// This instance can be shared between torrent by using [NodeServer::clone].
#[derive(Debug, Clone)]
pub struct NodeServer {
    inner: Arc<InnerServer>,
}

impl NodeServer {
    /// Create a new builder instance to create a new node server.
    pub fn builder() -> NodeServerBuilder {
        NodeServerBuilder::default()
    }

    /// Create a new DHT node server with the given node ID.
    /// This function allows creating a server with a specific node id.
    ///
    /// # Arguments
    ///
    /// * `id` - The node ID of the DHT server.
    /// * `bootstrap_nodes` - The initial nodes to bootstrap the DHT server from.
    pub async fn new(id: NodeId, bootstrap_nodes: Vec<SocketAddr>) -> Result<Self> {
        let socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
        let socket_addr = socket.local_addr()?;
        let (sender, receiver) = unbounded_channel();
        let cancellation_token = CancellationToken::new();
        let reader = NodeReader {
            socket: socket.clone(),
            socket_addr,
            sender,
            cancellation_token: cancellation_token.clone(),
        };
        let inner = Arc::new(InnerServer {
            id,
            transaction_id: Default::default(),
            socket,
            socket_addr,
            routing_table: Mutex::new(RoutingTable::new(id, 8)),
            pending_requests: Default::default(),
            bootstrap_nodes: Mutex::new(bootstrap_nodes),
            timout: Duration::from_secs(6),
            cancellation_token,
        });

        // start the reader in a separate thread
        tokio::spawn(async move {
            reader.start().await;
        });

        let inner_main = inner.clone();
        tokio::spawn(async move {
            inner_main.start(&inner_main, receiver).await;
        });

        Ok(Self { inner })
    }

    /// Get the ID of the DHT server.
    pub fn id(&self) -> NodeId {
        self.inner.id
    }

    /// Get the socket address on which this DHT server is running.
    pub fn addr(&self) -> SocketAddr {
        self.inner.socket_addr
    }

    /// Get the port on which the DHT server is running.
    pub fn port(&self) -> u16 {
        self.inner.socket_addr.port()
    }

    /// Try to ping the given node address.
    /// This functions waits for a response from the node, so it might be recommended to wrap this fn call in a timeout.
    pub async fn ping(&self, addr: SocketAddr) -> Result<()> {
        let rx = self.inner.ping(&addr).await;
        rx.await.map_err(|_| Error::Closed)??;
        Ok(())
    }

    /// Close the DHT node server.
    pub fn close(&self) {
        self.inner.close()
    }
}

impl Drop for NodeServer {
    fn drop(&mut self) {
        // check if only the main loop remains
        if Arc::strong_count(&self.inner) == 2 {
            self.close()
        }
    }
}

#[derive(Debug, Default)]
pub struct NodeServerBuilder {
    node_id: Option<NodeId>,
    bootstrap_nodes: Option<Vec<SocketAddr>>,
}

impl NodeServerBuilder {
    /// Set the ID of the node server.
    pub fn node_id(&mut self, id: NodeId) -> &mut Self {
        self.node_id = Some(id);
        self
    }

    /// Add the given address to the bootstrap nodes.
    pub fn bootstrap_node(&mut self, addr: SocketAddr) -> &mut Self {
        self.bootstrap_nodes.get_or_insert_default().push(addr);
        self
    }

    /// Set the initial bootstrap nodes to use.
    /// This replaces any already existing configured bootstrap nodes.
    pub fn bootstrap_nodes(&mut self, nodes: Vec<SocketAddr>) -> &mut Self {
        self.bootstrap_nodes = Some(nodes);
        self
    }

    /// Try to create a new DHT node server from this builder.
    pub async fn build(&mut self) -> Result<NodeServer> {
        let node_id = self.node_id.take().unwrap_or_else(|| NodeId::new());
        let bootstrap_server = self.bootstrap_nodes.take().unwrap_or_default();

        NodeServer::new(node_id, bootstrap_server).await
    }
}

#[derive(Debug, Display)]
#[display(fmt = "DHT node server [{}]", socket_addr)]
pub struct InnerServer {
    /// The unique ID of the server
    id: NodeId,
    /// The current transaction ID of the node server
    transaction_id: AtomicU16,
    /// The underlying socket used by the server
    socket: Arc<UdpSocket>,
    /// The address on which the server is listening
    socket_addr: SocketAddr,
    /// The routing table of the node server
    routing_table: Mutex<RoutingTable>,
    /// The currently pending requests of the server
    pending_requests: Mutex<HashMap<TransactionKey, PendingRequest>>,
    /// The bootstrap nodes for the server to initialize nodes
    bootstrap_nodes: Mutex<Vec<SocketAddr>>,
    /// The timeout duration of packages
    timout: Duration,
    /// The cancellation token of the server
    cancellation_token: CancellationToken,
}

impl InnerServer {
    async fn start(
        &self,
        inner: &Arc<Self>,
        mut receiver: UnboundedReceiver<(Message, SocketAddr)>,
    ) {
        if self.should_bootstrap().await {
            let inner = inner.clone();
            tokio::spawn(async move {
                inner.bootstrap().await;
            });
        }

        let mut refresh_interval = interval(Duration::from_secs(60 * REFRESH_INTERVAL_MINUTES));
        let mut cleanup_interval = interval(Duration::from_secs(2));

        debug!("{} started", self);
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some((message, addr)) = receiver.recv() => {
                    if let Err(e) = self.handle_incoming_message(message, addr).await {
                        warn!("DHT node server failed to process incoming message, {}", e);
                    }
                },
                _ = refresh_interval.tick() => self.refresh_routing_table().await,
                _ = cleanup_interval.tick() => self.cleanup_pending_requests().await,
            }
        }
        debug!("{} main loop ended", self);
    }

    /// Try to process an incoming DHT message from the given node address.
    async fn handle_incoming_message(&self, message: Message, addr: SocketAddr) -> Result<()> {
        trace!("{} received message from {}, {:?}", self, addr, message);
        let id = message.transaction_id();
        let key = TransactionKey { id, addr };

        // check the type of the message
        match &message.payload {
            MessagePayload::Query(query) => match query {
                QueryMessage::Ping { request } => {
                    self.send_response(
                        &message,
                        ResponseMessage::Ping {
                            response: PingMessage { id: self.id },
                        },
                        &addr,
                    )
                    .await?;

                    // add the node in case we don't know it yet
                    self.add_node(Node::new(request.id, addr)).await;
                }
                QueryMessage::FindNode { request } => {
                    let routing_table = self.routing_table.lock().await;
                    let target_node = request.target;
                    let nodes = routing_table.find_bucket_nodes(&target_node);
                    let compact_nodes = CompactIPv4Nodes::from(nodes);

                    self.send_response(
                        &message,
                        ResponseMessage::FindNode {
                            response: FindNodeResponse {
                                id: self.id,
                                nodes: (&compact_nodes).into(),
                            },
                        },
                        &addr,
                    )
                    .await?;
                }
                QueryMessage::GetPeers => {}
                QueryMessage::AnnouncePeer => {}
            },
            MessagePayload::Response(response) => {
                if let Some(pending_request) = self.pending_requests.lock().await.remove(&key) {
                    debug!(
                        "{} received response \"{}\" from {} for {}",
                        self,
                        response.name(),
                        addr,
                        key
                    );
                    let reply: Result<Reply>;

                    match response {
                        ResponseMessage::FindNode { response } => {
                            trace!("{} parsing compact nodes", self);
                            let nodes = CompactIPv4Nodes::try_from(response.nodes.as_slice()).map(
                                |nodes| {
                                    nodes
                                        .as_slice()
                                        .iter()
                                        .map(|node| Node::new(node.id, node.addr.clone().into()))
                                        .collect::<Vec<Node>>()
                                },
                            );

                            reply = match nodes {
                                Ok(nodes) => {
                                    debug!("{} discovered a total of {} nodes", self, nodes.len());
                                    self.add_nodes(nodes.as_slice()).await;
                                    Ok(Reply::FindNode(nodes))
                                }
                                Err(e) => Err(e),
                            }
                        }
                        ResponseMessage::Ping { response } => {
                            self.add_node(Node::new(response.id, addr)).await;
                            reply = Ok(Reply::Ping);
                        }
                    }

                    Self::send_reply(pending_request.request_type, reply)
                } else {
                    warn!(
                        "{} received response for unknown request, invalid transaction {}",
                        self, key
                    );
                }
            }
            MessagePayload::Error(err) => {
                if let Some(pending_request) = self.pending_requests.lock().await.remove(&key) {
                    debug!("{} received error for {}", self, key);
                    Self::send_reply(pending_request.request_type, Err(Error::from(err)))
                } else {
                    warn!(
                        "{} received error for unknown request, invalid transaction {}",
                        self, key
                    );
                }
            }
        }

        Ok(())
    }

    /// Ping the given node address.
    ///
    /// # Arguments
    ///
    /// * `addr` - the node address to ping.
    ///
    /// # Returns
    ///
    /// Returns the outgoing ping result with a receiver that completes when a response has been received.
    async fn ping(&self, addr: &SocketAddr) -> Receiver<Result<()>> {
        let (tx, rx) = oneshot::channel();

        self.send_query(
            QueryMessage::Ping {
                request: PingMessage { id: self.id },
            },
            addr,
            PendingRequestType::Ping(tx),
        )
        .await;

        rx
    }

    /// Find the closest nodes to the given target node id.
    async fn find_node(&self, target_id: NodeId, addr: &SocketAddr) -> Receiver<Result<Vec<Node>>> {
        let (tx, rx) = oneshot::channel();

        self.send_query(
            QueryMessage::FindNode {
                request: FindNodeRequest {
                    id: self.id,
                    target: target_id,
                },
            },
            addr,
            PendingRequestType::FindNode(tx),
        )
        .await;

        rx
    }

    /// Send a new query to the given node address.
    ///
    /// # Arguments
    ///
    /// * `query` - The query to send to the node.
    /// * `addr` - The node address.
    ///
    /// # Returns
    ///
    /// It returns an error when a failure occurred while sending the packet.
    async fn send_query(&self, query: QueryMessage, addr: &SocketAddr, tx: PendingRequestType) {
        // validate the remote node address
        if addr.ip().is_unspecified() || addr.port() == 0 {
            Self::send_error(tx, Error::InvalidAddr);
            return;
        }

        let name = query.name().to_string();
        let id = self.next_transaction_id();
        let message = match Message::builder()
            .transaction_id(id)
            .payload(MessagePayload::Query(query))
            .build()
        {
            Ok(message) => message,
            Err(err) => {
                Self::send_error(tx, err);
                return;
            }
        };

        debug!("{} is sending query \"{}\" to {}", self, name, addr);
        match self.send(message, addr).await {
            Ok(_) => {
                self.pending_requests.lock().await.insert(
                    TransactionKey { id, addr: *addr },
                    PendingRequest {
                        request_type: tx,
                        timestamp_sent: Instant::now(),
                    },
                );
            }
            Err(err) => {
                Self::send_error(tx, err);
            }
        }
    }

    /// Send the given response for a query message.
    ///
    /// # Arguments
    ///
    /// * `message` - The original query message.
    /// * `response` - The response payload.
    /// * `addr` - The node address to send the response to.
    ///
    /// # Returns
    ///
    /// It returns an error if the response failed to send.
    async fn send_response(
        &self,
        message: &Message,
        response: ResponseMessage,
        addr: &SocketAddr,
    ) -> Result<()> {
        let message = Message::builder()
            .transaction_id(message.transaction_id())
            .payload(MessagePayload::Response(response))
            .build()?;

        self.send(message, addr).await
    }

    async fn send(&self, message: Message, addr: &SocketAddr) -> Result<()> {
        if self.cancellation_token.is_cancelled() {
            return Err(Error::Closed);
        }

        let bytes = serde_bencode::to_bytes(&message)?;

        trace!(
            "{} is sending message ({} bytes) to {}, {:?}",
            self,
            bytes.len(),
            addr,
            message
        );
        let start_time = Instant::now();
        select! {
            _ = time::sleep(self.timout) => Err(Error::Io(io::Error::new(io::ErrorKind::TimedOut, format!("connection to {} has timed out", addr)))),
            result = self.socket.send_to(bytes.as_slice(), addr) => result.map_err(|e| Error::from(e)),
        }?;
        let elapsed = start_time.elapsed();
        trace!(
            "{} sent {} bytes to {} in {}.{:03}ms",
            self,
            bytes.len(),
            addr,
            elapsed.as_millis(),
            elapsed.as_micros()
        );

        Ok(())
    }

    /// Bootstrap the node server by querying the initial node addresses.
    async fn bootstrap(&self) {
        let bootstrap_nodes = self.bootstrap_nodes.lock().await;
        let mut futures = vec![];

        // iterate over the initial bootstrap nodes
        debug!("{} is bootstrapping {} nodes", self, bootstrap_nodes.len());
        for addr in bootstrap_nodes.iter() {
            futures.push(async move {
                match timeout(Duration::from_secs(3), self.ping(addr).await).await {
                    Ok(_) => {
                        timeout(Duration::from_secs(2), self.find_node(self.id, addr).await).await
                    }
                    Err(e) => Err(e),
                }
            });
        }

        let total_bootstrapped_nodes = futures::future::join_all(futures)
            .await
            .into_iter()
            .filter(|result| result.is_ok())
            .count();
        info!("{} bootstrapped {} nodes", self, total_bootstrapped_nodes);
    }

    /// Try to add the given nodes to the routing table.
    async fn add_nodes(&self, nodes: &[Node]) {
        let mut routing_table = self.routing_table.lock().await;
        for node in nodes {
            self.add_node_with_lock(node.clone(), &mut routing_table);
        }
    }

    /// Try to add the given node to the routing table.
    async fn add_node(&self, node: Node) {
        let mut routing_table = self.routing_table.lock().await;
        self.add_node_with_lock(node, &mut routing_table);
    }

    fn add_node_with_lock(&self, node: Node, routing_table: &mut MutexGuard<'_, RoutingTable>) {
        let addr = node.addr;
        if routing_table.add_node(node) {
            debug!("{} added new node {}", self, addr);
        }
    }

    /// Refresh the nodes within the routing table.
    async fn refresh_routing_table(&self) {
        let mut routing_table = self.routing_table.lock().await;
        trace!("{} is refreshing nodes within routing table", self);
        routing_table.refresh().await;
    }

    /// Cleanup pending requests which have not received a response.
    async fn cleanup_pending_requests(&self) {
        let mut pending_requests = self.pending_requests.lock().await;
        let now = Instant::now();
        let timed_out_request_keys: Vec<_> = pending_requests
            .iter()
            .filter(|(_, request)| {
                now - request.timestamp_sent >= Duration::from_secs(RESPONSE_TIMEOUT_AFTER_SECS)
            })
            .map(|(key, _)| key.clone())
            .collect();

        if timed_out_request_keys.is_empty() {
            return;
        }

        trace!(
            "{} is cleaning a total of {} timed-out requests",
            self,
            timed_out_request_keys.len()
        );
        for key in timed_out_request_keys {
            if let Some(request) = pending_requests.remove(&key) {
                Self::send_error(request.request_type, Error::Timeout);
            }
        }
    }

    /// Check if the node server needs to be bootstrapped.
    async fn should_bootstrap(&self) -> bool {
        let bootstrap_nodes = self.bootstrap_nodes.lock().await;
        let routing_table = self.routing_table.lock().await;

        routing_table.len() == 0 && bootstrap_nodes.len() > 0
    }

    /// Get the next transaction ID for sending a new message.
    /// The transaction ID within the server will be automatically wrapped when [u16::MAX] has been reached.
    fn next_transaction_id(&self) -> u16 {
        let mut old = self.transaction_id.load(Ordering::Relaxed);
        loop {
            let new = old.wrapping_add(1);
            match self.transaction_id.compare_exchange_weak(
                old,
                new,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(prev) => return prev,
                Err(current) => old = current,
            }
        }
    }

    fn close(&self) {
        if self.cancellation_token.is_cancelled() {
            return;
        }

        trace!("{} is closing", self);
        self.cancellation_token.cancel();
    }

    fn send_reply(response: PendingRequestType, result: Result<Reply>) {
        match response {
            PendingRequestType::Ping(tx) => {
                let _ = tx.send(result.map(|_| ()));
            }
            PendingRequestType::FindNode(tx) => {
                let _ = tx.send(result.and_then(|r| {
                    if let Reply::FindNode(nodes) = r {
                        Ok(nodes)
                    } else {
                        Err(Error::InvalidMessage(format!(
                            "expected Reply::FindNode, but got {:?} instead",
                            r
                        )))
                    }
                }));
            }
        };
    }

    fn send_error(response: PendingRequestType, err: Error) {
        match response {
            PendingRequestType::Ping(tx) => {
                let _ = tx.send(Err(err));
            }
            PendingRequestType::FindNode(tx) => {
                let _ = tx.send(Err(err));
            }
        }
    }
}

#[derive(Debug, Display)]
#[display(fmt = "DHT node reader [{}]", socket_addr)]
struct NodeReader {
    socket: Arc<UdpSocket>,
    socket_addr: SocketAddr,
    sender: UnboundedSender<(Message, SocketAddr)>,
    cancellation_token: CancellationToken,
}

impl NodeReader {
    /// Start the main reader loop of a node server.
    /// This will handle incoming packets and parse them before delivering them to the node server.
    async fn start(&self) {
        loop {
            let mut buffer = [0u8; MAX_PACKET_SIZE];
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Ok((len, addr)) = self.socket.recv_from(&mut buffer) => {
                    if let Err(e) = self.handle_incoming_message(&buffer[0..len], addr).await {
                        warn!("{} failed to read incoming message from {}, {}", self, addr, e);
                    }
                },
            }
        }
        debug!("{} main loop ended", self);
    }

    async fn handle_incoming_message(&self, bytes: &[u8], addr: SocketAddr) -> Result<()> {
        // check if the port of the sender is known
        if addr.port() == 0 {
            debug!(
                "{} received packet with unknown port, ignoring packet message",
                self
            );
            return Ok(());
        }

        let start_time = Instant::now();
        let message = serde_bencode::from_bytes::<Message>(bytes)?;
        let elapsed = start_time.elapsed();
        trace!(
            "{} read {} bytes from {} in {}.{:03}ms",
            self,
            bytes.len(),
            addr,
            elapsed.as_millis(),
            elapsed.as_micros(),
        );

        self.sender.send((message, addr)).map_err(|_| Error::Closed)
    }
}

/// Represents a request that has been sent to a DHT node and is awaiting a response.
#[derive(Debug)]
struct PendingRequest {
    request_type: PendingRequestType,
    timestamp_sent: Instant,
}

/// The type of a pending request.
/// It determines which result should be sent back to the waiter.
#[derive(Debug)]
enum PendingRequestType {
    Ping(Sender<Result<()>>),
    FindNode(Sender<Result<Vec<Node>>>),
}

/// The processed message which will be used as reply.
#[derive(Debug)]
enum Reply {
    Ping,
    FindNode(Vec<Node>),
}

#[derive(Debug, Display, Clone, PartialEq, Eq, Hash)]
#[display(fmt = "{}[{}]", addr, id)]
struct TransactionKey {
    pub id: u16,
    pub addr: SocketAddr,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::torrent::dht::tests::create_node_server_pair;
    use crate::{init_logger, timeout};

    #[tokio::test]
    async fn test_server_new() {
        init_logger!();
        let node_id = NodeId::new();
        let result = NodeServer::new(node_id, Vec::new())
            .await
            .expect("expected a new DHT server");

        assert_eq!(
            node_id,
            result.id(),
            "expected a new random node ID to have been generated"
        );
        assert_ne!(
            0,
            result.port(),
            "expected a server port to have been present"
        );
    }

    mod ping {
        use super::*;

        #[tokio::test]
        async fn test_ping_valid_address() {
            init_logger!();
            let (incoming, outgoing) = create_node_server_pair().await;

            let _ = timeout!(
                outgoing.ping(([127, 0, 0, 1], incoming.port()).into()),
                Duration::from_millis(750),
                "failed to ping node"
            )
            .expect("expected the ping to have been succeeded");

            // check if the incoming server has added the outgoing node
            let routing_table = incoming.inner.routing_table.lock().await;
            let result = routing_table.find_node(&outgoing.id());
            assert_ne!(
                None, result,
                "expected the outgoing ping node to have been found within the routing table"
            );

            // check if the outgoing server has added the node that was pinged
            let routing_table = outgoing.inner.routing_table.lock().await;
            let result = routing_table.find_node(&incoming.id());
            assert_ne!(
                None, result,
                "expected the pinged target node to have been added to the outgoing server"
            );
        }

        #[tokio::test]
        async fn test_ping_invalid_address() {
            init_logger!();
            let (incoming, outgoing) = create_node_server_pair().await;

            let result = timeout!(
                outgoing.ping(incoming.addr()), // this will try to send to 0.0.0.0:X
                Duration::from_millis(750),
                "failed to ping node"
            );

            assert_eq!(
                Err(Error::InvalidAddr),
                result,
                "expected an invalid address error"
            );
        }
    }

    mod bootstrap {
        use super::*;
        use crate::torrent::dns::DnsResolver;
        use std::str::FromStr;

        #[tokio::test]
        async fn test_bootstrap_nodes() {
            init_logger!();
            let mut node_server = NodeServer::builder();

            for addr in DEFAULT_BOOTSTRAP_SERVERS().drain(0..2) {
                match DnsResolver::from_str(addr) {
                    Ok(resolver) => {
                        resolver
                            .resolve()
                            .await
                            .into_iter()
                            .flatten()
                            .for_each(|addr| {
                                node_server.bootstrap_node(addr);
                            })
                    }
                    Err(e) => warn!("Failed to resolve IP of node bootstrap \"{}\", {}", addr, e),
                }
            }

            let node_server = node_server.build().await.unwrap();

            select! {
                _ = time::sleep(Duration::from_secs(10)) => assert!(false, "timed-out while bootstrapping nodes"),
                _ = async {
                    while node_server.inner.routing_table.lock().await.len() <= 1 {
                        time::sleep(Duration::from_millis(50)).await;
                    }
                } => {},
            }

            let result = node_server.inner.routing_table.lock().await.len();
            assert_ne!(0, result, "expected at least one bootstrap node");
        }
    }
}
