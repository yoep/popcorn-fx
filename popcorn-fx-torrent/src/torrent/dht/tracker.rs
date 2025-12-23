use crate::torrent::dht::compact::{
    CompactIPv4Node, CompactIPv4Nodes, CompactIPv6Node, CompactIPv6Nodes,
};
use crate::torrent::dht::krpc::{
    AnnouncePeerResponse, ErrorMessage, FindNodeRequest, FindNodeResponse, GetPeersRequest,
    GetPeersResponse, Message, MessagePayload, PingMessage, QueryMessage, ResponseMessage, Version,
    WantFamily,
};
use crate::torrent::dht::observer::Observer;
use crate::torrent::dht::peers::PeerStorage;
use crate::torrent::dht::routing_table::RoutingTable;
use crate::torrent::dht::traversal::TraversalAlgorithm;
use crate::torrent::dht::{
    DhtMetrics, Error, Node, NodeId, NodeState, NodeToken, Result, DEFAULT_ROUTING_NODE_SERVERS,
};
use crate::torrent::metrics::Metric;
use crate::torrent::{
    CompactIpAddr, CompactIpv4Addr, CompactIpv4Addrs, CompactIpv6Addr, CompactIpv6Addrs, InfoHash,
};
use derive_more::Display;
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use itertools::Itertools;
use log::{debug, trace, warn};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tokio::net::{lookup_host, UdpSocket};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot::Sender;
use tokio::sync::{oneshot, Mutex, MutexGuard};
use tokio::time::{interval, timeout};
use tokio::{select, time};
use tokio_util::sync::CancellationToken;
use url::Url;

/// The maximum size of a single UDP packet.
const MAX_PACKET_SIZE: usize = 65_535;
const VERSION_IDENTIFIER: &str = "FX0001";
const SEND_PACKAGE_TIMEOUT: Duration = Duration::from_secs(2);
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(8);
const REFRESH_TIMEOUT: Duration = Duration::from_secs(60 * 15);
const BOOTSTRAP_INTERVAL: Duration = Duration::from_secs(2);
const REFRESH_INTERVAL: Duration = Duration::from_secs(60 * 5);
const CLEANUP_INTERVAL: Duration = Duration::from_secs(5);
const STATS_INTERVAL: Duration = Duration::from_secs(1);
const DEFAULT_BUCKET_SIZE: usize = 8;

#[derive(Debug)]
pub enum DhtEvent {
    /// Invoked when the node ID of the DHT server changes.
    IDChanged,
    /// Invoked when the external IP address of the DHT server changes.
    ExternalIpChanged(IpAddr),
    /// Invoked when a new node is added to the routing table.
    NodeAdded(Node),
    /// Invoked when the stats of the DHT server are updated.
    Stats(DhtMetrics),
}

/// A tracker instance for managing DHT nodes.
/// This instance can be shared between torrents by using [DhtTracker::clone].
#[derive(Debug, Clone)]
pub struct DhtTracker {
    pub(crate) inner: Arc<TrackerContext>,
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
    /// * `routing_nodes` - The routing nodes to use to bootstrap the DHT network.
    pub async fn new(id: NodeId, routing_nodes: Vec<SocketAddr>) -> Result<Self> {
        let socket = Arc::new(Self::bind_socket().await?);
        let socket_addr = socket.local_addr()?;
        let (sender, receiver) = unbounded_channel();
        let (command_sender, command_receiver) = unbounded_channel();
        let cancellation_token = CancellationToken::new();
        let metrics = DhtMetrics::new();
        let reader = NodeReader {
            socket: socket.clone(),
            socket_addr,
            sender,
            cancellation_token: cancellation_token.clone(),
        };

        let inner = Arc::new(TrackerContext {
            transaction_id: Default::default(),
            socket,
            socket_addr,
            routing_table: Mutex::new(RoutingTable::new(id, DEFAULT_BUCKET_SIZE)),
            pending_requests: Default::default(),
            send_timeout: SEND_PACKAGE_TIMEOUT,
            metrics,
            command_sender,
            callbacks: MultiThreadedCallback::new(),
            cancellation_token,
        });

        // start the reader in a separate thread
        tokio::spawn(async move {
            reader.start().await;
        });

        let inner_main = inner.clone();
        tokio::spawn(async move {
            inner_main
                .start(receiver, command_receiver, routing_nodes)
                .await;
        });

        Ok(Self { inner })
    }

    /// Get the ID of the DHT server.
    pub async fn id(&self) -> NodeId {
        self.inner.routing_table.lock().await.id
    }

    /// Get the socket address on which this DHT server is running.
    pub fn addr(&self) -> SocketAddr {
        self.inner.socket_addr
    }

    /// Get the port on which the DHT server is running.
    pub fn port(&self) -> u16 {
        self.inner.socket_addr.port()
    }

    /// Get the DHT network metrics of the DHT server.
    pub fn metrics(&self) -> &DhtMetrics {
        &self.inner.metrics
    }

    /// Get the number of nodes within the routing table.
    pub async fn total_nodes(&self) -> usize {
        self.inner.routing_table.lock().await.len()
    }

    /// Returns all nodes within the routing table of the tracker.
    /// This doesn't include any router/search nodes.
    pub async fn nodes(&self) -> Vec<Node> {
        self.inner
            .routing_table
            .lock()
            .await
            .nodes()
            .cloned()
            .collect()
    }

    /// Add an unverified node to the routing table.
    /// The node will be pinged before it's actually added to the routing table.
    pub async fn add_node(&self, addr: SocketAddr) -> Result<()> {
        let response = self.inner.ping(&addr).await;
        match response.await {
            Ok(node) => {
                let _ = self
                    .inner
                    .command_sender
                    .send(TrackerCommand::AddTraversalNode((*node.id(), *node.addr())));
                Ok(())
            }
            Err(e) => Err(e),
        }
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
        let response = self.inner.ping(&addr).await;
        let _ = response.await?;
        Ok(())
    }

    /// Try to find nearby nodes for the given node id.
    /// This function waits for a response from one or more nodes within the routing table.
    /// Each queried node is limited to the given timeout.
    pub async fn find_nodes(&self, target_id: NodeId, timeout: Duration) -> Response<Vec<Node>> {
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

    /// Create a new UDP socket.
    async fn bind_socket() -> Result<UdpSocket> {
        match Self::bind_dual_stack().await {
            Ok(socket) => Ok(socket),
            Err(e) => {
                debug!("DHT node server failed to bind dual stack socket, {}", e);
                Ok(UdpSocket::bind("0.0.0.0:0").await?)
            }
        }
    }

    /// Try to bind a dual stack IPv4 & IPv6 udp socket.
    async fn bind_dual_stack() -> Result<UdpSocket> {
        // TODO: reimplement dual stack support
        Err(Error::Io(io::Error::new(
            io::ErrorKind::Other,
            "Dual stack support is currently not implemented",
        )))
    }
}

impl Callback<DhtEvent> for DhtTracker {
    fn subscribe(&self) -> Subscription<DhtEvent> {
        self.inner.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<DhtEvent>) {
        self.inner.callbacks.subscribe_with(subscriber)
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
    public_ip: Option<IpAddr>,
    routing_nodes: Vec<SocketAddr>,
    routing_node_urls: Vec<String>,
}

impl DhtTrackerBuilder {
    /// Set the ID of the node server.
    pub fn node_id(&mut self, id: NodeId) -> &mut Self {
        self.node_id = Some(id);
        self
    }

    /// Set the public ip address of the dht tracker.
    pub fn public_ip(&mut self, ip: IpAddr) -> &mut Self {
        self.public_ip = Some(ip);
        self
    }

    /// Add the default routing nodes used for searching new nodes.
    pub fn default_routing_nodes(&mut self) -> &mut Self {
        self.routing_node_urls.extend(
            DEFAULT_ROUTING_NODE_SERVERS()
                .into_iter()
                .map(|e| e.to_string()),
        );
        self
    }

    /// Add the given address to the routing nodes used for searching new nodes.
    pub fn routing_node(&mut self, addr: SocketAddr) -> &mut Self {
        self.routing_nodes.push(addr);
        self
    }

    /// Set the routing nodes to use for searching new nodes.
    /// This replaces any already existing configured routing nodes.
    pub fn routing_nodes(&mut self, nodes: Vec<SocketAddr>) -> &mut Self {
        self.routing_nodes = nodes;
        self
    }

    /// Add the given node url to use for searching new nodes.
    pub fn routing_node_url<S: AsRef<str>>(&mut self, url: S) -> &mut Self {
        self.routing_node_urls.push(url.as_ref().to_string());
        self
    }

    /// Try to create a new DHT node server from this builder.
    pub async fn build(&mut self) -> Result<DhtTracker> {
        let node_id = self.node_id.take().unwrap_or_else(|| {
            self.public_ip
                .take()
                .map(|e| NodeId::from_ip(&e))
                .unwrap_or(NodeId::new())
        });
        let mut routing_nodes: HashSet<SocketAddr> = self.routing_nodes.drain(..).collect();

        for node_url in self.routing_node_urls.drain(..).filter_map(Self::host) {
            match lookup_host(node_url.as_str()).await {
                Ok(addrs) => {
                    routing_nodes.extend(addrs);
                }
                Err(e) => trace!("DHT router node failed to resolve \"{}\", {}", node_url, e),
            }
        }

        DhtTracker::new(node_id, routing_nodes.into_iter().collect()).await
    }

    fn host<S: AsRef<str>>(url: S) -> Option<String> {
        let url = Url::parse(url.as_ref()).ok()?;
        if let Some(host) = url.host_str() {
            let port = url.port().unwrap_or(80);
            return Some(format!("{}:{}", host, port));
        }

        Some(url.as_ref().to_string())
    }
}

/// An internal command executed within the tracker.
#[derive(Debug)]
pub(crate) enum TrackerCommand {
    AddTraversalNode((NodeId, SocketAddr)),
    UpdateExternalIp(IpAddr),
}

#[derive(Debug, Display)]
#[display(fmt = "DHT node server [{}]", socket_addr)]
pub(crate) struct TrackerContext {
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
    /// The tracker metrics of the DHT network
    metrics: DhtMetrics,
    /// The underlying async command sender
    command_sender: UnboundedSender<TrackerCommand>,
    /// The callback of the tracker
    callbacks: MultiThreadedCallback<DhtEvent>,
    /// The cancellation token of the server
    cancellation_token: CancellationToken,
}

impl TrackerContext {
    async fn start(
        &self,
        mut receiver: UnboundedReceiver<ReaderMessage>,
        mut command_receiver: UnboundedReceiver<TrackerCommand>,
        routing_nodes: Vec<SocketAddr>,
    ) {
        let mut observer = Observer::new();
        let mut traversal =
            TraversalAlgorithm::new(self.routing_table.lock().await.bucket_size, routing_nodes);
        let mut peers = PeerStorage::new();

        let mut bootstrap_interval = interval(BOOTSTRAP_INTERVAL);
        let mut refresh_interval = interval(REFRESH_INTERVAL);
        let mut cleanup_interval = interval(CLEANUP_INTERVAL);
        let mut stats_interval = interval(STATS_INTERVAL);

        debug!("{} started", self);
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(message) = receiver.recv() => self.on_message_received(message, &mut observer, &mut peers).await,
                Some(command) = command_receiver.recv() => self.handle_command(command, &mut traversal).await,
                _ = cleanup_interval.tick() => self.cleanup_pending_requests().await,
                _ = refresh_interval.tick() => self.refresh_routing_table().await,
                _ = bootstrap_interval.tick() => self.bootstrap(&mut traversal).await,
                _ = stats_interval.tick() => self.stats_tick().await,
            }
        }
        debug!("{} main loop ended", self);
    }

    /// Returns a reference to the main loop command sender.
    /// This sender allows access to execute task on the main loop.
    pub(crate) fn command_sender(&self) -> &UnboundedSender<TrackerCommand> {
        &self.command_sender
    }

    /// Returns a lock on the routing table of the node server.
    pub(crate) async fn routing_table_lock<'a>(&'a self) -> MutexGuard<'a, RoutingTable> {
        self.routing_table.lock().await
    }

    async fn bootstrap(&self, traversal: &mut TraversalAlgorithm) {
        let id = self.routing_table.lock().await.id;
        traversal.run(id, &self).await;
    }

    async fn on_message_received(
        &self,
        message: ReaderMessage,
        observer: &mut Observer,
        peers: &mut PeerStorage,
    ) {
        match message {
            ReaderMessage::Message {
                message,
                message_len,
                addr,
            } => {
                self.metrics.bytes_in.inc_by(message_len as u64);
                observer.observe(addr, message.ip.as_ref(), &self).await;
                if let Err(e) = self.handle_incoming_message(message, addr, peers).await {
                    warn!("{} failed to process incoming message, {}", self, e);
                }
            }
            ReaderMessage::Error {
                error,
                payload_len,
                addr,
            } => {
                warn!(
                    "{} failed to read incoming message from {}, {}",
                    self, addr, error
                );
                self.metrics.bytes_in.inc_by(payload_len as u64);
                self.metrics.errors.inc();
            }
        }
    }

    /// Try to process an incoming DHT message from the given node address.
    async fn handle_incoming_message(
        &self,
        message: Message,
        addr: SocketAddr,
        peers: &mut PeerStorage,
    ) -> Result<()> {
        trace!(
            "{} received message (transaction {}) from {}, {:?}",
            self,
            message.transaction_id(),
            addr,
            message
        );
        let id = self.routing_table.lock().await.id;
        let node_id = message.id().cloned();
        let transaction_id = message.transaction_id();
        let key = TransactionKey {
            id: transaction_id,
            addr,
        };

        // check the type of the message
        match message.payload {
            MessagePayload::Query(query) => match query {
                QueryMessage::Ping { .. } => {
                    self.send_response(
                        transaction_id,
                        ResponseMessage::Ping {
                            response: PingMessage { id },
                        },
                        &addr,
                    )
                    .await?;
                }
                QueryMessage::FindNode { request } => {
                    let routing_table = self.routing_table.lock().await;
                    let target_node = request.target;
                    let (compact_nodes, compact_nodes6) =
                        Self::closest_node_pairs(&*routing_table, &target_node);

                    self.send_response(
                        transaction_id,
                        ResponseMessage::FindNode {
                            response: FindNodeResponse {
                                id,
                                nodes: compact_nodes.into(),
                                nodes6: compact_nodes6.into(),
                                token: None,
                            },
                        },
                        &addr,
                    )
                    .await?;
                }
                QueryMessage::GetPeers { request } => {
                    self.on_get_peers_request(id, transaction_id, &addr, request, &peers)
                        .await?;
                }
                QueryMessage::AnnouncePeer { request } => {
                    let routing_table = self.routing_table.lock().await;
                    if let Some(node) = routing_table.find_node(&request.id) {
                        // check if the address matches the node
                        if node.addr() != &addr {
                            return self
                                .send_error(
                                    transaction_id,
                                    ErrorMessage::Protocol("Bad node".to_string()),
                                    &addr,
                                )
                                .await;
                        }

                        let is_valid = match NodeToken::try_from(request.token.as_bytes()) {
                            Ok(token) => node.verify_token(&token, &addr.ip()).await,
                            Err(_) => false,
                        };
                        if !is_valid {
                            return self
                                .send_error(
                                    transaction_id,
                                    ErrorMessage::Protocol("Bad token".to_string()),
                                    &addr,
                                )
                                .await;
                        };
                    }

                    peers.update_peer(request.info_hash, addr, request.seed.unwrap_or(false));
                    self.send_response(
                        transaction_id,
                        ResponseMessage::Announce {
                            response: AnnouncePeerResponse { id },
                        },
                        &addr,
                    )
                    .await?;
                }
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
                            if !response.id.verify_id(&addr.ip()) {
                                debug!("{} detected spoofed ping from {}", self, key);
                                Self::send_reply(
                                    pending_request.request_type,
                                    Err(Error::InvalidNodeId),
                                );
                                return Ok(());
                            }

                            self.node_query_result(&addr, true).await;
                            reply = Ok(Reply::Ping(Node::new(response.id, addr)));
                        }
                        ResponseMessage::FindNode { response } => {
                            if !response.id.verify_id(&addr.ip()) {
                                debug!("{} detected spoofed find_node from {}", self, key);
                                Self::send_reply(
                                    pending_request.request_type,
                                    Err(Error::InvalidNodeId),
                                );
                                return Ok(());
                            }

                            self.node_query_result(&addr, true).await;
                            let nodes = response
                                .nodes
                                .as_slice()
                                .into_iter()
                                .map(|e| Node::new(e.id, e.addr.clone().into()))
                                .chain(
                                    response
                                        .nodes6
                                        .as_slice()
                                        .into_iter()
                                        .map(|e| Node::new(e.id, e.addr.clone().into())),
                                )
                                .collect::<Vec<_>>();

                            debug!(
                                "{} node {} discovered a total of {} nodes",
                                self,
                                addr,
                                nodes.len()
                            );
                            reply = Ok(Reply::FindNode(nodes))
                        }
                        ResponseMessage::GetPeers { response } => {
                            match self.on_get_peers_response(&key, &addr, response).await {
                                None => return Ok(()),
                                Some(e) => reply = e,
                            }
                        }
                        ResponseMessage::Announce { response } => {
                            if !response.id.verify_id(&addr.ip()) {
                                debug!("{} detected spoofed announce_peer from {}", self, key);
                                Self::send_reply(
                                    pending_request.request_type,
                                    Err(Error::InvalidNodeId),
                                );
                                return Ok(());
                            }

                            self.node_query_result(&addr, true).await;
                            return Ok(());
                        }
                    }

                    Self::send_reply(pending_request.request_type, reply)
                } else {
                    warn!(
                        "{} received response for unknown request, invalid transaction {}",
                        self, key
                    );
                    self.metrics.errors.inc();
                }
            }
            MessagePayload::Error(err) => {
                self.on_error_response(&addr, &key, err).await;
            }
        }

        if let Some(id) = node_id {
            self.update_node(id, addr).await;
        }
        Ok(())
    }

    /// Process a received get_peers query.
    /// The query will be processed only when the node is already known within the routing table.
    ///
    /// # Arguments
    ///
    /// * `id` - The node id of the server.
    /// * `transaction_id` - The transaction id of the query.
    /// * `addr`- The source address of the node.
    /// * `request` - The get_peers query arguments.
    /// * `peers` - The peer storage of the server.
    async fn on_get_peers_request(
        &self,
        id: NodeId,
        transaction_id: u16,
        addr: &SocketAddr,
        request: GetPeersRequest,
        peers: &PeerStorage,
    ) -> Result<()> {
        let token: NodeToken;
        let nodes: CompactIPv4Nodes;
        let nodes6: CompactIPv6Nodes;

        {
            let routing_table = self.routing_table.lock().await;
            match routing_table.find_node(&request.id) {
                None => {
                    return self
                        .send_error(
                            transaction_id,
                            ErrorMessage::Generic("Bad node".to_string()),
                            &addr,
                        )
                        .await;
                }
                Some(node) => {
                    let info_hash_as_node = match NodeId::try_from(
                        request.info_hash.short_info_hash_bytes().as_slice(),
                    ) {
                        Ok(e) => e,
                        Err(e) => {
                            warn!("{} failed to parse info hash as node id, {}", self, e);
                            return self
                                .send_error(
                                    transaction_id,
                                    ErrorMessage::Server("A Server Error Occurred".to_string()),
                                    &addr,
                                )
                                .await;
                        }
                    };

                    token = node.generate_token().await;
                    (nodes, nodes6) = Self::closest_node_pairs(&*routing_table, &info_hash_as_node);
                }
            }
        }

        let values = peers
            .peers(&request.info_hash)
            .filter(|e| e.addr.is_ipv4() == self.socket_addr.is_ipv4())
            .map(|e| CompactIpAddr::from(e.addr))
            .map(|e| e.as_bytes())
            .concat();

        self.send_response(
            transaction_id,
            ResponseMessage::GetPeers {
                response: GetPeersResponse {
                    id,
                    token: token.to_vec(),
                    values: Some(values).filter(|e| !e.is_empty()),
                    nodes: nodes.into(),
                    nodes6: nodes6.into(),
                },
            },
            &addr,
        )
        .await?;
        Ok(())
    }

    /// Process a received response message for a query.
    async fn on_get_peers_response(
        &self,
        key: &TransactionKey,
        addr: &SocketAddr,
        response: GetPeersResponse,
    ) -> Option<Result<Reply>> {
        if !response.id.verify_id(&addr.ip()) {
            debug!("{} detected spoofed get_peers from {}", self, key);
            return None;
        }

        self.node_query_result(&addr, true).await;
        if let Err(e) = self
            .update_announce_token(&response.id, &response.token)
            .await
        {
            return Some(Err(e));
        }

        let nodes = response
            .nodes
            .as_slice()
            .into_iter()
            .map(|e| Node::new(e.id, e.addr.clone().into()))
            .chain(
                response
                    .nodes6
                    .as_slice()
                    .into_iter()
                    .map(|e| Node::new(e.id, e.addr.clone().into())),
            )
            .collect::<Vec<_>>();
        for node in nodes {
            let _ = self
                .command_sender
                .send(TrackerCommand::AddTraversalNode((*node.id(), *node.addr())));
        }

        let peers: Vec<SocketAddr> = if let Some(values) = response.values {
            match addr.ip() {
                IpAddr::V4(_) => match CompactIpv4Addrs::try_from(values.as_slice()) {
                    Ok(addrs) => addrs.into_iter().map(|e| e.into()).collect::<Vec<_>>(),
                    Err(e) => return Some(Err(Error::Parse(e.to_string()))),
                },
                IpAddr::V6(_) => match CompactIpv6Addrs::try_from(values.as_slice()) {
                    Ok(addrs) => addrs.into_iter().map(|e| e.into()).collect::<Vec<_>>(),
                    Err(e) => return Some(Err(Error::Parse(e.to_string()))),
                },
            }
        } else {
            vec![]
        };

        self.metrics.discovered_peers.inc_by(peers.len() as u64);
        Some(Ok(Reply::GetPeers(peers)))
    }

    async fn on_error_response(
        &self,
        addr: &SocketAddr,
        key: &TransactionKey,
        message: ErrorMessage,
    ) {
        self.metrics.errors.inc();
        self.node_query_result(&addr, false).await;

        if let Some(pending_request) = self.pending_requests.lock().await.remove(&key) {
            debug!("{} received error for {}", self, key);
            Self::send_reply(pending_request.request_type, Err(Error::from(message)))
        } else {
            warn!(
                "{} received error for unknown request, invalid transaction {}",
                self, key
            );
        }
    }

    /// Process a received tracker command.
    async fn handle_command(&self, command: TrackerCommand, traversal: &mut TraversalAlgorithm) {
        match command {
            TrackerCommand::AddTraversalNode((id, addr)) => traversal.add_node(Some(id), addr),
            TrackerCommand::UpdateExternalIp(ip) => {
                self.update_external_ip(ip).await;
                traversal.restart();
            }
        }
    }

    /// Ping the given node address.
    ///
    /// # Arguments
    ///
    /// * `addr` - the node address to ping.
    /// * `sender` - The result sender for the ping operation.
    pub(crate) async fn ping(&self, addr: &SocketAddr) -> Response<Node> {
        let id = self.routing_table.lock().await.id;
        self.send_query(
            QueryMessage::Ping {
                request: PingMessage { id },
            },
            addr,
            || async {},
            |e| PendingRequestType::Ping(e),
        )
        .await
    }

    /// Find the closest nodes for the given target node id.
    /// This will query all stored nodes within the routing table.
    ///
    /// # Arguments
    ///
    /// * `target_id` - The target node id to retrieve the closest nodes of.
    /// * `timeout` - The timeout of the query for individual nodes.
    async fn find_nodes(&self, target_id: NodeId, timeout: Duration) -> Response<Vec<Node>> {
        let (id, nodes) = {
            let routing_table = self.routing_table.lock().await;
            (
                routing_table.id,
                Self::find_good_search_nodes(&routing_table)
                    .await
                    .cloned()
                    .collect::<Vec<_>>(),
            )
        };

        let futures: Vec<_> = nodes
            .into_iter()
            .map(|node| async move {
                let response = self.find_node(id, target_id, &node).await;
                select! {
                    _ = time::sleep(timeout) => Err(Error::Timeout),
                    e = response => e,
                }
            })
            .collect();

        let (tx, rx) = oneshot::channel();
        let nodes = futures::future::join_all(futures)
            .await
            .into_iter()
            .flat_map(|result| result.ok().unwrap_or(Vec::with_capacity(0)))
            .collect::<Vec<_>>();
        let _ = tx.send(Ok(nodes));

        Response::new(rx)
    }

    /// Find the closest nodes for the given target node id.
    ///
    /// # Arguments
    ///
    /// * `id` - The current id of the node server.
    /// * `target` - The target node id to retrieve the closest nodes of.
    /// * `node` - The node to which the address belongs to, if available.
    pub(crate) async fn find_node(
        &self,
        id: NodeId,
        target: NodeId,
        node: &Node,
    ) -> Response<Vec<Node>> {
        self.send_query(
            QueryMessage::FindNode {
                request: FindNodeRequest {
                    id,
                    target,
                    want: WantFamily::Ipv4 | WantFamily::Ipv6,
                },
            },
            node.addr(),
            || node.failed(),
            |e| PendingRequestType::FindNode(e),
        )
        .await
    }

    async fn update_external_ip(&self, ip: IpAddr) {
        let mut routing_table = self.routing_table.lock().await;
        let new_node_id = NodeId::from_ip(&ip);
        let existing_nodes = routing_table.nodes().cloned().collect::<Vec<_>>();
        let bucket_size = routing_table.bucket_size;

        // replace the routing table
        *routing_table = RoutingTable::new(new_node_id, bucket_size);
        for node in existing_nodes {
            let _ = routing_table.add_node(node).await;
        }

        debug!("{} detected external IP {}", self, ip);
        let _ = self.callbacks.invoke(DhtEvent::ExternalIpChanged(ip));
    }

    /// Get peers for the given torrent info hash.
    ///
    /// # Arguments
    ///
    /// * `info_hash` - The info hash to search peers for.
    /// * `timeout` - The timeout of the query for individual nodes.
    async fn get_peers(&self, info_hash: &InfoHash, timeout: Duration) -> Result<Vec<SocketAddr>> {
        let nodes = {
            let routing_table = self.routing_table.lock().await;
            Self::find_good_search_nodes(&routing_table)
                .await
                .cloned()
                .collect::<Vec<_>>()
        };

        let id = self.routing_table.lock().await.id;
        let futures: Vec<_> = nodes
            .into_iter()
            .map(|node| {
                let info_hash = info_hash.clone();
                async move {
                    let response = self
                        .send_query(
                            QueryMessage::GetPeers {
                                request: GetPeersRequest {
                                    id,
                                    info_hash,
                                    want: WantFamily::none(),
                                },
                            },
                            node.addr(),
                            || node.failed(),
                            |e| PendingRequestType::GetPeers(e),
                        )
                        .await;

                    select! {
                        _ = time::sleep(timeout) => {
                            node.failed().await;
                            Err(Error::Timeout)
                        },
                        result = response => {
                            match result {
                                Ok(e) => Ok(e),
                                Err(_) => {
                                    node.failed().await;
                                    Err(Error::Closed)
                                },
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

    /// Try to send a new query to the given node address.
    /// Returns a [Response] for the send query to the given node address.
    ///
    /// # Arguments
    ///
    /// * `query` - The query to send to the node address.
    /// * `addr` - The address to send the query to.
    /// * `on_failed` - The closure to execute when the query couldn't be sent.
    /// * `on_pending` - The closure mapper to execute when the query is pending.
    async fn send_query<'a, T, F, M>(
        &self,
        query: QueryMessage,
        addr: &SocketAddr,
        on_failed: F,
        on_pending: M,
    ) -> Response<T>
    where
        F: AsyncFnOnce(),
        M: FnOnce(Sender<Result<T>>) -> PendingRequestType,
    {
        // validate the remote node address
        if addr.ip().is_unspecified() || addr.port() == 0 {
            return Response::err(Error::InvalidAddr);
        }

        let name = query.name().to_string();
        let id = self.next_transaction_id();
        let message = match Message::builder()
            .transaction_id(id)
            .payload(MessagePayload::Query(query))
            .build()
        {
            Ok(message) => message,
            Err(e) => return Response::err(e),
        };

        debug!(
            "{} is sending query \"{}\" (transaction {}) to {}",
            self, name, id, addr
        );
        match self.send(message, addr).await {
            Ok(_) => {
                let (tx, rx) = oneshot::channel();
                let mut pending_requests = self.pending_requests.lock().await;
                pending_requests.insert(
                    TransactionKey { id, addr: *addr },
                    PendingRequest {
                        request_type: on_pending(tx),
                        timestamp_sent: Instant::now(),
                    },
                );
                self.metrics
                    .pending_queries
                    .set(pending_requests.len() as u64);
                Response::new(rx)
            }
            Err(e) => {
                on_failed().await;
                Response::err(e)
            }
        }
    }

    /// Send the given response for a query message.
    ///
    /// # Arguments
    ///
    /// * `transaction_id` - The original query transaction id.
    /// * `response` - The response payload.
    /// * `addr` - The node address to send the response to.
    ///
    /// # Returns
    ///
    /// It returns an error if the response failed to send.
    async fn send_response(
        &self,
        transaction_id: u16,
        response: ResponseMessage,
        addr: &SocketAddr,
    ) -> Result<()> {
        let message = Message::builder()
            .transaction_id(transaction_id)
            .version(Version::from(VERSION_IDENTIFIER))
            .payload(MessagePayload::Response(response))
            .ip((*addr).into())
            .port(addr.port())
            .build()?;

        self.send(message, addr).await
    }

    /// Send the given error response for a query message.
    ///
    /// # Arguments
    ///
    /// * `transaction_id` - The original transaction id of the message.
    /// * `error` - The error payload.
    /// * `addr` - The node address to send the response to.
    async fn send_error(
        &self,
        transaction_id: u16,
        error: ErrorMessage,
        addr: &SocketAddr,
    ) -> Result<()> {
        let message = Message::builder()
            .transaction_id(transaction_id)
            .payload(MessagePayload::Error(error))
            .ip((*addr).into())
            .port(addr.port())
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

        self.metrics.bytes_out.inc_by(bytes.len() as u64);
        Ok(())
    }

    /// Update the nodes information of the given node.
    async fn update_node(&self, id: NodeId, addr: SocketAddr) {
        let mut routing_table = self.routing_table.lock().await;
        match routing_table.find_node(&id) {
            Some(node) => {
                node.seen().await;
            }
            None => {
                let node = Node::new(id, addr);
                let event_node = node.clone();

                match routing_table.add_node(node).await {
                    Ok(bucket_index) => {
                        self.metrics.nodes.set(routing_table.len() as u64);
                        debug!(
                            "{} added verified node {} to bucket {}",
                            self,
                            event_node.addr(),
                            bucket_index
                        );

                        let _ = self.command_sender.send(TrackerCommand::AddTraversalNode((
                            *event_node.id(),
                            *event_node.addr(),
                        )));
                        self.callbacks.invoke(DhtEvent::NodeAdded(event_node));
                    }
                    Err(e) => {
                        trace!(
                            "{} failed to add verified node {}, {}",
                            self,
                            event_node.addr(),
                            e
                        );
                    }
                }
            }
        }
    }

    async fn node_query_result(&self, node_addr: &SocketAddr, success: bool) {
        let routing_table = self.routing_table.lock().await;
        if let Some(node) = routing_table.nodes().find(|e| e.addr() == node_addr) {
            if success {
                node.confirmed().await;
            } else {
                node.failed().await;
            }
        };
    }

    async fn stats_tick(&self) {
        self.callbacks
            .invoke(DhtEvent::Stats(self.metrics.snapshot()));

        self.metrics.tick(STATS_INTERVAL);
        self.routing_table.lock().await.tick(STATS_INTERVAL);
    }

    /// Refresh the nodes within the routing table.
    async fn refresh_routing_table(&self) {
        let routing_table = self.routing_table.lock().await;
        let id = routing_table.id;

        trace!("{} is refreshing nodes within routing table", self);
        for bucket in routing_table.buckets() {
            let nodes_last_seen =
                futures::future::join_all(bucket.nodes.iter().map(|e| e.last_seen())).await;

            // rotate all bucket node secret tokens when needed
            futures::future::join_all(bucket.nodes.iter().map(|e| e.rotate_token_secret())).await;

            // check if all nodes within the bucket need to be refreshed
            // Buckets that have not been changed in 15 minutes should be "refreshed."
            if nodes_last_seen
                .into_iter()
                .all(|e| e.elapsed() > REFRESH_TIMEOUT)
            {
                let target_node = bucket.nodes.first().map(|e| *e.id()).unwrap_or(id);
                for node in &bucket.nodes {
                    let _ = self.find_node(id, target_node, node).await;
                }
            }
        }
    }

    /// Cleanup pending requests which have not received a response.
    async fn cleanup_pending_requests(&self) {
        let mut pending_requests = self.pending_requests.lock().await;
        let now = Instant::now();
        let timed_out_request_keys: Vec<_> = pending_requests
            .iter()
            .filter(|(_, request)| now - request.timestamp_sent >= RESPONSE_TIMEOUT)
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
            self.node_query_result(&key.addr, false).await;
            if let Some(request) = pending_requests.remove(&key) {
                Self::pending_request_error(request.request_type, Error::Timeout);
            }
        }

        self.metrics
            .pending_queries
            .set(pending_requests.len() as u64);
    }

    /// Try to update the announce token for the given node ID.
    ///
    /// It returns an error when the node ID couldn't be found within the routing table or the token value is invalid.
    async fn update_announce_token(&self, id: &NodeId, value: impl AsRef<[u8]>) -> Result<()> {
        let token = NodeToken::try_from(value.as_ref())?;
        let routing_table = self.routing_table.lock().await;
        let node = routing_table.find_node(id).ok_or(Error::InvalidNodeId)?;
        node.update_announce_token(token).await;
        Ok(())
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

    /// Returns all non [NodeState::Bad] search nodes from the routing table
    async fn find_good_search_nodes<'a>(
        routing_table: &'a MutexGuard<'_, RoutingTable>,
    ) -> impl Iterator<Item = &'a Node> {
        let nodes_with_state =
            futures::future::join_all(routing_table.nodes().map(|node| async move {
                let state = node.state().await;
                (node, state)
            }))
            .await;

        nodes_with_state.into_iter().flat_map(|(node, state)| {
            if state != NodeState::Bad {
                Some(node)
            } else {
                None
            }
        })
    }

    fn send_reply(pending_request: PendingRequestType, result: Result<Reply>) {
        match pending_request {
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
                // a GetPeers request might sometimes result in a FindNode response, this is due to the remote node not having any peers for the requested hash
                // in such a case, we send an empty reply instead of an error
                let _ = tx.send(Self::map_reply(result, "GetPeers", |r| match r {
                    Reply::FindNode(_) => Some(vec![]),
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

    fn closest_node_pairs(
        routing_table: &RoutingTable,
        target: &NodeId,
    ) -> (CompactIPv4Nodes, CompactIPv6Nodes) {
        let mut compact_nodes = Vec::new();
        let mut compact_nodes6 = Vec::new();
        for node in routing_table.find_bucket_nodes(&target) {
            let addr = node.addr();
            match addr.ip() {
                IpAddr::V4(ip) => {
                    compact_nodes.push(CompactIPv4Node {
                        id: *node.id(),
                        addr: CompactIpv4Addr {
                            ip,
                            port: addr.port(),
                        },
                    });
                }
                IpAddr::V6(ip) => {
                    compact_nodes6.push(CompactIPv6Node {
                        id: *node.id(),
                        addr: CompactIpv6Addr {
                            ip,
                            port: addr.port(),
                        },
                    });
                }
            }
        }
        (compact_nodes.into(), compact_nodes6.into())
    }
}

#[derive(Debug)]
enum ReaderMessage {
    Message {
        message: Message,
        message_len: usize,
        addr: SocketAddr,
    },
    Error {
        error: Error,
        payload_len: usize,
        addr: SocketAddr,
    },
}

#[derive(Debug, Display)]
#[display(fmt = "DHT node reader [{}]", socket_addr)]
struct NodeReader {
    socket: Arc<UdpSocket>,
    socket_addr: SocketAddr,
    sender: UnboundedSender<ReaderMessage>,
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
                        let _ = self.sender.send(ReaderMessage::Error { error: e, payload_len: len, addr });
                    }
                },
            }
        }
        debug!("{} main loop ended", self);
    }

    async fn handle_incoming_message(&self, bytes: &[u8], addr: SocketAddr) -> Result<()> {
        // check if the port of the sender is known
        if addr.port() == 0 {
            trace!(
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

        let message_len = bytes.len();
        let _ = self.sender.send(ReaderMessage::Message {
            message,
            message_len,
            addr,
        });
        Ok(())
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

/// Receives a response message from a [Node].
/// As the response is a [Future], `.await` the response to get the response sent by the [Node].
///
/// All responses will result in either the expected `<T>` or an [Error].
#[derive(Debug)]
pub struct Response<T> {
    inner: oneshot::Receiver<Result<T>>,
}

impl<T> Response<T> {
    /// Creates a new [`Response`] from an existing oneshot receiver.
    ///
    /// Normally you won't need to call this directly; it is used internally
    /// by whatever component sends requests to the [`Node`].
    pub(crate) fn new(inner: oneshot::Receiver<Result<T>>) -> Self {
        Self { inner }
    }

    pub(crate) fn err(e: Error) -> Self {
        let (tx, rx) = oneshot::channel();
        let _ = tx.send(Err(e));
        Self { inner: rx }
    }
}

impl<T> Future for Response<T> {
    type Output = Result<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        Pin::new(&mut this.inner).poll(cx).map(|e| {
            e.map_err(|err| Error::Io(io::Error::new(io::ErrorKind::Interrupted, err)))
                .flatten()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::create_node_server_pair;
    use crate::{init_logger, timeout};
    use std::net::Ipv4Addr;

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
            let result = tracker.id().await;
            assert_eq!(
                node_id, result,
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

    mod ping {
        use super::*;

        #[tokio::test]
        async fn test_ping_valid_address() {
            init_logger!();
            let (incoming, outgoing) = create_node_server_pair!();

            let result = timeout!(
                outgoing.ping((Ipv4Addr::LOCALHOST, incoming.port()).into()),
                Duration::from_millis(750),
                "failed to ping node"
            );
            assert_eq!(Ok(()), result);

            // check if the incoming server has added the node that pinged it
            {
                let routing_table = incoming.inner.routing_table.lock().await;
                let result = routing_table.find_node(&outgoing.id().await);
                assert_ne!(
                    None, result,
                    "expected the incoming ping node to have been added to the routing table"
                );
            }

            // check if the outgoing server has added the pinged target node
            {
                let routing_table = outgoing.inner.routing_table.lock().await;
                let result = routing_table.find_node(&incoming.id().await);
                assert_ne!(
                    None, result,
                    "expected the pinged target node to have been added to the outgoing server"
                );
            }
        }

        #[tokio::test]
        async fn test_ping_invalid_address() {
            init_logger!();
            let addr = SocketAddr::from(([0, 0, 0, 0], 9000));
            let tracker = DhtTracker::new(NodeId::new(), Vec::new()).await.unwrap();

            let result = timeout!(
                tracker.ping(addr),
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
            let node_incoming_id = NodeId::from_ip_with_rand(&Ipv4Addr::LOCALHOST.into(), rand);
            let node_outgoing_id = NodeId::from_ip_with_rand(&Ipv4Addr::LOCALHOST.into(), rand);
            let (incoming, outgoing) = create_node_server_pair!(node_incoming_id, node_outgoing_id);

            // register the incoming tracker with the outgoing tracker
            let incoming_node_id = incoming.id().await;
            outgoing
                .inner
                .update_node(
                    incoming_node_id,
                    (Ipv4Addr::LOCALHOST, incoming.port()).into(),
                )
                .await;

            // calculate the bucket which will be retrieved by the search node
            let bucket_index = node_incoming_id.distance(&search_node_id);
            // create a node which matches the search bucket index
            let nearby_node = create_bucket_matching_node(bucket_index, node_incoming_id);
            incoming
                .inner
                .update_node(nearby_node, ([132, 141, 45, 30], 8090).into())
                .await;

            // request the node info from the nearby node
            let result = outgoing
                .find_nodes(search_node_id, Duration::from_millis(500))
                .await
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
            let incoming_node_id = incoming.id().await;
            outgoing
                .inner
                .update_node(
                    incoming_node_id,
                    (Ipv4Addr::LOCALHOST, incoming.port()).into(),
                )
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

        #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
        async fn test_bootstrap_nodes() {
            init_logger!();
            let rand = 13;
            let node_id = NodeId::from_ip_with_rand(&[141, 130, 12, 89].into(), rand);
            let bootstrap_node_id = NodeId::from_ip_with_rand(&[180, 13, 0, 3].into(), rand);
            let bootstrap_node = DhtTracker::builder()
                .node_id(bootstrap_node_id)
                .build()
                .await
                .unwrap();

            // fill the bootstrap node with nodes which can be found through the `find_node` search
            let futures = (1..111u8)
                .into_iter()
                .map(|e| async move {
                    DhtTracker::builder()
                        .node_id(NodeId::from_ip_with_rand(
                            &IpAddr::V4(Ipv4Addr::new(127, 0, 0, e)),
                            rand,
                        ))
                        .build()
                        .await
                        .unwrap()
                })
                .collect::<Vec<_>>();
            let nodes = futures::future::join_all(futures).await;

            for node in &nodes {
                let node_id = node.id().await;
                bootstrap_node
                    .inner
                    .update_node(node_id, (Ipv4Addr::LOCALHOST, node.port()).into())
                    .await;
            }

            // create the DHT tracker which will use the bootstrap node for its bootstrap process
            let tracker = DhtTracker::builder()
                .node_id(node_id)
                .routing_nodes(vec![(Ipv4Addr::LOCALHOST, bootstrap_node.port()).into()])
                .build()
                .await
                .expect("expected a new DHT tracker to have been created");

            select! {
                _ = time::sleep(Duration::from_secs(10)) => assert!(false, "timed-out while bootstrapping nodes"),
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
