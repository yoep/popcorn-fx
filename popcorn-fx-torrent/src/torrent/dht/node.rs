use crate::torrent::dht::{Error, NodeId, NodeMetrics, Result};
use rand::{rng, Rng};
use sha1::{Digest, Sha1};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};

const TOKEN_SECRET_SIZE: usize = 20;
const TOKEN_SIZE: usize = 4;
const QUESTIONABLE_NODE_AFTER: Duration = Duration::from_secs(15 * 60); // 15 mins.
const BAD_NODE_AFTER_TIMEOUTS: usize = 5;
const BAD_NODE_ERROR_RATE_THRESHOLD: usize = 2;
const TOKEN_SECRET_REFRESH: Duration = Duration::from_secs(60 * 5); // 5 mins.

/// The opaque token secret value used within the hashing process.
type TokenSecretValue = [u8; TOKEN_SECRET_SIZE];

/// The announce token for a node.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct NodeToken([u8; TOKEN_SIZE]);

impl NodeToken {
    /// Copy the token bytes into a new buffer.
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    fn from<T: AsRef<[u8]>>(value: T) -> Self {
        let mut token = [0u8; TOKEN_SIZE];
        token.copy_from_slice(&value.as_ref()[0..TOKEN_SIZE]);
        Self(token)
    }
}

impl TryFrom<&[u8]> for NodeToken {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        if value.len() != TOKEN_SIZE {
            return Err(Error::InvalidToken);
        }

        Ok(Self::from(value))
    }
}

/// The node information within the DHT network
#[derive(Debug, Clone)]
pub struct Node {
    inner: Arc<InnerNode>,
}

impl Node {
    /// Create a new node for the given ID and address.
    pub fn new(id: NodeId, addr: SocketAddr) -> Self {
        Self::new_with_state(id, addr, NodeState::Good)
    }

    /// Create a new node for the given ID and address with the given state.
    pub(crate) fn new_with_state(id: NodeId, addr: SocketAddr, state: NodeState) -> Self {
        Self {
            inner: Arc::new(InnerNode {
                id,
                addr,
                token: RwLock::new(TokenSecret::new()),
                announce_token: Mutex::new(None),
                state: Mutex::new(state),
                last_seen: Mutex::new(Instant::now()),
                metrics: NodeMetrics::new(),
            }),
        }
    }

    /// Returns a reference to the id of this node.
    pub fn id(&self) -> &NodeId {
        &self.inner.id
    }

    /// Returns a reference to the address of this node.
    pub fn addr(&self) -> &SocketAddr {
        &self.inner.addr
    }

    /// Returns the metrics of this node.
    pub fn metrics(&self) -> &NodeMetrics {
        &self.inner.metrics
    }

    /// Returns the current state of this node.
    pub async fn state(&self) -> NodeState {
        *self.inner.state.lock().await
    }

    /// Returns the last time we received a message from this node.
    pub async fn last_seen(&self) -> Instant {
        *self.inner.last_seen.lock().await
    }

    /// Verify that the given token is valid for this node.
    pub(crate) async fn verify_token(&self, token: &NodeToken, ip: &IpAddr) -> bool {
        self.inner.token.read().await.verify(token, ip)
    }

    /// Generate a new secret token for announcing peers.
    /// This token is always based on the ip of the node.
    pub(crate) async fn generate_token(&self) -> NodeToken {
        self.inner
            .token
            .read()
            .await
            .generate(&self.inner.addr.ip())
    }

    /// Rotate the token secret for this node, if needed.
    /// This is done every 5 minutes
    pub(crate) async fn rotate_token_secret(&self) {
        let mut token_secret = self.inner.token.write().await;

        if token_secret.needs_rotation() {
            token_secret.rotate();
        }
    }

    /// Update the opaque token for this node.
    pub(crate) async fn update_announce_token(&self, token: NodeToken) {
        self.inner.update_announce_token(token).await;
    }

    /// The node has successfully responded to a query.
    pub(crate) async fn confirmed(&self) {
        self.inner.confirmed().await;
    }

    /// The node has sent a query message.
    pub(crate) async fn seen(&self) {
        self.inner.update_last_seen().await;
    }

    /// Increase the number of times the node failed to respond to a query.
    pub(crate) async fn failed(&self) {
        self.inner.failed().await;
    }

    /// Get the distance between this node and the target node.
    /// See [NodeId::distance] for more information.
    pub fn distance(&self, node: &Node) -> u8 {
        self.inner.id.distance(&node.inner.id)
    }

    /// Check if the [NodeId] is valid for its own ip address.
    /// See BEP42 for more info.
    pub fn is_secure(&self) -> bool {
        self.inner.id.verify_id(&self.inner.addr.ip())
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

#[derive(Debug)]
struct InnerNode {
    /// The unique ID of the node
    id: NodeId,
    /// The address of the node within the DHT network
    addr: SocketAddr,
    /// The unique token of the node
    token: RwLock<TokenSecret>,
    /// The token to use for announcing a peer
    announce_token: Mutex<Option<NodeToken>>,
    /// The current state of the node
    state: Mutex<NodeState>,
    /// The last time we received a message from the node
    last_seen: Mutex<Instant>,
    /// The metrics of the node
    metrics: NodeMetrics,
}

impl InnerNode {
    async fn confirmed(&self) {
        self.update_last_seen().await;
        self.metrics.confirmed_queries.inc();
        self.update_state(NodeState::Good).await;
    }

    async fn failed(&self) {
        self.metrics.errors.inc();

        let last_seen = *self.last_seen.lock().await;
        let new_state = NodeState::calculate(Instant::now() - last_seen, &self.metrics);
        self.update_state(new_state).await;
    }

    async fn update_last_seen(&self) {
        let now = Instant::now();
        let mut last_seen = self.last_seen.lock().await;
        *last_seen = now;
    }

    async fn update_announce_token(&self, token: NodeToken) {
        let mut announce_token = self.announce_token.lock().await;
        *announce_token = Some(token);
    }

    async fn update_state(&self, new_state: NodeState) {
        let mut state = self.state.lock().await;
        *state = new_state;
    }
}

impl PartialEq for InnerNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.addr == other.addr
    }
}

/// The state of a node
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NodeState {
    Good = 0,
    Questionable = 1,
    Bad = 2,
}

impl NodeState {
    /// Calculate the node state from the given metrics.
    ///
    /// A good node is a node has responded to one of our queries within the last 15 minutes.
    /// After 15 minutes of inactivity, a node becomes questionable.
    /// Nodes become bad when they fail to respond to multiple queries in a row.
    pub(crate) fn calculate(last_seen_since: Duration, metrics: &NodeMetrics) -> Self {
        let total_queries = metrics.confirmed_queries.total();
        let total_errors = metrics.errors.total();

        // if we've never had successful query response and X timeouts,
        // the node is considered bad
        if total_queries == 0 && total_errors as usize > BAD_NODE_AFTER_TIMEOUTS {
            return Self::Bad;
        }

        // if the error rate (5s avg success rate - 5s avg timeout rate) also exceeds the threshold,
        // the node is considered bad
        let error_rate = metrics
            .confirmed_queries
            .rate()
            .saturating_sub(metrics.errors.rate());
        if error_rate as usize > BAD_NODE_ERROR_RATE_THRESHOLD {
            return Self::Bad;
        }

        if last_seen_since < QUESTIONABLE_NODE_AFTER {
            return Self::Good;
        }

        Self::Questionable
    }
}

/// The token information of a node.
#[derive(Debug, Clone)]
struct TokenSecret {
    secret: TokenSecretValue,
    old_secret: TokenSecretValue,
    last_refreshed: Instant,
}

impl TokenSecret {
    fn new() -> Self {
        let mut random = rng();
        Self {
            secret: random.random(),
            old_secret: random.random(),
            last_refreshed: Instant::now(),
        }
    }

    fn verify(&self, token: &NodeToken, addr: &IpAddr) -> bool {
        Self::verify_with(self, token, addr, &self.secret)
            || Self::verify_with(self, token, addr, &self.old_secret)
    }

    fn generate(&self, addr: &IpAddr) -> NodeToken {
        let hash = Self::hash(&self.secret, addr);
        NodeToken::from(hash.as_slice())
    }

    /// Rotate the token secret.
    fn rotate(&mut self) {
        self.old_secret = self.secret;
        self.secret = rng().random();
        self.last_refreshed = Instant::now();
    }

    /// Verify if the token secret needs to be rotated.
    /// This is done every 5 minutes.
    fn needs_rotation(&self) -> bool {
        self.last_refreshed.elapsed() > TOKEN_SECRET_REFRESH
    }

    fn verify_with(&self, token: &NodeToken, addr: &IpAddr, secret: &TokenSecretValue) -> bool {
        let hash = Self::hash(secret, &addr);
        let validation_token = NodeToken::from(hash.as_slice());
        token == &validation_token
    }

    fn hash(secret: &TokenSecretValue, addr: &IpAddr) -> Vec<u8> {
        let mut hasher = Sha1::new();
        hasher.update(addr.to_string().as_bytes());
        hasher.update(secret);
        hasher.finalize().to_vec()
    }
}

impl TryFrom<&[u8]> for TokenSecret {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        if value.len() != TOKEN_SECRET_SIZE {
            return Err(Error::InvalidToken);
        }

        let secret: [u8; TOKEN_SECRET_SIZE] = value.try_into().map_err(|_| Error::InvalidToken)?;

        Ok(Self {
            secret,
            old_secret: secret,
            last_refreshed: Instant::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    mod token {
        use super::*;

        #[test]
        fn test_verify() {
            let ip = IpAddr::V4(Ipv4Addr::new(190, 180, 170, 5));
            let mut token_secret = TokenSecret::new();
            let token = token_secret.generate(&ip);

            let result = token_secret.verify(&token, &ip);
            assert!(result, "expected the token to be valid");

            // rotate the secret
            token_secret.rotate();
            let result = token_secret.verify(&token, &ip);
            assert!(
                result,
                "expected the old secret token to be valid after rotation"
            );

            // rotate the secret a 2nd time
            token_secret.rotate();
            let result = token_secret.verify(&token, &ip);
            assert!(
                !result,
                "expected the old secret token to be invalid after 2nd rotation"
            );
        }

        #[test]
        fn test_generate() {
            let ip = IpAddr::V4(Ipv4Addr::new(120, 188, 12, 1));
            let token_secret = TokenSecret::new();

            let result = token_secret.generate(&ip);

            assert!(
                !result.0.iter().all(|e| *e == 0),
                "expected the token to be non-zero"
            );
        }

        #[test]
        fn test_from_byte_slice() {
            let token = "aoeusnthaoeusnthaoeu".as_bytes();

            let result =
                TokenSecret::try_from(token).expect("expected the token value to be valid");

            assert_eq!(
                result.secret,
                token[..token.len()],
                "expected the token secret to match the parsed value"
            );
            assert_eq!(
                result.old_secret,
                token[..token.len()],
                "expected the old secret to match the parsed value"
            );
        }
    }

    mod node_state {
        use super::*;
        use crate::torrent::metrics::Metric;

        #[test]
        fn test_calculate_good_state() {
            let metrics = NodeMetrics::new();
            metrics.confirmed_queries.inc();
            metrics.tick(Duration::from_secs(1));
            let result = NodeState::calculate(Duration::from_secs(3 * 60), &metrics);
            assert_eq!(NodeState::Good, result);

            let metrics = NodeMetrics::new();
            metrics.errors.inc_by(2);
            metrics.tick(Duration::from_secs(1));
            let result = NodeState::calculate(Duration::from_secs(10 * 60), &metrics);
            assert_eq!(NodeState::Good, result);
        }

        #[test]
        fn test_calculate_questionable_state() {
            let metrics = NodeMetrics::new();
            let result = NodeState::calculate(Duration::from_secs(15 * 60), &metrics);
            assert_eq!(NodeState::Questionable, result);

            let metrics = NodeMetrics::new();
            let result = NodeState::calculate(Duration::from_secs(16 * 60), &metrics);
            assert_eq!(NodeState::Questionable, result);
        }

        #[test]
        fn test_calculate_bad_state() {
            let metrics = NodeMetrics::new();
            metrics.errors.inc_by(6);
            metrics.tick(Duration::from_secs(1));
            let result = NodeState::calculate(Duration::from_secs(5 * 60), &metrics);
            assert_eq!(NodeState::Bad, result);
        }
    }
}
