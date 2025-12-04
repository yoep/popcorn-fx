use crate::torrent::dht::compact::{CompactIPv4Nodes, CompactIPv6Nodes};
use crate::torrent::dht::{Error, NodeId, Result};
use crate::torrent::{CompactIpAddr, InfoHash};
use bitmask_enum::bitmask;
use serde::de::SeqAccess;
use serde::ser::SerializeSeq;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::result;
use std::str::FromStr;

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
    GetPeers {
        #[serde(rename = "a")]
        request: GetPeersRequest,
    },
    #[serde(rename = "announce_peer")]
    AnnouncePeer {
        #[serde(rename = "a")]
        request: AnnouncePeerRequest,
    },
}

impl QueryMessage {
    /// Returns the node ID of the sender.
    pub fn id(&self) -> &NodeId {
        match self {
            QueryMessage::Ping { request } => &request.id,
            QueryMessage::FindNode { request } => &request.id,
            QueryMessage::GetPeers { request } => &request.id,
            QueryMessage::AnnouncePeer { request } => &request.id,
        }
    }

    /// Returns the name/type of the query message.
    pub fn name(&self) -> &str {
        match self {
            QueryMessage::Ping { .. } => "ping",
            QueryMessage::FindNode { .. } => "find_node",
            QueryMessage::GetPeers { .. } => "get_peers",
            QueryMessage::AnnouncePeer { .. } => "announce_peer",
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
    Announce {
        #[serde(rename = "r")]
        response: AnnouncePeerResponse,
    },
}

impl ResponseMessage {
    /// Returns the node ID of the sender.
    pub fn id(&self) -> &NodeId {
        match self {
            ResponseMessage::GetPeers { response } => &response.id,
            ResponseMessage::FindNode { response } => &response.id,
            ResponseMessage::Ping { response } => &response.id,
            ResponseMessage::Announce { response } => &response.id,
        }
    }

    /// Returns the name/type of the response message.
    pub fn name(&self) -> &str {
        match self {
            ResponseMessage::GetPeers { .. } => "get_peers",
            ResponseMessage::FindNode { .. } => "find_node",
            ResponseMessage::Ping { .. } => "ping",
            ResponseMessage::Announce { .. } => "announce_peer",
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
    #[serde(default, skip_serializing_if = "WantFamily::is_none")]
    pub want: WantFamily,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FindNodeResponse {
    /// The id of the node that was queried.
    pub id: NodeId,
    #[serde(skip_serializing_if = "CompactIPv4Nodes::is_empty")]
    pub nodes: CompactIPv4Nodes,
    #[serde(default, skip_serializing_if = "CompactIPv6Nodes::is_empty")]
    pub nodes6: CompactIPv6Nodes,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "serde_bytes")]
    pub token: Option<Vec<u8>>,
}

#[bitmask(u8)]
pub enum WantFamily {
    Ipv4,
    Ipv6,
}

impl WantFamily {
    /// Returns the underlying want value.
    pub fn values(&self) -> Vec<&str> {
        let mut result = vec![];
        if self.contains(WantFamily::Ipv4) {
            result.push("n4");
        }
        if self.contains(WantFamily::Ipv6) {
            result.push("n6");
        }
        result
    }

    /// Returns the number of wanted values.
    pub fn len(&self) -> usize {
        let mut len = 0;
        if self.contains(WantFamily::Ipv4) {
            len += 1;
        }
        if self.contains(WantFamily::Ipv6) {
            len += 1;
        }
        len
    }
}

impl FromStr for WantFamily {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "n4" => Ok(WantFamily::Ipv4),
            "n6" => Ok(WantFamily::Ipv6),
            _ => Err(Error::Parse(
                format!("invalid want value {}", s).to_string(),
            )),
        }
    }
}

impl Default for WantFamily {
    fn default() -> Self {
        WantFamily::none()
    }
}

impl Serialize for WantFamily {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for value in self.values() {
            seq.serialize_element(value)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for WantFamily {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct WantVisitor;
        impl<'de> de::Visitor<'de> for WantVisitor {
            type Value = WantFamily;

            fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
                write!(f, "expected a sequence of Want values")
            }

            fn visit_seq<A>(self, mut seq: A) -> result::Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut result = WantFamily::none();
                while let Some(value) = seq.next_element::<String>().map_err(|e| {
                    de::Error::custom(format!("failed to deserialize Want value: {}", e))
                })? {
                    result |= WantFamily::from_str(value.as_str()).map_err(de::Error::custom)?;
                }
                Ok(result)
            }
        }

        deserializer.deserialize_any(WantVisitor)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GetPeersRequest {
    pub id: NodeId,
    pub info_hash: InfoHash,
    #[serde(default, skip_serializing_if = "WantFamily::is_none")]
    pub want: WantFamily,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GetPeersResponse {
    pub id: NodeId,
    #[serde(with = "serde_bytes")]
    pub token: Vec<u8>,
    #[serde(default, with = "serde_bytes")]
    pub values: Option<Vec<u8>>,
    #[serde(default, skip_serializing_if = "CompactIPv4Nodes::is_empty")]
    pub nodes: CompactIPv4Nodes,
    #[serde(default, skip_serializing_if = "CompactIPv6Nodes::is_empty")]
    pub nodes6: CompactIPv6Nodes,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AnnouncePeerRequest {
    pub id: NodeId,
    pub implied_port: bool,
    pub info_hash: InfoHash,
    pub port: u16,
    pub token: String,
    /// The name of the torrent, if provided
    #[serde(default, rename = "n", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<bool>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AnnouncePeerResponse {
    pub id: NodeId,
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

/// The version info of the DHT node.
#[derive(Debug, PartialEq)]
pub struct Version {
    raw: Vec<u8>,
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match std::str::from_utf8(&self.raw) {
            Ok(e) => serializer.serialize_str(e),
            Err(_) => serializer.serialize_bytes(&self.raw),
        }
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VersionVisitor;
        impl<'de> de::Visitor<'de> for VersionVisitor {
            type Value = Version;

            fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
                write!(f, "expected a version string or bytes")
            }

            fn visit_str<E>(self, v: &str) -> result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Self::Value::from(v))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Self::Value { raw: v.to_vec() })
            }
        }

        deserializer.deserialize_any(VersionVisitor)
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", std::str::from_utf8(&self.raw).unwrap_or("UNKNOWN"))
    }
}

impl Default for Version {
    fn default() -> Self {
        Self { raw: vec![] }
    }
}

impl From<&str> for Version {
    fn from(s: &str) -> Self {
        Self {
            raw: s.as_bytes().to_vec(),
        }
    }
}

/// The KRPC message communication between nodes.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Message {
    #[serde(rename = "t", with = "serde_bytes")]
    pub transaction_id_bytes: TransactionId,
    #[serde(default, rename = "v", skip_serializing_if = "Option::is_none")]
    pub version: Option<Version>,
    #[serde(flatten)]
    pub payload: MessagePayload,
    /// The node's external IP.
    /// See BEP42 for more info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip: Option<CompactIpAddr>,
    /// The node's external port
    #[serde(default, skip_serializing_if = "Option::is_none", with = "serde_bytes")]
    pub port: Option<[u8; 2]>, // this field is present in libtorrent, but not documented in a BEP
    #[serde(default, rename = "ro", skip_serializing_if = "std::ops::Not::not")]
    pub read_only: bool,
}

impl Message {
    /// Returns a new builder instance to create a message.
    pub fn builder() -> MessageBuilder {
        MessageBuilder::new()
    }

    /// Returns the node ID of the sender, if available.
    pub fn id(&self) -> Option<&NodeId> {
        match &self.payload {
            MessagePayload::Query(q) => Some(q.id()),
            MessagePayload::Response(r) => Some(r.id()),
            MessagePayload::Error(_) => None,
        }
    }

    /// Returns the [u16] representation of the transaction ID.
    pub fn transaction_id(&self) -> u16 {
        u16::from_be_bytes(self.transaction_id_bytes)
    }
}

#[derive(Debug, Default)]
pub(crate) struct MessageBuilder {
    transaction_id: Option<Vec<u8>>,
    version: Option<Version>,
    payload: Option<MessagePayload>,
    ip: Option<CompactIpAddr>,
    port: Option<[u8; 2]>,
    read_only: Option<bool>,
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
    pub fn version(&mut self, version: Version) -> &mut Self {
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

    /// Set the read-only flag of the message.
    pub fn read_only(&mut self, read_only: bool) -> &mut Self {
        self.read_only = Some(read_only);
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
            read_only: self.read_only.take().unwrap_or(false),
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

    mod want {
        use super::*;

        #[test]
        fn test_deserialize() {
            let want = WantFamily::Ipv4;
            let bytes = serde_bencode::to_bytes(&want).unwrap();
            let result = serde_bencode::from_bytes::<WantFamily>(bytes.as_slice()).unwrap();
            assert_eq!(want, result);

            let want = WantFamily::Ipv6;
            let bytes = serde_bencode::to_bytes(&want).unwrap();
            let result = serde_bencode::from_bytes::<WantFamily>(bytes.as_slice()).unwrap();
            assert_eq!(want, result);

            let want = WantFamily::Ipv4 | WantFamily::Ipv6;
            let bytes = serde_bencode::to_bytes(&want).unwrap();
            let result = serde_bencode::from_bytes::<WantFamily>(bytes.as_slice()).unwrap();
            assert_eq!(want, result);
        }
    }
}
