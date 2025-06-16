use crate::torrent::dht::{Error, NodeId, Result};
use crate::torrent::{CompactIpAddr, InfoHash};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_bytes::ByteBuf;
use std::collections::HashMap;
use std::result;

/// The unique transaction ID of a message.
pub type TransactionId = [u8; 2];

/// The query request message.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "q")]
pub enum QueryMessage {
    #[serde(rename = "ping")]
    Ping {
        #[serde(rename = "a")]
        request: PingMessage,
    },
    #[serde(rename = "find_node")]
    FindNode {
        #[serde(rename = "a")]
        request: FindNodeRequest,
    },
    #[serde(rename = "get_peers")]
    GetPeers,
    #[serde(rename = "announce_peer")]
    AnnouncePeer,
}

impl QueryMessage {
    /// Get the name/type of the query message.
    pub fn name(&self) -> &str {
        match self {
            QueryMessage::Ping { .. } => "ping",
            QueryMessage::FindNode { .. } => "find_node",
            QueryMessage::GetPeers => "get_peers",
            QueryMessage::AnnouncePeer => "announce_peer",
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)] // as we don't have a specific type defined in response, we need to keep in mind the order of the values within this enum
pub enum ResponseMessage {
    GetPeers {
        #[serde(rename = "r")]
        response: GetPeersResponse,
    },
    FindNode {
        #[serde(rename = "r")]
        response: FindNodeResponse,
    },
    Ping {
        #[serde(rename = "r")]
        response: PingMessage,
    },
}

impl ResponseMessage {
    /// Get the name/type of the response message.
    pub fn name(&self) -> &str {
        match self {
            ResponseMessage::GetPeers { .. } => "get_peers",
            ResponseMessage::FindNode { .. } => "find_node",
            ResponseMessage::Ping { .. } => "ping",
        }
    }
}

/// The request- & response message of a ping query.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PingMessage {
    pub id: NodeId,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FindNodeRequest {
    pub id: NodeId,
    pub target: NodeId,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FindNodeResponse {
    /// The id of the node that was queried.
    pub id: NodeId,
    /// The compact node info or the closest good nodes.
    #[serde(with = "serde_bytes")]
    pub nodes: Vec<u8>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GetPeersRequest {
    pub id: NodeId,
    /// The info hash of the torrent to retrieve peers for.
    pub info_hash: InfoHash,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GetPeersResponse {
    pub id: NodeId,
    #[serde(with = "serde_bytes")]
    pub token: Vec<u8>,
    #[serde(default)]
    pub values: Option<Vec<ByteBuf>>,
    #[serde(default, with = "serde_bytes")]
    pub nodes: Option<Vec<u8>>,
}

/// The error message.
#[derive(Debug, PartialEq)]
pub enum ErrorMessage {
    Generic(String),
    Server(String),
    Protocol(String),
    Method(String),
}

impl ErrorMessage {
    /// Get the error code of the error message.
    /// See BEP5 for more info about codes.
    pub fn code(&self) -> u16 {
        match self {
            ErrorMessage::Generic(_) => 201,
            ErrorMessage::Server(_) => 202,
            ErrorMessage::Protocol(_) => 203,
            ErrorMessage::Method(_) => 204,
        }
    }

    /// Get the error description of the error message.
    pub fn description(&self) -> &str {
        match self {
            ErrorMessage::Generic(msg) => msg.as_str(),
            ErrorMessage::Server(msg) => msg.as_str(),
            ErrorMessage::Protocol(msg) => msg.as_str(),
            ErrorMessage::Method(msg) => msg.as_str(),
        }
    }
}

impl Serialize for ErrorMessage {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = (self.code(), self.description());

        HashMap::from([("e".to_string(), value)]).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ErrorMessage {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map = HashMap::<String, (u16, String)>::deserialize(deserializer)?;

        if let Some((_, (code, msg))) = map.into_iter().next() {
            match code {
                201 => Ok(ErrorMessage::Generic(msg)),
                202 => Ok(ErrorMessage::Server(msg)),
                203 => Ok(ErrorMessage::Protocol(msg)),
                404 => Ok(ErrorMessage::Method(msg)),
                _ => Err(de::Error::custom(format!("Unknown error code {}", code))),
            }
        } else {
            Err(de::Error::custom("expected an error key pair"))
        }
    }
}

/// The payload data of a message.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "y")]
pub enum MessagePayload {
    #[serde(rename = "q")]
    Query(QueryMessage),
    #[serde(rename = "r")]
    Response(ResponseMessage),
    #[serde(rename = "e")]
    Error(ErrorMessage),
}

impl MessagePayload {
    /// Get the payload type of the message.
    pub fn payload_type(&self) -> &str {
        match &self {
            MessagePayload::Query(_) => "query",
            MessagePayload::Response(_) => "response",
            MessagePayload::Error(_) => "error",
        }
    }
}

/// The KRPC message communication between nodes.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Message {
    #[serde(rename = "t", with = "serde_bytes")]
    pub transaction_id_bytes: TransactionId,
    #[serde(rename = "v")]
    pub version: Option<String>,
    #[serde(flatten)]
    pub payload: MessagePayload,
    /// The node's external IP.
    /// See BEP42 for more info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip: Option<CompactIpAddr>,
    /// The node's external port
    #[serde(default, skip_serializing_if = "Option::is_none", with = "serde_bytes")]
    pub port: Option<[u8; 2]>, // this field is present in libtorrent, but not documented in a BEP
}

impl Message {
    /// Get a builder instance to create a new message.
    pub fn builder() -> MessageBuilder {
        MessageBuilder::new()
    }

    /// Get the [u16] representation of the transaction ID.
    pub fn transaction_id(&self) -> u16 {
        u16::from_be_bytes(self.transaction_id_bytes)
    }
}

#[derive(Debug, Default)]
pub(crate) struct MessageBuilder {
    transaction_id: Option<Vec<u8>>,
    version: Option<String>,
    payload: Option<MessagePayload>,
    ip: Option<CompactIpAddr>,
    port: Option<[u8; 2]>,
}

impl MessageBuilder {
    /// Create a new instance of the message builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the transaction of the message.
    pub fn transaction_id(&mut self, id: u16) -> &mut Self {
        self.transaction_id = Some(id.to_be_bytes().to_vec());
        self
    }

    /// Set the transaction id of the message from the given string.
    pub fn transaction_id_str<S: AsRef<str>>(&mut self, id: S) -> &mut Self {
        self.transaction_id = Some(id.as_ref().as_bytes().to_vec());
        self
    }

    /// Set the version of the message.
    pub fn version(&mut self, version: String) -> &mut Self {
        self.version = Some(version);
        self
    }

    /// Set the payload data of the message.
    pub fn payload(&mut self, payload: MessagePayload) -> &mut Self {
        self.payload = Some(payload);
        self
    }

    /// Set the node's external compact IP address.
    pub fn ip(&mut self, ip: CompactIpAddr) -> &mut Self {
        self.ip = Some(ip);
        self
    }

    /// Set the node's external port.
    pub fn port(&mut self, port: [u8; 2]) -> &mut Self {
        self.port = Some(port);
        self
    }

    /// Finalize the builder and try to create a new message.
    ///
    /// The transaction ID and message type are required fields.
    /// When one of the required fields was not provided, it will return an error.
    pub fn build(&mut self) -> Result<Message> {
        let transaction_id_value = self
            .transaction_id
            .take()
            .ok_or(Error::InvalidMessage("missing transaction id".to_string()))?;

        Ok(Message {
            transaction_id_bytes: transaction_id_value
                .as_slice()
                .try_into()
                .map_err(|_| Error::InvalidTransactionId)?,
            version: self.version.take(),
            payload: self
                .payload
                .take()
                .ok_or(Error::InvalidMessage("missing payload".to_string()))?,
            ip: self.ip.take(),
            port: self.port.take(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_ping_request_deserialize() {
        let payload = "d1:ad2:id20:abcdefghij0123456789e1:q4:ping1:t2:aa1:y1:qe";
        let node_id = NodeId::try_from("abcdefghij0123456789".as_bytes()).unwrap();
        let expected_result = Message::builder()
            .transaction_id_str("aa")
            .payload(MessagePayload::Query(QueryMessage::Ping {
                request: PingMessage { id: node_id },
            }))
            .build()
            .unwrap();

        let result = serde_bencode::from_str::<Message>(payload).expect("expected a valid message");

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_message_ping_response_deserialize() {
        let payload = "d1:rd2:id20:mnopqrstuvwxyz123456e1:t2:aa1:y1:re";
        let node_id = NodeId::try_from("mnopqrstuvwxyz123456".as_bytes()).unwrap();
        let expected_result = Message::builder()
            .transaction_id_str("aa")
            .payload(MessagePayload::Response(ResponseMessage::Ping {
                response: PingMessage { id: node_id },
            }))
            .build()
            .unwrap();

        let result = serde_bencode::from_str::<Message>(payload).expect("expected a valid message");

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_message_error_deserialize() {
        let payload = "d1:eli201e23:A Generic Error Ocurrede1:t2:aa1:y1:ee";
        let expected_result = Message::builder()
            .transaction_id_str("aa")
            .payload(MessagePayload::Error(ErrorMessage::Generic(
                "A Generic Error Ocurred".to_string(),
            )))
            .build()
            .unwrap();

        let result = serde_bencode::from_str::<Message>(payload).expect("expected a valid message");

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_message_error_serialize() {
        let message = Message::builder()
            .transaction_id_str("aa")
            .payload(MessagePayload::Error(ErrorMessage::Generic(
                "A Generic Error Occurred".to_string(),
            )))
            .build()
            .unwrap();
        let expected_result = "d1:eli201e24:A Generic Error Occurrede1:t2:aa1:y1:ee";

        let result = serde_bencode::to_string(&message).unwrap();

        assert_eq!(expected_result, result.as_str());
    }
}
