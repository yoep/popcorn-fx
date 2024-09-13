use crate::torrents::peers::errors::Result;
use crate::torrents::peers::PeerError;
use std::collections::HashMap;

/// The message types used in the BitTorrent protocol.
/// This is always the first byte of the wire message.
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

    // BEP6: Fast extension
    HaveAll = 14,
    HaveNone = 15,

    // BEP10: Extension Protocol
    Extended = 20,

    // BEP52: The BitTorrent Protocol Specification v2
    HashRequest = 21,
    Hashes = 22,
    HashReject = 23,
}

impl TryFrom<u8> for MessageType {
    type Error = PeerError;

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
            _ => Err(PeerError::UnsupportedMessage(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(Err(PeerError::UnsupportedMessage(byte)), result);
    }
}
