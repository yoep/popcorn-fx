use crate::torrent::peer::extension::{Extension, ExtensionNumber};
use crate::torrent::peer::protocol_bt::Message;
use crate::torrent::peer::{extension, PeerContext, PeerEvent};
use std::fmt::{Debug, Formatter};

use crate::torrent::{PieceIndex, TorrentMetadataInfo};
use async_trait::async_trait;
use log::{debug, error, trace, warn};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::io::Cursor;
use tokio::sync::RwLock;
use tokio_util::bytes::Buf;

pub const EXTENSION_NAME_METADATA: &str = "ut_metadata";
// The expected metadata piece size is 16 KiB, see BEP9
const METADATA_PIECE_SIZE: usize = 1024 * 16;

/// The BEP9 extension protocol message for the metadata extension.
#[derive(Serialize, Deserialize, PartialEq)]
struct MetadataExtensionMessage {
    /// Indicates which part of the metadata this message refers to
    pub piece: PieceIndex,
    /// The size of the additional bytes after the message
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_size: Option<usize>,
    #[serde(
        serialize_with = "serialize_metadata_type",
        deserialize_with = "deserialize_metadata_type"
    )]
    pub msg_type: MetadataMessageType,
    /// The remaining data within the metadata payload message
    #[serde(skip)]
    pub data: Vec<u8>,
}

impl Debug for MetadataExtensionMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetadataExtensionMessage")
            .field("piece", &self.piece)
            .field("total_size", &self.total_size)
            .field("msg_type", &self.msg_type)
            .field("data", &format!("[size {}]", self.data.len()))
            .finish()
    }
}

/// The metadata action type of the message.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetadataMessageType {
    Request = 0,
    Data = 1,
    Reject = 2,
}

pub struct MetadataExtension {
    /// The number of expected pieces
    total_pieces: RwLock<Option<usize>>,
    /// The received metadata pieces
    metadata_buffer: RwLock<Option<Vec<u8>>>,
}

impl MetadataExtension {
    pub fn new() -> Self {
        Self {
            total_pieces: RwLock::new(None),
            metadata_buffer: RwLock::new(None),
        }
    }

    async fn send_metadata<'a>(
        &'a self,
        piece: PieceIndex,
        peer: &'a PeerContext,
    ) -> extension::Result<()> {
        // retrieve the current known metadata
        let metadata = peer.metadata().await.info;

        if let Some(metadata) = metadata {
            Self::send_metadata_piece(&metadata, piece, peer).await?;
        } else {
            debug!(
                "Unable to provide torrent metadata to peer {}, metadata is unknown at this moment",
                peer
            );

            // send a reject to the peer as we're unable to provide the metadata
            let message = MetadataExtensionMessage {
                piece: 0,
                total_size: None,
                msg_type: MetadataMessageType::Reject,
                data: vec![],
            };
            let payload = serde_bencode::to_bytes(&message)
                .map_err(|e| extension::Error::Io(e.to_string()))?;

            trace!(
                "Peer {} sending torrent metadata reject, {:?}",
                peer,
                message
            );
            peer.send(Message::ExtendedPayload(1, payload))
                .await
                .map_err(|e| extension::Error::Io(e.to_string()))?;
        }

        Ok(())
    }

    async fn process_metadata<'a>(
        &'a self,
        message: MetadataExtensionMessage,
        peer: &'a PeerContext,
    ) -> extension::Result<()> {
        let mut total_pieces = self.total_pieces.read().await.as_ref().map(|e| e.clone());
        let current_piece = message.piece;

        // check if the total pieces that should be requested is already known
        if let None = total_pieces {
            let metadata_total_size = message.total_size.ok_or(extension::Error::Operation(
                "expected the total size of the metadata to be known".to_string(),
            ))?;
            // always make sure we round up so we get the last piece
            let calculated_total_pieces =
                (metadata_total_size + METADATA_PIECE_SIZE - 1) / METADATA_PIECE_SIZE;

            let mut mutex = self.total_pieces.write().await;
            *mutex = Some(calculated_total_pieces);
            total_pieces = Some(calculated_total_pieces);
            debug!(
                "Peer {} requires a total of {} metadata requests",
                peer, calculated_total_pieces
            );
        }

        {
            // append the data to the metadata buffer
            let mut mutex = self.metadata_buffer.write().await;
            if let Some(metadata_buffer) = mutex.as_mut() {
                metadata_buffer.extend_from_slice(&message.data);
            } else {
                *mutex = Some(message.data);
            }
        }

        if let Some(total_pieces) = total_pieces {
            if total_pieces - 1 == message.piece as usize {
                // try to deserialize the metadata
                let metadata_buffer = self.metadata_buffer.read().await;
                let metadata: TorrentMetadataInfo =
                    serde_bencode::from_bytes(metadata_buffer.as_ref().unwrap())?;
                drop(metadata_buffer);
                debug!("Peer {} completed metadata requests, {:?}", peer, metadata);

                // update the metadata of the underlying torrent through the peer
                peer.update_metadata(metadata).await;
                // make sure the metadata_buffer is released before trying to clear it
                self.clear_buffer().await;
            } else if self.should_request_metadata(&peer).await {
                trace!(
                    "Requesting next metadata piece {} out of {}",
                    current_piece + 1,
                    total_pieces
                );
                self.request_metadata(current_piece + 1, peer).await?;
            }
        } else {
            warn!("The metadata total pieces should be known at this point");
        }

        Ok(())
    }

    async fn request_metadata<'a>(
        &'a self,
        piece_index: PieceIndex,
        peer: &'a PeerContext,
    ) -> extension::Result<()> {
        let extension_number =
            self.find_metadata_extension_number(&peer)
                .await
                .ok_or(extension::Error::Operation(
                    "failed to find metadata extension".to_string(),
                ))?;
        let message = MetadataExtensionMessage {
            piece: piece_index,
            total_size: None,
            msg_type: MetadataMessageType::Request,
            data: vec![],
        };
        let payload = serde_bencode::to_bytes(&message)?;

        trace!(
            "Sending metadata request {}",
            String::from_utf8_lossy(payload.as_ref())
        );
        peer.send(Message::ExtendedPayload(extension_number, payload))
            .await
            .map_err(|e| extension::Error::Io(format!("{}", e)))
    }

    /// Try to find the metadata extensions number of the remote peer.
    async fn find_metadata_extension_number<'a>(
        &'a self,
        peer: &'a PeerContext,
    ) -> Option<ExtensionNumber> {
        peer.remote_extension_registry()
            .await
            .and_then(|e| e.get(EXTENSION_NAME_METADATA).cloned())
    }

    /// Check if the metadata extension is supported by the remote peer.
    async fn is_metadata_extension_supported<'a>(&'a self, peer: &'a PeerContext) -> bool {
        self.find_metadata_extension_number(&peer).await.is_some()
    }

    /// Check if the metadata should be requested for the torrent.
    async fn should_request_metadata<'a>(&'a self, peer: &'a PeerContext) -> bool {
        peer.metadata().await.info.is_none()
    }

    async fn on_extended_handshake(&self, peer: &PeerContext) {
        if self.is_metadata_extension_supported(&peer).await
            && self.should_request_metadata(peer).await
        {
            if let Err(e) = self.request_metadata(0, peer).await {
                error!("Peer {} failed to request metadata, {}", peer, e);
            }
        }
    }

    async fn send_metadata_piece(
        metadata: &TorrentMetadataInfo,
        piece: PieceIndex,
        peer: &PeerContext,
    ) -> extension::Result<()> {
        // serialize the metadata
        let metadata_bytes = serde_bencode::to_bytes(&metadata)?;
        let metadata_size = metadata_bytes.len();
        let message = MetadataExtensionMessage {
            piece,
            total_size: Some(metadata_size),
            msg_type: MetadataMessageType::Data,
            data: vec![],
        };
        let mut payload = serde_bencode::to_bytes(&message)?;

        // calculate the payload size that should be sent
        let start_index = piece * METADATA_PIECE_SIZE;
        let mut end_index = start_index + METADATA_PIECE_SIZE;

        // check if the last piece is smaller than the METADATA_PIECE_SIZE
        // if so, we need to adjust the end index
        if end_index > metadata_size {
            end_index = metadata_size;
        }

        // append the metadata_bytes slice from the start to end index to the payload
        payload.extend_from_slice(&metadata_bytes[start_index as usize..end_index as usize]);

        // send the payload to the peer
        trace!("Sending torrent metadata to peer {}, {:?}", peer, message);
        peer.send(Message::ExtendedPayload(1, payload))
            .await
            .map_err(|e| extension::Error::Io(e.to_string()))?;
        Ok(())
    }

    async fn clear_buffer(&self) {
        let mut mutex = self.metadata_buffer.write().await;
        let _ = mutex.take();
    }

    /// A custom deserializer for the metadata extension message.
    /// This is only used for the [MetadataMessageType::Data] as it contains additional bytes within
    /// the payload which represent the bencoded metadata.
    fn deserialize(payload: &[u8]) -> extension::Result<MetadataExtensionMessage> {
        let mut cursor = Cursor::new(payload);
        let mut deserializer = serde_bencode::de::Deserializer::new(&mut cursor);

        let mut message: MetadataExtensionMessage = Deserialize::deserialize(&mut deserializer)?;
        message.data = cursor.chunk().to_vec();

        Ok(message)
    }
}

#[async_trait]
impl Extension for MetadataExtension {
    fn name(&self) -> &str {
        EXTENSION_NAME_METADATA
    }

    async fn handle<'a>(
        &'a self,
        payload: &'a [u8],
        peer: &'a PeerContext,
    ) -> extension::Result<()> {
        let message: MetadataExtensionMessage = Self::deserialize(payload)?;
        trace!("Received metadata message {:?}", message);

        match message.msg_type {
            MetadataMessageType::Request => self.send_metadata(message.piece, peer).await?,
            MetadataMessageType::Data => self.process_metadata(message, peer).await?,
            MetadataMessageType::Reject => debug!(
                "Peer {} rejected the metadata request of piece {}",
                peer, message.piece
            ),
        }

        Ok(())
    }

    async fn on<'a>(&'a self, event: &'a PeerEvent, peer: &'a PeerContext) {
        match event {
            PeerEvent::ExtendedHandshakeCompleted => self.on_extended_handshake(peer).await,
            _ => {}
        }
    }
}

impl Debug for MetadataExtension {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetadataExtension")
            .field("total_pieces", &self.total_pieces)
            .finish()
    }
}

fn serialize_metadata_type<S>(
    message_type: &MetadataMessageType,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u8(message_type.clone() as u8)
}

fn deserialize_metadata_type<'de, D>(deserializer: D) -> Result<MetadataMessageType, D::Error>
where
    D: Deserializer<'de>,
{
    let value = u8::deserialize(deserializer)?;
    match value {
        0 => Ok(MetadataMessageType::Request),
        1 => Ok(MetadataMessageType::Data),
        2 => Ok(MetadataMessageType::Reject),
        _ => Err(de::Error::custom(format!(
            "Invalid message type {} specified",
            value
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let extension = MetadataExtensionMessage {
            piece: 0,
            total_size: None,
            msg_type: MetadataMessageType::Request,
            data: vec![],
        };
        let expected_result = "d8:msg_typei0e5:piecei0ee";

        let result = serde_bencode::to_string(&extension).unwrap();

        assert_eq!(expected_result, result.as_str());
    }

    #[test]
    fn test_deserialize() {
        let message = "d5:piecei5e8:msg_typei1e10:total_sizei12000ee";
        let expected_result = MetadataExtensionMessage {
            piece: 5,
            total_size: Some(12000),
            msg_type: MetadataMessageType::Data,
            data: vec![],
        };

        let result = serde_bencode::from_bytes(message.as_bytes()).unwrap();

        assert_eq!(expected_result, result);
    }
}
