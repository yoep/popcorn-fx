use crate::torrent::dht::{Error, NodeId, Result};
use rand::{rng, Rng};
use sha1::{Digest, Sha1};
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};

const TOKEN_SECRET_SIZE: usize = 20;
const TOKEN_SIZE: usize = 4;
const QUESTIONABLE_NODE_AFTER: Duration = Duration::from_secs(15 * 60); // 15 mins.
const BAD_NODE_AFTER_TIMEOUTS: usize = 5;

/// A node token type alias.
pub type Token = [u8; TOKEN_SIZE];

/// The node information within the DHT network
#[derive(Debug, Clone)]
pub struct Node {
    /// The unique ID of the node
    pub id: NodeId,
    /// The address of the node within the DHT network
    pub addr: SocketAddr,
    /// The unique token of the node
    pub token: TokenSecret,
    /// The token to use for announcing a peer
    pub announce_token: Option<TokenSecret>,
    /// The current state of the node
    pub state: NodeState,
    /// The last time we received a message from the node
    pub last_seen: Instant,
    /// The number of times the node failed to respond to a query in a row
    pub timeout_count: usize,
}

impl Node {
    /// Create a new node for the given ID and address.
    pub fn new(id: NodeId, addr: SocketAddr) -> Self {
        Self {
            id,
            addr,
            token: TokenSecret::new(),
            announce_token: None,
            state: NodeState::Good,
            last_seen: Instant::now(),
            timeout_count: 0,
        }
    }

    /// Generate a new token for the given peer address.
    pub fn generate_token(&self, addr: IpAddr) -> Token {
        self.token.generate_token(addr)
    }

    /// Update the opaque token for this node.
    pub fn update_announce_token(&mut self, token: TokenSecret) {
        self.announce_token = Some(token);
    }

    /// Update the state of the node.
    pub fn update_state(&mut self, state: NodeState) {
        self.state = state;
    }

    /// The node has successfully responded to a query.
    pub fn confirmed(&mut self) {
        self.last_seen = Instant::now();
        self.timeout_count = 0;
        self.state = NodeState::Good;
    }

    /// Increase the number of times the node failed to respond to a query.
    pub fn timed_out(&mut self) {
        self.timeout_count += 1;
        self.state = NodeState::calculate(Instant::now() - self.last_seen, self.timeout_count);
    }

    /// Get the distance between this node and the target node.
    /// See [NodeId::distance] for more information.
    pub fn distance(&self, node: &Node) -> u8 {
        self.id.distance(&node.id)
    }

    /// Check if the [NodeId] is valid for its own ip address.
    /// See BEP42 for more info.
    pub fn is_secure(&self) -> bool {
        self.id.verify_id(&self.addr.ip())
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
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
    pub fn calculate(last_seen_since: Duration, timeout_count: usize) -> Self {
        if timeout_count > BAD_NODE_AFTER_TIMEOUTS {
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
pub struct TokenSecret {
    secret: [u8; TOKEN_SECRET_SIZE],
    old_secret: [u8; TOKEN_SECRET_SIZE],
    last_refreshed: Instant,
}

impl TokenSecret {
    pub fn new() -> Self {
        let mut random = rng();
        Self {
            secret: random.random(),
            old_secret: random.random(),
            last_refreshed: Instant::now(),
        }
    }

    /// Generate a new token for the given peer address.
    pub fn generate_token(&self, addr: IpAddr) -> [u8; TOKEN_SIZE] {
        let mut hasher = Sha1::new();

        hasher.update(addr.to_string().as_bytes());
        hasher.update(&self.secret);

        let hash = hasher.finalize();
        let mut token = [0u8; TOKEN_SIZE];
        token.copy_from_slice(&hash[0..TOKEN_SIZE]);

        token
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

    #[test]
    fn test_node_generate_token() {
        let id = NodeId::new();
        let node_addr: SocketAddr = ([127, 0, 0, 1], 8080).into();
        let peer_addr: IpAddr = IpAddr::V4([92, 123, 0, 1].into());
        let node = Node::new(id, node_addr);

        let result = node.generate_token(peer_addr);

        assert_ne!([0u8; TOKEN_SIZE], result);
    }

    mod token {
        use super::*;

        #[test]
        fn test_token_from_byte_slice() {
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

        #[test]
        fn test_calculate_good_state() {
            let result = NodeState::calculate(Duration::from_secs(3 * 60), 0);
            assert_eq!(NodeState::Good, result);

            let result = NodeState::calculate(Duration::from_secs(10 * 60), 2);
            assert_eq!(NodeState::Good, result);
        }

        #[test]
        fn test_calculate_questionable_state() {
            let result = NodeState::calculate(Duration::from_secs(10 * 60), 6);
            assert_eq!(NodeState::Questionable, result);
        }

        #[test]
        fn test_calculate_bad_state() {
            let result = NodeState::calculate(Duration::from_secs(16 * 60), 0);
            assert_eq!(NodeState::Bad, result);
        }
    }
}
