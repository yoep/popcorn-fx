use crate::torrent::dht::compact::{CompactIPv4Nodes, CompactIPv6Nodes};
use crate::torrent::dht::krpc::{
    ErrorMessage, FindNodeRequest, FindNodeResponse, GetPeersRequest, GetPeersResponse, Message,
    MessagePayload, PingMessage, QueryMessage, ResponseMessage,
};
use crate::torrent::dht::routing_table::RoutingTable;
use crate::torrent::dht::{Error, Node, NodeId, Result, Token, TokenSecret};
use crate::torrent::{CompactIpv4Addr, CompactIpv6Addr, InfoHash, COMPACT_IPV6_ADDR_LEN};
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
use tokio::sync::oneshot::Sender;
use tokio::sync::{oneshot, Mutex};
use tokio::time::{interval, timeout};
use tokio::{select, time};
use tokio_util::sync::CancellationToken;

/// The maximum size of a single UDP packet.
const MAX_PACKET_SIZE: usize = 65_535;
const REFRESH_INTERVAL_MINUTES: u64 = 15;
const SEND_PACKAGE_TIMEOUT_AFTER_SECS: u64 = 6;
const RESPONSE_TIMEOUT_AFTER_SECS: u64 = 8;
const DEFAULT_BUCKET_SIZE: usize = 8;
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

/// The ping operation result sender.
type PingSender = Sender<Result<Node>>;
/// The find node operation result sender.
type FindNodeSender = Sender<Result<Vec<Node>>>;

/// A tracker instance for managing DHT nodes.
/// This instance can be shared between torrents by using [DhtTracker::clone].
#[derive(Debug, Clone)]
pub struct DhtTracker {
    inner: Arc<InnerTracker>,
}

impl DhtTracker {
    /// Create a new builder instance to create a new node server.
    pub fn builder() -> DhtTrackerBuilder {
        DhtTrackerBuilder::default()
    }

    /// Create a new DHT node server with the given node ID.
    /// This function allows creating a server with a specific node id.
    ///
    /// # Arguments
    ///
    /// * `id` - The node ID of the DHT server.
    /// * `routing_nodes` - The router nodes only used for searching new nodes. These are never returned in a response.
    pub async fn new(id: NodeId, routing_nodes: Vec<Node>) -> Result<Self> {
        let socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
        let socket_addr = socket.local_addr()?;
        let (sender, receiver) = unbounded_channel();
        let (command_sender, command_receiver) = unbounded_channel();
        let cancellation_token = CancellationToken::new();
        let reader = NodeReader {
            socket: socket.clone(),
            socket_addr,
            sender,
            cancellation_token: cancellation_token.clone(),
        };
        let inner = Arc::new(InnerTracker {
            id,
            transaction_id: Default::default(),
            socket,
            socket_addr,
            routing_table: Mutex::new(RoutingTable::new(id, DEFAULT_BUCKET_SIZE, routing_nodes)),
            pending_requests: Default::default(),
            send_timeout: Duration::from_secs(SEND_PACKAGE_TIMEOUT_AFTER_SECS),
            command_sender,
            cancellation_token,
        });

        // start the reader in a separate thread
        tokio::spawn(async move {
            reader.start().await;
        });

        let inner_main = inner.clone();
        tokio::spawn(async move {
            inner_main.start(receiver, command_receiver).await;
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

    /// Add the given router node address to the tracker.
    pub fn add_router_node(&self, addr: SocketAddr) {
        let node = Node::new(NodeId::from(&addr), addr);
        let _ = self
            .inner
            .command_sender
            .send(TrackerCommand::AddRouterNode(node));
    }

    /// Try to ping the given node address.
    /// This function waits for a response from the node, so it might be recommended to wrap this fn call in a timeout.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    ///  use std::net::SocketAddr;
    ///  use std::time::Duration;
    ///  use tokio::select;
    ///  use tokio::time;
    ///  use popcorn_fx_torrent::torrent::dht::DhtTracker;
    ///
    ///  async fn example(dht_tracker: &DhtTracker, target_addr: SocketAddr) {
    ///     select! {
    ///         _ = time::sleep(Duration::from_secs(10)) => return,
    ///         result = dht_tracker.ping(target_addr) => {
    ///             // do something with the result response
    ///             return
    ///         }
    ///     }
    ///  }
    /// ```
    pub async fn ping(&self, addr: SocketAddr) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.inner.ping(&addr, tx).await;
        let node = rx.await.map_err(|_| Error::Closed)??;
        self.inner.add_verified_node(node).await;
        Ok(())
    }

    /// Try to find nearby nodes for the given node id.
    /// This function waits for a response from one or more nodes within the routing table.
    /// Each queried node is limited to the given timeout.
    pub async fn find_nodes(&self, target_id: NodeId, timeout: Duration) -> Result<Vec<Node>> {
        self.inner.find_nodes(target_id, timeout).await
    }

    /// Try to find peers for the given torrent info hash.
    ///This function waits for a response from one oe more nodes within the routing table.
    /// Each queried node is limited to the given timeout.
    pub async fn get_peers(
        &self,
        info_hash: &InfoHash,
        timeout: Duration,
    ) -> Result<Vec<SocketAddr>> {
        self.inner.get_peers(info_hash, timeout).await
    }

    /// Close the DHT node server.
    pub fn close(&self) {
        self.inner.close()
    }
}

impl Drop for DhtTracker {
    fn drop(&mut self) {
        // check if only the main loop remains
        if Arc::strong_count(&self.inner) == 2 {
            self.close()
        }
    }
}

#[derive(Debug, Default)]
pub struct DhtTrackerBuilder {
    node_id: Option<NodeId>,
    routing_nodes: Option<Vec<SocketAddr>>,
}

impl DhtTrackerBuilder {
    /// Set the ID of the node server.
    pub fn node_id(&mut self, id: NodeId) -> &mut Self {
        self.node_id = Some(id);
        self
    }

    /// Add the given address to the routing nodes used for searching new nodes.
    pub fn routing_node(&mut self, addr: SocketAddr) -> &mut Self {
        self.routing_nodes.get_or_insert_default().push(addr);
        self
    }

    /// Set the routing nodes to use for searching new nodes.
    /// This replaces any already existing configured routing nodes.
    pub fn routing_nodes(&mut self, nodes: Vec<SocketAddr>) -> &mut Self {
        self.routing_nodes = Some(nodes);
        self
    }

    /// Try to create a new DHT node server from this builder.
    pub async fn build(&mut self) -> Result<DhtTracker> {
        let node_id = self.node_id.take().unwrap_or_else(|| NodeId::new());
        let bootstrap_server = self
            .routing_nodes
            .take()
            .unwrap_or_default()
            .into_iter()
            .map(|addr| Node::new(NodeId::from(&addr), addr))
            .collect();

        DhtTracker::new(node_id, bootstrap_server).await
    }
}

/// An internal command executed within the tracker.
#[derive(Debug)]
enum TrackerCommand {
    /// Ping the given node address.
    Ping(SocketAddr, PingSender),
    /// Query the closest nodes at the given node address for the target node id.
    FindNode(NodeId, SocketAddr, FindNodeSender),
    /// Try to add the node to the routing by first verifying if it's valid
    AddNode(Node),
    /// Add the verified node to the routing table
    AddVerifiedNode(Node),
    /// Add the router node to the routing table
    AddRouterNode(Node),
}

#[derive(Debug, Display)]
#[display(fmt = "DHT node server [{}]", socket_addr)]
struct InnerTracker {
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
    /// The timeout while trying to send packages to a target address
    send_timeout: Duration,
    /// The underlying async command sender
    command_sender: UnboundedSender<TrackerCommand>,
    /// The cancellation token of the server
    cancellation_token: CancellationToken,
}

impl InnerTracker {
    async fn start(
        &self,
        mut receiver: UnboundedReceiver<(Message, SocketAddr)>,
        mut command_receiver: UnboundedReceiver<TrackerCommand>,
    ) {
        {
            let routing_table = self.routing_table.lock().await;
            if routing_table.router_nodes().len() > 0 {
                let tracker_id = self.id;
                let tracker_info = self.to_string();
                let command_sender = self.command_sender.clone();
                let router_nodes = routing_table.router_nodes().to_vec();
                tokio::spawn(async move {
                    Self::bootstrap(tracker_id, tracker_info, router_nodes, command_sender).await;
                });
            }
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
                Some(command) = command_receiver.recv() => self.handle_command(command).await,
                _ = refresh_interval.tick() => self.refresh_routing_table().await,
                _ = cleanup_interval.tick() => self.cleanup_pending_requests().await,
            }
        }
        debug!("{} main loop ended", self);
    }

    /// Try to process an incoming DHT message from the given node address.
    async fn handle_incoming_message(&self, message: Message, addr: SocketAddr) -> Result<()> {
        trace!(
            "{} received message (transaction {}) from {}, {:?}",
            self,
            message.transaction_id(),
            addr,
            message
        );
        let id = message.transaction_id();
        let key = TransactionKey { id, addr };

        // check the type of the message
        match &message.payload {
            MessagePayload::Query(query) => match query {
                QueryMessage::Ping { request } => {
                    self.add_verified_node(Node::new(request.id, addr.clone()))
                        .await;
                    self.send_response(
                        &message,
                        ResponseMessage::Ping {
                            response: PingMessage { id: self.id },
                        },
                        &addr,
                    )
                    .await?;
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
                QueryMessage::GetPeers { request } => {
                    let token: Token;

                    {
                        let mut routing_table = self.routing_table.lock().await;
                        if let Some(node) = routing_table.find_node_mut(&request.id) {
                            token = node.generate_token(addr.ip());
                        } else {
                            return self
                                .send_error(
                                    &message,
                                    ErrorMessage::Generic("unknown node id".to_string()),
                                    &addr,
                                )
                                .await;
                        }
                    }

                    self.send_response(
                        &message,
                        ResponseMessage::GetPeers {
                            response: GetPeersResponse {
                                id: self.id,
                                token: token.to_vec(),
                                values: None,
                                nodes: None,
                            },
                        },
                        &addr,
                    )
                    .await?;
                }
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
                        ResponseMessage::Ping { response } => {
                            self.node_queried(&response.id).await;
                            reply = Ok(Reply::Ping(Node::new(response.id, addr)));
                        }
                        ResponseMessage::FindNode { response } => {
                            self.node_queried(&response.id).await;
                            let nodes = CompactIPv4Nodes::try_from(response.nodes.as_slice()).map(
                                |compact_nodes| {
                                    compact_nodes
                                        .into_iter()
                                        .map(|node| Node::new(node.id, node.addr.clone().into()))
                                        .collect::<Vec<Node>>()
                                },
                            );

                            reply = match nodes {
                                Ok(nodes) => {
                                    debug!("{} discovered a total of {} nodes", self, nodes.len());
                                    for node in &nodes {
                                        let _ = self
                                            .command_sender
                                            .send(TrackerCommand::AddNode(node.clone()));
                                    }

                                    Ok(Reply::FindNode(nodes))
                                }
                                Err(e) => Err(e),
                            }
                        }
                        ResponseMessage::GetPeers { response } => {
                            self.node_queried(&response.id).await;
                            if let Err(e) = self
                                .update_announce_token(&response.id, &response.token)
                                .await
                            {
                                reply = Err(e);
                            } else {
                                if let Some(nodes) = &response.nodes {
                                    match self.parse_compact_nodes(nodes.as_slice()) {
                                        Ok(nodes) => {
                                            for node in nodes {
                                                let _ = self
                                                    .command_sender
                                                    .send(TrackerCommand::AddNode(node));
                                            }
                                        }
                                        Err(e) => warn!("{} failed to parse nodes, {}", self, e),
                                    }
                                }

                                let peers = if let Some(peers) = &response.values {
                                    peers
                                        .iter()
                                        .map(|e| {
                                            if e.len() == COMPACT_IPV6_ADDR_LEN {
                                                CompactIpv6Addr::try_from(e.as_slice())
                                                    .map(Into::<SocketAddr>::into)
                                                    .map_err(|e| {
                                                        Error::Parse(format!(
                                                            "failed to parse compact peer, {}",
                                                            e
                                                        ))
                                                    })
                                            } else {
                                                CompactIpv4Addr::try_from(e.as_slice())
                                                    .map(Into::<SocketAddr>::into)
                                                    .map_err(|e| {
                                                        Error::Parse(format!(
                                                            "failed to parse compact peer, {}",
                                                            e
                                                        ))
                                                    })
                                            }
                                        })
                                        .collect::<Result<Vec<SocketAddr>>>()
                                } else {
                                    Ok(Vec::with_capacity(0))
                                };

                                reply = peers.map(Reply::GetPeers)
                            }
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

    /// Process a received tracker command.
    async fn handle_command(&self, command: TrackerCommand) {
        match command {
            TrackerCommand::Ping(addr, sender) => self.ping(&addr, sender).await,
            TrackerCommand::FindNode(id, addr, sender) => self.find_node(id, &addr, sender).await,
            TrackerCommand::AddNode(node) => self.add_node(node).await,
            TrackerCommand::AddVerifiedNode(node) => self.add_verified_node(node).await,
            TrackerCommand::AddRouterNode(node) => self.add_router_node(node).await,
        }
    }

    /// Ping the given node address.
    ///
    /// # Arguments
    ///
    /// * `addr` - the node address to ping.
    /// * `sender` - The result sender for the ping operation.
    async fn ping(&self, addr: &SocketAddr, sender: PingSender) {
        self.send_query(
            QueryMessage::Ping {
                request: PingMessage { id: self.id },
            },
            addr,
            PendingRequestType::Ping(sender),
        )
        .await;
    }

    /// Find the closest nodes for the given target node id.
    /// This will query all stored nodes within the routing table.
    ///
    /// # Arguments
    ///
    /// * `target_id` - The target node id to retrieve the closest nodes of.
    /// * `timeout` - The timeout of the query for individual nodes.
    async fn find_nodes(&self, target_id: NodeId, timeout: Duration) -> Result<Vec<Node>> {
        let nodes: Vec<Node>;

        {
            let routing_table = self.routing_table.lock().await;
            nodes = routing_table.search_nodes();
        }

        let futures: Vec<_> = nodes
            .into_iter()
            .map(|node| async move {
                let (tx, rx) = oneshot::channel();
                self.find_node(target_id, &node.addr, tx).await;
                select! {
                    _ = time::sleep(timeout) => Err(Error::Timeout),
                    result = rx => {
                        match result {
                            Ok(e) => e,
                            Err(_) => Err(Error::Closed),
                        }
                    },
                }
            })
            .collect();

        Ok(futures::future::join_all(futures)
            .await
            .into_iter()
            .flat_map(|result| match result {
                Err(e) => {
                    trace!("{} failed to query nodes, {}", self, e);
                    Vec::with_capacity(0)
                }
                Ok(e) => e,
            })
            .collect())
    }

    /// Find the closest nodes for the given target node id.
    ///
    /// # Arguments
    ///
    /// * `target_id` - The target node id to retrieve the closest nodes of.
    /// * `addr` - The node address to query.
    /// * `sender` - Teh result sender for the find node operation.
    async fn find_node(&self, target_id: NodeId, addr: &SocketAddr, sender: FindNodeSender) {
        self.send_query(
            QueryMessage::FindNode {
                request: FindNodeRequest {
                    id: self.id,
                    target: target_id,
                },
            },
            addr,
            PendingRequestType::FindNode(sender),
        )
        .await;
    }

    /// Get peers for the given torrent info hash.
    ///
    /// # Arguments
    ///
    /// * `info_hash` - The info hash to search peers for.
    /// * `timeout` - The timeout of the query for individual nodes.
    async fn get_peers(&self, info_hash: &InfoHash, timeout: Duration) -> Result<Vec<SocketAddr>> {
        let nodes: Vec<Node>;

        {
            let routing_table = self.routing_table.lock().await;
            nodes = routing_table.search_nodes();
        }

        let futures: Vec<_> = nodes
            .into_iter()
            .map(|node| {
                let info_hash = info_hash.clone();
                async move {
                    let (tx, rx) = oneshot::channel();
                    self.send_query(
                        QueryMessage::GetPeers {
                            request: GetPeersRequest {
                                id: self.id,
                                info_hash,
                            },
                        },
                        &node.addr,
                        PendingRequestType::GetPeers(tx),
                    )
                    .await;

                    select! {
                        _ = time::sleep(timeout) => Err(Error::Timeout),
                        result = rx => {
                            match result {
                                Ok(e) => e,
                                Err(_) => Err(Error::Closed),
                            }
                        },
                    }
                }
            })
            .collect();

        Ok(futures::future::join_all(futures)
            .await
            .into_iter()
            .flat_map(|result| match result {
                Err(e) => {
                    trace!("{} failed to get peers, {}", self, e);
                    Vec::with_capacity(0)
                }
                Ok(e) => e,
            })
            .collect())
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
            Self::pending_request_error(tx, Error::InvalidAddr);
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
                Self::pending_request_error(tx, err);
                return;
            }
        };

        debug!(
            "{} is sending query \"{}\" (transaction {}) to {}",
            self, name, id, addr
        );
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
                Self::pending_request_error(tx, err);
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
            .ip((*addr).into())
            .build()?;

        self.send(message, addr).await
    }

    /// Send the given error response for a query message.
    ///
    /// # Arguments
    ///
    /// * `message` - The original query message.
    /// * `error` - The error payload.
    /// * `addr` - The node address to send the response to.
    async fn send_error(
        &self,
        message: &Message,
        error: ErrorMessage,
        addr: &SocketAddr,
    ) -> Result<()> {
        let message = Message::builder()
            .transaction_id(message.transaction_id())
            .payload(MessagePayload::Error(error))
            .ip((*addr).into())
            .build()?;

        self.send(message, addr).await
    }

    async fn send(&self, message: Message, addr: &SocketAddr) -> Result<()> {
        if self.cancellation_token.is_cancelled() {
            return Err(Error::Closed);
        }

        let bytes = serde_bencode::to_bytes(&message)?;

        trace!(
            "{} is sending message ({} bytes, transaction {}) to {}, {:?}",
            self,
            bytes.len(),
            message.transaction_id(),
            addr,
            message
        );
        let start_time = Instant::now();
        timeout(
            self.send_timeout,
            self.socket.send_to(bytes.as_slice(), addr),
        )
        .await
        .map_err(|_| {
            Error::Io(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("connection to {} has timed out", addr),
            ))
        })?
        .map_err(Error::from)?;
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

    /// Try to add the given node to the routing table.
    /// The node will be pinged before it's being added to the routing table.
    async fn add_node(&self, node: Node) {
        let (tx, rx) = oneshot::channel();
        self.ping(&node.addr, tx).await;

        let tracker_info = self.to_string();
        let command_sender = self.command_sender.clone();
        tokio::spawn(async move {
            let addr = node.addr;
            // let _ = command_sender.send(TrackerCommand::Ping(node.addr, tx));

            match timeout(Duration::from_secs(3), rx).await {
                Ok(_) => {
                    let _ = command_sender.send(TrackerCommand::AddVerifiedNode(node));
                }
                Err(e) => trace!("{} failed to ping {}, {}", tracker_info, addr, e),
            }
        });
    }

    /// Add the given verified node.
    /// This should only be called if the node could be reached with a ping.
    async fn add_verified_node(&self, node: Node) {
        let addr = node.addr;
        if let Some(bucket) = self.routing_table.lock().await.add_node(node) {
            debug!("{} added node {} to bucket {}", self, addr, bucket);
        }
    }

    /// Add the given router node to the routing table.
    /// These will only be used during searches, but never returned in a response.
    async fn add_router_node(&self, node: Node) {
        self.routing_table.lock().await.add_router_node(node)
    }

    /// Update the node metrics with a successful query.
    async fn node_queried(&self, node_id: &NodeId) {
        let mut routing_table = self.routing_table.lock().await;
        if let Some(node) = routing_table.find_node_mut(node_id) {
            node.confirmed();
        }
    }

    /// Update the node metrics with a timed out query.
    async fn node_timeout(&self, node_addr: &SocketAddr) {
        let mut routing_table = self.routing_table.lock().await;
        if let Some(node) = routing_table.find_node_by_addr_mut(node_addr) {
            node.timed_out();
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
            self.node_timeout(&key.addr).await;
            if let Some(request) = pending_requests.remove(&key) {
                Self::pending_request_error(request.request_type, Error::Timeout);
            }
        }
    }

    /// Try to update the announce token for the given node ID.
    ///
    /// It returns an error when the node ID couldn't be found within the routing table or the token value is invalid.
    async fn update_announce_token(
        &self,
        id: &NodeId,
        token_value: impl AsRef<[u8]>,
    ) -> Result<()> {
        let mut routing_table = self.routing_table.lock().await;
        let node = routing_table
            .find_node_mut(id)
            .ok_or(Error::InvalidNodeId)?;
        let token = TokenSecret::try_from(token_value.as_ref())?;
        node.update_announce_token(token);
        Ok(())
    }

    /// Check if the current track is an IPv4 tracker.
    fn is_ipv4(&self) -> bool {
        self.socket_addr.is_ipv4()
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

    /// Parse the given compact nodes byte slice into nodes.
    fn parse_compact_nodes(&self, compact_nodes_bytes: &[u8]) -> Result<Vec<Node>> {
        if self.is_ipv4() {
            CompactIPv4Nodes::try_from(compact_nodes_bytes)
                .map(|nodes| nodes.into_iter().map(Node::from).collect())
        } else {
            CompactIPv6Nodes::try_from(compact_nodes_bytes)
                .map(|nodes| nodes.into_iter().map(Node::from).collect())
        }
    }

    fn close(&self) {
        if self.cancellation_token.is_cancelled() {
            return;
        }

        trace!("{} is closing", self);
        self.cancellation_token.cancel();
    }

    /// Bootstrap the given nodes for a tracker through the given command sender.
    async fn bootstrap(
        tracker_id: NodeId,
        tracker_info: String,
        bootstrap_nodes: Vec<Node>,
        command_sender: UnboundedSender<TrackerCommand>,
    ) {
        let mut futures = vec![];

        // iterate over the initial bootstrap nodes
        debug!(
            "{} is bootstrapping {} nodes",
            tracker_info,
            bootstrap_nodes.len()
        );
        for node in bootstrap_nodes.iter() {
            let command_sender = command_sender.clone();
            futures.push(Self::bootstrap_node(&node.addr, tracker_id, command_sender));
        }

        let total_bootstrapped_nodes = futures::future::join_all(futures)
            .await
            .into_iter()
            .filter(|result| result.is_ok())
            .count();
        info!(
            "{} bootstrapped {} nodes",
            tracker_info, total_bootstrapped_nodes
        );
    }

    /// Bootstrap from the given node address.
    async fn bootstrap_node(
        addr: &SocketAddr,
        tracker_id: NodeId,
        command_sender: UnboundedSender<TrackerCommand>,
    ) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        let _ = command_sender.send(TrackerCommand::Ping(*addr, tx));
        let node = timeout(Duration::from_secs(6), rx)
            .await
            .map_err(|_| Error::Timeout)?
            .map_err(|_| Error::Closed)??;

        // add the bootstrap node
        let _ = command_sender.send(TrackerCommand::AddVerifiedNode(node));

        // request available nodes from the bootstrap node
        let (tx, rx) = oneshot::channel();
        let _ = command_sender.send(TrackerCommand::FindNode(tracker_id, *addr, tx));

        timeout(Duration::from_secs(6), rx)
            .await
            .map(|_| ())
            .map_err(|_| Error::Timeout)
    }

    fn send_reply(response: PendingRequestType, result: Result<Reply>) {
        match response {
            PendingRequestType::Ping(tx) => {
                let _ = tx.send(Self::map_reply(result, "Ping", |r| match r {
                    Reply::Ping(node) => Some(node),
                    _ => None,
                }));
            }
            PendingRequestType::FindNode(tx) => {
                let _ = tx.send(Self::map_reply(result, "FindNode", |r| match r {
                    Reply::FindNode(nodes) => Some(nodes),
                    _ => None,
                }));
            }
            PendingRequestType::GetPeers(tx) => {
                let _ = tx.send(Self::map_reply(result, "GetPeers", |r| match r {
                    Reply::GetPeers(peers) => Some(peers),
                    _ => None,
                }));
            }
        };
    }

    fn map_reply<T, F>(result: Result<Reply>, expected_variant: &str, f: F) -> Result<T>
    where
        F: FnOnce(Reply) -> Option<T>,
    {
        result.and_then(|r| {
            let r_type = format!("{:?}", r);
            f(r).ok_or_else(|| {
                Error::InvalidMessage(format!(
                    "expected Reply::{}, but got {} instead",
                    expected_variant, r_type
                ))
            })
        })
    }

    /// Invoke the given [Error] for a pending request.
    fn pending_request_error(response: PendingRequestType, err: Error) {
        match response {
            PendingRequestType::Ping(tx) => {
                let _ = tx.send(Err(err));
            }
            PendingRequestType::FindNode(tx) => {
                let _ = tx.send(Err(err));
            }
            PendingRequestType::GetPeers(tx) => {
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
    Ping(Sender<Result<Node>>),
    FindNode(Sender<Result<Vec<Node>>>),
    GetPeers(Sender<Result<Vec<SocketAddr>>>),
}

/// The processed message which will be used as reply.
#[derive(Debug)]
enum Reply {
    Ping(Node),
    FindNode(Vec<Node>),
    GetPeers(Vec<SocketAddr>),
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

    use crate::create_node_server_pair;
    use crate::{init_logger, timeout};

    mod new {
        use super::*;

        #[tokio::test]
        async fn test_tracker_new() {
            init_logger!();
            let node_id = NodeId::new();
            let tracker = DhtTracker::new(node_id, Vec::new())
                .await
                .expect("expected a new DHT server");

            // verify the tracker id
            assert_eq!(
                node_id,
                tracker.id(),
                "expected a new random node ID to have been generated"
            );
            assert_ne!(
                0,
                tracker.port(),
                "expected a server port to have been present"
            );

            // verify the routing table id
            let routing_table_id = tracker.inner.routing_table.lock().await.id;
            assert_eq!(
                node_id, routing_table_id,
                "expected the routing table id to match the given ID"
            );
        }
    }

    mod add_router_node {
        use super::*;
        use crate::assert_timeout;

        #[tokio::test]
        async fn test_add_router_node() {
            init_logger!();
            let socket_addr: SocketAddr = ([133, 156, 76, 80], 8900).into();
            let tracker = DhtTracker::builder().build().await.unwrap();

            tracker.add_router_node(socket_addr.clone());
            assert_timeout!(
                Duration::from_millis(500),
                tracker
                    .inner
                    .routing_table
                    .lock()
                    .await
                    .router_nodes()
                    .len()
                    == 1,
                "expected the router node to have been added to the routing table"
            );

            let router_nodes = tracker
                .inner
                .routing_table
                .lock()
                .await
                .router_nodes()
                .to_vec();
            assert_eq!(
                router_nodes.len(),
                1,
                "expected the node to have been added to the router nodes"
            );
            assert_eq!(
                socket_addr, router_nodes[0].addr,
                "expected the router node to match the router address"
            );
        }
    }

    mod ping {
        use super::*;

        #[tokio::test]
        async fn test_ping_valid_address() {
            init_logger!();
            let (incoming, outgoing) = create_node_server_pair!();

            let _ = timeout!(
                outgoing.ping(([127, 0, 0, 1], incoming.port()).into()),
                Duration::from_millis(750),
                "failed to ping node"
            )
            .expect("expected the ping to have been succeeded");

            // check if the incoming server has added the node that pinged it
            let routing_table = incoming.inner.routing_table.lock().await;
            let result = routing_table.find_node(&outgoing.id());
            assert_ne!(
                None, result,
                "expected the incoming ping node to have been added to the routing table"
            );

            // check if the outgoing server has added the pinged target node
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
            let (incoming, outgoing) = create_node_server_pair!();

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

    mod find_node {
        use super::*;

        #[tokio::test]
        async fn test_find_node() {
            init_logger!();
            let rand = 2;
            let search_node_id = NodeId::from_ip_with_rand(&[132, 141, 12, 40].into(), rand);
            let node_incoming_id = NodeId::from_ip_with_rand(&[127, 0, 0, 1].into(), rand);
            let node_outgoing_id = NodeId::from_ip_with_rand(&[127, 0, 0, 1].into(), rand);
            let (incoming, outgoing) = create_node_server_pair!(node_incoming_id, node_outgoing_id);

            // register the incoming tracker with the outgoing tracker
            outgoing
                .inner
                .add_router_node(Node::new(
                    incoming.id(),
                    ([127, 0, 0, 1], incoming.port()).into(),
                ))
                .await;

            // calculate the bucket which will be retrieved by the search node
            let bucket_index = node_incoming_id.distance(&search_node_id);
            // create a node which matches the search bucket index
            let nearby_node = create_bucket_matching_node(bucket_index, node_incoming_id);
            incoming
                .inner
                .add_verified_node(Node::new(nearby_node, ([132, 141, 45, 30], 8090).into()))
                .await;

            // request the node info from the nearby node
            let result = outgoing
                .find_nodes(search_node_id, Duration::from_millis(500))
                .await
                .expect("expected to retrieve relevant nodes");
            assert_eq!(1, result.len(), "expected one node to have been present");
        }
    }

    mod get_peers {
        use super::*;

        use std::str::FromStr;

        #[tokio::test]
        async fn test_get_peers() {
            init_logger!();
            let info_hash = InfoHash::from_str("urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7")
                .expect("expected a valid hash");
            let (incoming, outgoing) = create_node_server_pair!();

            // register the incoming tracker with the outgoing tracker
            outgoing
                .inner
                .add_verified_node(Node::new(
                    incoming.id(),
                    ([127, 0, 0, 1], incoming.port()).into(),
                ))
                .await;

            let result = outgoing
                .get_peers(&info_hash, Duration::from_secs(2))
                .await
                .expect("expected to get peers");
            assert_eq!(
                Vec::<SocketAddr>::with_capacity(0),
                result,
                "expected an empty peers list to have been returned"
            );
        }
    }

    mod bootstrap {
        use super::*;
        use tokio::time;

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn test_bootstrap_nodes() {
            init_logger!();
            let rand = 13;
            let node_id = NodeId::from_ip_with_rand(&[141, 130, 12, 89].into(), rand);
            let bootstrap_node_id = NodeId::from_ip_with_rand(&[180, 13, 0, 3].into(), rand);
            let find_node_id = NodeId::from_ip_with_rand(&[127, 0, 0, 1].into(), rand);
            let (bootstrap_node, find_node) =
                create_node_server_pair!(bootstrap_node_id, find_node_id);

            // add a node to the bootstrap node which can be found by the `find_node` search
            let distance = bootstrap_node_id.distance(&node_id);
            let node_id = create_bucket_matching_node(distance, bootstrap_node_id);
            bootstrap_node
                .inner
                .add_verified_node(Node::new(
                    NodeId::new(),
                    ([127, 0, 0, 1], find_node.port()).into(),
                ))
                .await;

            let tracker = DhtTracker::builder()
                .node_id(node_id)
                .routing_nodes(vec![
                    ([127, 0, 0, 1], bootstrap_node.port()).into(),
                    ([127, 0, 0, 1], find_node.port()).into(),
                ])
                .build()
                .await
                .expect("expected a new DHT tracker to have been created");

            select! {
                _ = time::sleep(Duration::from_secs(20)) => assert!(false, "timed-out while bootstrapping nodes"),
                _ = async {
                    while tracker.inner.routing_table.lock().await.len() <= 1 {
                        time::sleep(Duration::from_millis(50)).await;
                    }
                } => {},
            }

            let result = tracker.inner.routing_table.lock().await.len();
            assert_ne!(0, result, "expected at least one bootstrap node");
        }
    }

    fn create_bucket_matching_node(bucket_index: u8, routing_table_id: NodeId) -> NodeId {
        let mut node_id = NodeId::new();

        while routing_table_id.distance(&node_id) != bucket_index {
            node_id = NodeId::new();
        }

        node_id
    }
}
