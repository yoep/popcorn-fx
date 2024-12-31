use crate::torrent::peer::extension::{Extension, Result};
use crate::torrent::peer::{PeerContext, PeerEvent};
use crate::torrent::PieceIndex;
use async_trait::async_trait;
use log::{debug, trace};
use serde::{Deserialize, Serialize};

const DONTHAVE_EXTENSION_NAME: &str = "lt_donthave";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct DontHaveMessage {
    /// The piece index that is no longer available
    piece: u32,
}

#[derive(Debug)]
pub struct DontHaveExtension;

impl DontHaveExtension {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Extension for DontHaveExtension {
    fn name(&self) -> &str {
        DONTHAVE_EXTENSION_NAME
    }

    async fn handle<'a>(&'a self, payload: &'a [u8], peer: &'a PeerContext) -> Result<()> {
        trace!("Peer {} is parsing donthave message", peer);
        let message: DontHaveMessage = serde_bencode::from_bytes(payload)?;
        debug!("Peer {} parsed donthave message {:?}", peer, message);

        peer.remote_has_piece(message.piece as PieceIndex, false)
            .await;
        Ok(())
    }

    async fn on<'a>(&'a self, _: &'a PeerEvent, _: &'a PeerContext) {
        // no-op
    }
}
