use super::{Error, PeerId, Result};
use crate::torrents::peers::extensions::{ExtensionNumber, ExtensionRegistry};
use crate::torrents::{InfoHash, PieceIndex};
use bit_vec::BitVec;
use bitmask_enum::bitmask;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use log::trace;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::io::{Cursor, Read, Write};
use tokio_util::bytes::Buf;

pub const PROTOCOL: &str = "BitTorrent protocol";
const EXTENSION_BIT_AZUREUS: u8 = 0x80;
const EXTENSION_BIT_LTEP: u8 = 0x10;
const EXTENSION_BIT_ENP: u8 = 0x02;
const EXTENSION_BIT_DHT: u8 = 0x01;
const EXTENSION_BIT_XBT: u8 = 0x02;
const EXTENSION_BIT_FAST: u8 = 0x04;
const EXTENSION_BIT_NAT: u8 = 0x08;
const EXTENSION_BIT_V2_UPGRADE: u8 = 0x10;

/// The extension flags of the protocol.
/// See BEP4 (https://www.bittorrent.org/beps/bep_0004.html) for more info.
///
/// _The known collisions mentioned in BEP4, are ignored within these flags._
#[bitmask(u16)]
#[bitmask_config(vec_debug, flags_iter)]
pub enum ExtensionFlags {
    None,
    /// Azureus Messaging Protocol
    Azureus,
    /// Libtorrent Extension Protocol, aka Extensions
    LTEP,
    /// Extension Negotiation Protocol
    ENP,
    /// BitTorrent DHT
    Dht,
    /// XBT Peer Exchange
    XbtPeerExchange,
    /// suggest, haveall, havenone, reject request, and allow fast extensions
    Fast,
    /// NAT Traversal
    Nat,
    /// hybrid torrent legacy to v2 upgrade
    SupportV2,
}

impl Into<[u8; 8]> for ExtensionFlags {
    fn into(self) -> [u8; 8] {
        let mut bit_array = [0; 8];

        if self.contains(Self::Azureus) {
            bit_array[0] |= EXTENSION_BIT_AZUREUS;
        }
        if self.contains(Self::LTEP) {
            bit_array[5] |= EXTENSION_BIT_LTEP;
        }
        if self.contains(Self::ENP) {
            bit_array[5] |= EXTENSION_BIT_ENP;
        }
        if self.contains(Self::Dht) {
            bit_array[7] |= EXTENSION_BIT_DHT;
        }
        if self.contains(Self::XbtPeerExchange) {
            bit_array[7] |= EXTENSION_BIT_XBT;
        }
        if self.contains(Self::Fast) {
            bit_array[7] |= EXTENSION_BIT_FAST;
        }
        if self.contains(Self::Nat) {
            bit_array[7] |= EXTENSION_BIT_NAT;
        }
        if self.contains(Self::SupportV2) {
            bit_array[7] |= EXTENSION_BIT_V2_UPGRADE;
        }

        bit_array
    }
}

impl From<[u8; 8]> for ExtensionFlags {
    fn from(bits: [u8; 8]) -> Self {
        let mut flags = Self::None;

        if bits[0] & EXTENSION_BIT_AZUREUS == EXTENSION_BIT_AZUREUS {
            flags |= Self::Azureus;
        }
        if bits[5] & EXTENSION_BIT_LTEP == EXTENSION_BIT_LTEP {
            flags |= Self::LTEP;
        }
        if bits[5] & EXTENSION_BIT_ENP == EXTENSION_BIT_ENP {
            flags |= Self::ENP;
        }
        if bits[7] & EXTENSION_BIT_DHT == EXTENSION_BIT_DHT {
            flags |= Self::Dht;
        }
        if bits[7] & EXTENSION_BIT_XBT == EXTENSION_BIT_XBT {
            flags |= Self::XbtPeerExchange;
        }
        if bits[7] & EXTENSION_BIT_FAST == EXTENSION_BIT_FAST {
            flags |= Self::Fast;
        }
        if bits[7] & EXTENSION_BIT_NAT == EXTENSION_BIT_NAT {
            flags |= Self::Nat;
        }
        if bits[7] & EXTENSION_BIT_V2_UPGRADE == EXTENSION_BIT_V2_UPGRADE {
            flags |= Self::SupportV2;
        }

        flags
    }
}

impl Display for ExtensionFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut extensions = Vec::new();

        if self.contains(Self::Azureus) {
            extensions.push("Azureus");
        }
        if self.contains(Self::LTEP) {
            extensions.push("LTEP");
        }
        if self.contains(Self::ENP) {
            extensions.push("ENP");
        }
        if self.contains(Self::Dht) {
            extensions.push("DHT");
        }
        if self.contains(Self::XbtPeerExchange) {
            extensions.push("XBT");
        }
        if self.contains(Self::Fast) {
            extensions.push("Fast");
        }
        if self.contains(Self::Nat) {
            extensions.push("Nat");
        }
        if self.contains(Self::SupportV2) {
            extensions.push("SupportV2");
        }

        write!(f, "{}", extensions.join(" | "))
    }
}

/// These message types are used in the BitTorrent protocol and defined in BEP04.
/// This is always the first byte of the wire message.
///
/// See https://www.bittorrent.org/beps/bep_0004.html for more info.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MessageType {
    KeepAlive = 99,

    // BEP3: The BitTorrent Protocol Specification v1
    Choke = 0,
    Unchoke = 1,
    Interested = 2,
    NotInterested = 3,
    Have = 4,
    Bitfield = 5,
    Request = 6,
    Piece = 7,
    Cancel = 8,

    // BEP5: DHT extension
    Port = 0x09,

    // BEP6: Fast extension
    Suggest = 0x0D,
    HaveAll = 0x0E,
    HaveNone = 0x0F,
    RejectRequest = 0x10,
    AllowedFast = 0x11,

    // BEP10: Extension Protocol
    Extended = 0x14,

    // BEP52: The BitTorrent Protocol Specification v2
    HashRequest = 0x15,
    Hashes = 0x16,
    HashReject = 0x17,
}

impl TryFrom<u8> for MessageType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(MessageType::Choke),
            1 => Ok(MessageType::Unchoke),
            2 => Ok(MessageType::Interested),
            3 => Ok(MessageType::NotInterested),
            4 => Ok(MessageType::Have),
            5 => Ok(MessageType::Bitfield),
            6 => Ok(MessageType::Request),
            7 => Ok(MessageType::Piece),
            8 => Ok(MessageType::Cancel),
            20 => Ok(MessageType::Extended),
            21 => Ok(MessageType::HashRequest),
            22 => Ok(MessageType::Hashes),
            23 => Ok(MessageType::HashReject),
            _ => Err(Error::UnsupportedMessage(value)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Handshake {
    pub supported_extensions: ExtensionFlags,
    pub info_hash: InfoHash,
    pub peer_id: PeerId,
}

impl Handshake {
    pub fn new(info_hash: InfoHash, peer_id: PeerId, extensions: ExtensionFlags) -> Self {
        let mut extensions = extensions;

        if info_hash.has_v2() {
            extensions |= ExtensionFlags::SupportV2;
        }

        Self {
            supported_extensions: extensions,
            info_hash,
            peer_id,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(bytes);

        // read the protocol length
        let protocol_len = cursor.read_u8()?;
        if protocol_len != PROTOCOL.len() as u8 {
            return Err(Error::Handshake(format!(
                "expected protocol length {}, but got {}",
                PROTOCOL.len(),
                protocol_len
            )));
        }

        // read the protocol string
        let mut protocol_buf = vec![0; protocol_len as usize];
        cursor.read_exact(&mut protocol_buf)?;
        let protocol =
            String::from_utf8(protocol_buf).map_err(|e| Error::Handshake(e.to_string()))?;
        if protocol != PROTOCOL {
            return Err(Error::Handshake(format!(
                "expected protocol {}, but got {}",
                PROTOCOL, protocol
            )));
        }

        // read the extensions
        let mut extensions_buf = [0u8; 8];
        cursor.read_exact(&mut extensions_buf)?;
        let supported_extensions = ExtensionFlags::from(extensions_buf);

        // read the info hash
        let mut info_hash_bytes: [u8; 20] = [0; 20];
        cursor.read_exact(&mut info_hash_bytes)?;
        let info_hash = InfoHash::try_from_bytes(info_hash_bytes)
            .map_err(|e| Error::Handshake(e.to_string()))?;

        // read the peer id
        let mut peer_bytes = [0; 20];
        cursor.read(&mut peer_bytes)?;
        let peer_id = PeerId::try_from(peer_bytes.as_ref())?;

        Ok(Self {
            supported_extensions,
            info_hash,
            peer_id,
        })
    }
}

impl TryInto<Vec<u8>> for Handshake {
    type Error = Error;

    fn try_into(self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let info_hash_bytes = self.info_hash.short_info_hash_bytes();

        // write the length of the protocol string
        buffer.write_u8(PROTOCOL.len() as u8)?;
        // write the protocol string
        buffer.write_all(PROTOCOL.as_bytes())?;
        // write the supported extensions in the reserved field (8 bytes)
        buffer.write_all(&Into::<[u8; 8]>::into(self.supported_extensions))?;
        // write the info v1 hash
        buffer.write_all(&info_hash_bytes)?;
        // write the peer id
        buffer.write_all(&self.peer_id.value())?;

        Ok(buffer)
    }
}

#[derive(Clone, PartialEq)]
pub enum Message {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have(u32),
    Bitfield(BitVec),
    Request(Request),
    Piece(Piece),
    Cancel(Request),
    ExtendedHandshake(ExtendedHandshake),
    ExtendedPayload(ExtensionNumber, Vec<u8>),
    HaveAll,
    HaveNone,
}

impl Message {
    pub fn message_type(&self) -> MessageType {
        match &self {
            Message::KeepAlive => MessageType::KeepAlive,
            Message::Choke => MessageType::Choke,
            Message::Unchoke => MessageType::Unchoke,
            Message::Interested => MessageType::Interested,
            Message::NotInterested => MessageType::NotInterested,
            Message::Have(_) => MessageType::Have,
            Message::Bitfield(_) => MessageType::Bitfield,
            Message::Request(_) => MessageType::Request,
            Message::Piece(_) => MessageType::Piece,
            Message::Cancel(_) => MessageType::Cancel,
            Message::ExtendedHandshake(_) => MessageType::Extended,
            Message::ExtendedPayload(_, _) => MessageType::Extended,
            Message::HaveAll => MessageType::HaveAll,
            Message::HaveNone => MessageType::HaveNone,
        }
    }

    /// Convert the message into Bittorrent wire protocol byte array.
    ///
    /// # Returns
    ///
    /// Returns the byte array of the message.
    pub fn to_bytes(self) -> Result<Vec<u8>> {
        self.try_into()
    }
}

impl TryFrom<&[u8]> for Message {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        // verify if the received length is 0
        // in such a case, it's a keep alive message from the peer
        if bytes.is_empty() {
            trace!("Parsing keep alive message, as the received payload is empty");
            return Ok(Message::KeepAlive);
        }

        // create a cursor for the given bytes
        let payload_len = bytes.len() - 1;
        let mut cursor = Cursor::new(bytes);

        // try to read the message type which is the first single byte in the message
        let msg_type_id = cursor.read_u8()?;
        trace!("Trying to parse message type id {}", msg_type_id);
        let msg_type = MessageType::try_from(msg_type_id)?;

        trace!(
            "Trying to deserialize payload (size {}) for message type {:?}",
            payload_len,
            msg_type
        );
        match msg_type {
            MessageType::Choke => Ok(Message::Choke),
            MessageType::Unchoke => Ok(Message::Unchoke),
            MessageType::Interested => Ok(Message::Interested),
            MessageType::NotInterested => Ok(Message::NotInterested),
            MessageType::Have => Ok(Message::Have(cursor.read_u32::<BigEndian>()?)),
            MessageType::Bitfield => {
                let buffer_len = cursor.remaining();
                let mut buffer = vec![0u8; buffer_len];

                cursor.read_exact(&mut buffer).map_err(|e| {
                    Error::Parsing(format!("failed to read bitfield payload, {}", e))
                })?;

                Ok(Message::Bitfield(BitVec::from_bytes(&buffer)))
            }
            MessageType::Request => {
                let request = Request::try_from(cursor)?;
                Ok(Message::Request(request))
            }
            MessageType::Piece => {
                let piece = Piece::try_from(cursor)?;
                Ok(Message::Piece(piece))
            }
            MessageType::Cancel => {
                let request = Request::try_from(cursor)?;
                Ok(Message::Cancel(request))
            }
            MessageType::Extended => {
                let extended_id = cursor.read_u8()?;
                let buffer_len = cursor.remaining();
                let mut buffer = vec![0u8; buffer_len];

                // read the remaining bytes of the cursor into the buffer
                cursor.read_exact(&mut buffer).map_err(|e| {
                    Error::Parsing(format!("failed to read extended message payload, {}", e))
                })?;

                // if the extension id = 0, then the incoming message is an extended handshake
                // otherwise it's an extended payload which should be processed by the peer extensions
                if extended_id == 0 {
                    let extended = serde_bencode::from_bytes(&buffer).map_err(|e| {
                        Error::Parsing(format!("failed to parse extended handshake, {}", e))
                    })?;
                    Ok(Message::ExtendedHandshake(extended))
                } else {
                    Ok(Message::ExtendedPayload(extended_id, buffer))
                }
            }
            MessageType::HaveAll => Ok(Message::HaveAll),
            MessageType::HaveNone => Ok(Message::HaveNone),
            _ => Err(Error::UnsupportedMessage(msg_type as u8)),
        }
    }
}

impl TryInto<Vec<u8>> for Message {
    type Error = Error;

    fn try_into(self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        // write the message type as first byte in the buffer
        // for the keep alive message, we'll send the length message only
        // this will result in a message of length 0
        if self != Message::KeepAlive {
            let message_type = self.message_type();
            trace!("Writing peer message type {:?}", message_type);
            buffer.write_u8(message_type as u8)?;
        }

        trace!("Serializing message {:?}", self);
        match self {
            Message::Have(e) => {
                buffer.write_u32::<BigEndian>(e)?;
            }
            Message::Bitfield(bitfield) => {
                let bytes = bitfield.to_bytes();
                buffer.extend_from_slice(bytes.as_slice());
            }
            Message::Request(e) => {
                buffer.write_u32::<BigEndian>(e.index as u32)?;
                buffer.write_u32::<BigEndian>(e.begin as u32)?;
                buffer.write_u32::<BigEndian>(e.length as u32)?;
            }
            Message::Piece(e) => {
                let buffer_len = e.data.len();

                // verify if the data length matches the length field
                // if not, return an error so that the peer doesn't receive an invalid message
                if e.length != buffer_len {
                    return Err(Error::InvalidLength(e.length as u32, buffer_len as u32));
                }

                buffer.write_u32::<BigEndian>(e.index as u32)?;
                buffer.write_u32::<BigEndian>(e.begin as u32)?;
                buffer.write_u32::<BigEndian>(e.length as u32)?;

                buffer.write_all(&e.data)?;
            }
            Message::Cancel(e) => {
                buffer.write_u32::<BigEndian>(e.index as u32)?;
                buffer.write_u32::<BigEndian>(e.begin as u32)?;
                buffer.write_u32::<BigEndian>(e.length as u32)?;
            }
            Message::ExtendedHandshake(e) => {
                // the handshake identifier
                buffer.write_u8(0)?;
                buffer.write_all(
                    &*serde_bencode::to_bytes(&e).map_err(|e| Error::Parsing(e.to_string()))?,
                )?;
            }
            Message::ExtendedPayload(extension_number, bytes) => {
                buffer.write_u8(extension_number)?;
                buffer.write_all(&*bytes)?
            }
            _ => {}
        }

        Ok(buffer)
    }
}

impl Debug for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::KeepAlive => f.write_str("KeepAlive"),
            Message::Choke => f.write_str("Choke"),
            Message::Unchoke => f.write_str("Unchoke"),
            Message::Interested => f.write_str("Interested"),
            Message::NotInterested => f.write_str("NotInterested"),
            Message::Have(e) => f.debug_tuple("Have").field(e).finish(),
            Message::Bitfield(e) => f.write_fmt(format_args!("Bitfield([size {}])", e.len())),
            Message::Request(e) => f.write_fmt(format_args!("Request({:?})", e)),
            Message::Piece(e) => f.write_fmt(format_args!("Piece({:?})", e)),
            Message::Cancel(e) => f.write_fmt(format_args!("Cancel({:?})", e)),
            Message::ExtendedHandshake(e) => {
                f.write_fmt(format_args!("ExtendedHandshake({:?})", e))
            }
            Message::ExtendedPayload(number, payload) => f.write_fmt(format_args!(
                "ExtendedPayload({:?}, [size {}])",
                number,
                payload.len()
            )),
            Message::HaveAll => f.write_str("HaveAll"),
            Message::HaveNone => f.write_str("HaveNone"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtendedHandshake {
    /// Dictionary of supported extension messages which maps names of extensions to an extended message ID for each extension message.
    /// The only requirement on these IDs is that no extension message share the same one.
    /// Setting an extension number to zero means that the extension is not supported/disabled.
    /// The client should ignore any extension names it doesn't recognize.
    pub m: ExtensionRegistry,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Client name and version (as an utf-8 string).
    /// This is a much more reliable way of identifying the client than relying on the peer id encoding.
    #[serde(rename = "v")]
    pub client: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regg: Option<i32>,
    #[serde(default)]
    pub encryption: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_size: Option<u32>,
    /// Local TCP listen port. Allows each side to learn about the TCP port number of the other side.
    /// Note that there is no need for the receiving side of the connection to send this extension message, since its port number is already known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u32>,
    /// A string containing the compact representation of the ip address this peer sees you as. i.e. this is the receiver's external ip address (no port is included).
    /// This may be either an IPv4 (4 bytes) or an IPv6 (16 bytes) address.
    #[serde(
        rename = "yourip",
        default,
        with = "serde_bytes",
        skip_serializing_if = "Option::is_none"
    )]
    pub your_ip: Option<Vec<u8>>,
    #[serde(default, with = "serde_bytes", skip_serializing_if = "Option::is_none")]
    pub ipv4: Option<Vec<u8>>,
    #[serde(default, with = "serde_bytes", skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    /// The index of the piece that is being requested
    pub index: PieceIndex,
    /// The offset within the piece
    pub begin: usize,
    /// The length in bytes of the piece that is requested
    pub length: usize,
}

impl TryFrom<Cursor<&[u8]>> for Request {
    type Error = Error;

    fn try_from(mut value: Cursor<&[u8]>) -> Result<Self> {
        let index = value.read_u32::<BigEndian>()?;
        let begin = value.read_u32::<BigEndian>()?;
        let length = value.read_u32::<BigEndian>()?;

        Ok(Self {
            index: index as PieceIndex,
            begin: begin as usize,
            length: length as usize,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Piece {
    /// The index of the piece that is being requested
    pub index: PieceIndex,
    /// The offset within the piece
    pub begin: usize,
    /// The length in bytes of the piece that is requested
    pub length: usize,
    /// The data of the piece
    pub data: Vec<u8>,
}

impl TryFrom<Cursor<&[u8]>> for Piece {
    type Error = Error;

    fn try_from(mut value: Cursor<&[u8]>) -> Result<Self> {
        let index = value.read_u32::<BigEndian>()?;
        let begin = value.read_u32::<BigEndian>()?;
        let length = value.read_u32::<BigEndian>()? as usize;
        let mut buffer = vec![0u8; length];

        if length != value.remaining() {
            return Err(Error::Parsing(format!(
                "expected {} piece bytes, but got {} instead",
                length,
                value.remaining()
            )));
        }

        // read the remaining
        value.read_exact(&mut buffer)?;

        Ok(Self {
            index: index as PieceIndex,
            begin: begin as usize,
            length,
            data: buffer,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use popcorn_fx_core::testing::init_logger;
    use std::str::FromStr;

    #[test]
    fn test_handshake_new() {
        let info_hash = InfoHash::from_str(
            "urn:btmh:1220cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd",
        )
        .unwrap();
        let peer_id = PeerId::new();
        let expected_result = Handshake {
            supported_extensions: ExtensionFlags::Dht | ExtensionFlags::SupportV2,
            info_hash: info_hash.clone(),
            peer_id,
        };

        let result = Handshake::new(info_hash, peer_id, ExtensionFlags::Dht);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_handshake_into_bytes() {
        let info_hash = InfoHash::from_str(
            "urn:btmh:1220cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd",
        )
        .unwrap();
        let peer_id = PeerId::new();
        let handshake = Handshake::new(
            info_hash,
            peer_id,
            ExtensionFlags::Dht | ExtensionFlags::LTEP,
        );

        let result = TryInto::<Vec<u8>>::try_into(handshake).unwrap();

        assert_eq!(68, result.len(), "expected the handshake length to be 68");
    }

    #[test]
    fn test_message_deserialization_extended_handshake() {
        init_logger();
        let message_payload = "d1:ei1e1:md11:ut_metadatai3e6:ut_pexi1ee13:metadata_sizei304838e1:pi51413e4:reqqi512e11:upload_onlyi1e1:v17:Transmission 3.00e";
        let mut message_bytes: Vec<u8> = vec![20, 0];
        let expected_result = Message::ExtendedHandshake(ExtendedHandshake {
            m: vec![("ut_pex".to_string(), 1), ("ut_metadata".to_string(), 3)]
                .into_iter()
                .collect(),
            client: Some("Transmission 3.00".to_string()),
            regg: None,
            encryption: false,
            metadata_size: Some(304838),
            port: None,
            your_ip: None,
            ipv4: None,
            ipv6: None,
        });

        message_bytes.extend_from_slice(message_payload.as_bytes());

        let result = Message::try_from(message_bytes.as_ref()).unwrap();

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_message_type_try_from() {
        let byte = 0;
        let result = MessageType::try_from(byte);
        assert_eq!(Ok(MessageType::Choke), result);

        let byte = 1;
        let result = MessageType::try_from(byte);
        assert_eq!(Ok(MessageType::Unchoke), result);

        let byte = 2;
        let result = MessageType::try_from(byte);
        assert_eq!(Ok(MessageType::Interested), result);

        let byte = 3;
        let result = MessageType::try_from(byte);
        assert_eq!(Ok(MessageType::NotInterested), result);

        let byte = 20;
        let result = MessageType::try_from(byte);
        assert_eq!(Ok(MessageType::Extended), result);

        let byte = 21;
        let result = MessageType::try_from(byte);
        assert_eq!(Ok(MessageType::HashRequest), result);
    }

    #[test]
    fn test_message_type_invalid_byte() {
        let byte = 97;
        let result = MessageType::try_from(byte);
        assert_eq!(Err(Error::UnsupportedMessage(byte)), result);
    }

    #[test]
    fn test_message_keep_alive_to_bytes() {
        let message = Message::KeepAlive;
        let expected_result = vec![0u8; 0];

        let result = message.to_bytes().unwrap();

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_message_bitfield_to_bytes() {
        let mut bitfield = BitVec::from_elem(32, true);
        bitfield.set(13, false);
        bitfield.set(27, false);
        let mut expected_result = vec![MessageType::Bitfield as u8];
        expected_result.extend_from_slice(&bitfield.to_bytes());
        let message = Message::Bitfield(bitfield);

        let result = message.to_bytes().unwrap();

        assert_eq!(expected_result, result);
    }
}
