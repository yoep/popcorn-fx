use crate::torrent::dht::{Error, NodeId, Result};
use rand::{rng, Rng};
use sha1::{Digest, Sha1};
use std::net::{IpAddr, SocketAddr};
use std::time::Instant;

const TOKEN_SECRET_SIZE: usize = 20;
const TOKEN_SIZE: usize = 4;

/// The node information within the DHT network
#[derive(Debug, Clone)]
pub struct Node {
    /// The unique ID of the node
    pub id: NodeId,
    /// The address of the node within the DHT network
    pub addr: SocketAddr,
    /// The unique token of the node
    pub token: Token,
    /// The token to use for announcing a peer
    pub announce_token: Option<Token>,
    /// The current state of the node
    pub state: NodeState,
}

impl Node {
    /// Create a new node for the given ID and address.
    pub fn new(id: NodeId, addr: SocketAddr) -> Self {
        Self {
            id,
            addr,
            token: Token::new(),
            announce_token: None,
            state: NodeState::Good,
        }
    }

    /// Generate a new token for the given peer address.
    pub fn generate_token(&self, addr: IpAddr) -> [u8; TOKEN_SIZE] {
        self.token.generate_token(addr)
    }

    /// Update the opaque token for this node.
    pub fn update_announce_token(&mut self, token: Token) {
        self.announce_token = Some(token);
    }

    /// Get the distance between this node and the target node.
    /// See [NodeId::distance] for more information.
    pub fn distance(&self, node: &Node) -> u8 {
        self.id.distance(&node.id)
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

/// The token information of a node.
#[derive(Debug, Clone)]
pub struct Token {
    secret: [u8; TOKEN_SECRET_SIZE],
    old_secret: [u8; TOKEN_SECRET_SIZE],
    last_refreshed: Instant,
}

impl Token {
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

impl TryFrom<&[u8]> for Token {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        if value.len() != TOKEN_SIZE {
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
            let token = "aoeusnth".as_bytes();

            let result = Token::try_from(token).expect("expected the token value to be valid");

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
}
