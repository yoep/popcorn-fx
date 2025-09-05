use crate::torrent::{PieceError, PieceIndex, PiecePart};
use std::collections::BTreeMap;
use tokio::sync::RwLock;

/// The piece pool stores piece chunks/parts that have been received from remote peers.
#[derive(Debug)]
pub struct PiecePool {
    pool: RwLock<BTreeMap<PieceIndex, Vec<u8>>>,
}

impl PiecePool {
    pub fn new() -> Self {
        Self {
            pool: RwLock::new(BTreeMap::new()),
        }
    }

    /// Add a chunk to the pool for the given received piece part.
    ///
    /// # Arguments
    ///
    /// * `piece_part` - The part of the piece that has been received.
    /// * `piece_length` - The length of the piece, used for initializing the chunk vector.
    /// * `data` - The data that has been received
    pub async fn add_chunk(
        &self,
        piece_part: &PiecePart,
        piece_length: usize,
        data: Vec<u8>,
    ) -> Result<(), PieceError> {
        let mut mutex = self.pool.write().await;
        let chunks = mutex
            .entry(piece_part.piece)
            .or_insert_with(|| Vec::with_capacity(piece_length));
        let part_end = piece_part.begin + piece_part.length;

        if part_end > piece_length {
            return Err(PieceError::InvalidChunkSize(piece_length, part_end));
        }

        // ensure that the vector is large enough to hold the data at the part_end
        if chunks.len() < part_end {
            chunks.resize(part_end, 0);
        }

        chunks.splice(piece_part.begin..part_end, data);
        Ok(())
    }

    /// Get the data from the given piece.
    /// This will return the buffered data and remove it from the pool.
    pub async fn get(&self, piece: PieceIndex) -> Option<Vec<u8>> {
        let mut mutex = self.pool.write().await;
        mutex.remove(&piece)
    }
}
