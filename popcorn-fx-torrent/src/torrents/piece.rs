use crate::torrents::{InfoHash, PieceError};
use bit_vec::BitVec;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// The maximum size in bytes of a piece part can be requested from a peer.
pub const MAX_PIECE_PART_SIZE: usize = 16 * 1024; // 16 KiB

/// The alias type used to identify piece indexes.
pub type PieceIndex = usize;

/// The alias type used to identify piece part indexes.
pub type PartIndex = PieceIndex;

/// The priority of a piece.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PiecePriority {
    /// Indicates that there is no interest in this piece and the piece will be ignored
    None = 0,
    Normal = 1,
    High = 2,
    Readahead = 3,
    Next = 4,
    Now = 5,
}

impl Default for PiecePriority {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone)]
pub struct Piece {
    /// The hash information of the piece
    pub hash: InfoHash,
    /// The index of the piece
    pub index: PieceIndex,
    /// The offset of the piece within the torrent
    pub offset: usize,
    /// The piece length in bytes
    pub length: usize,
    /// The priority of this piece
    pub priority: PiecePriority,
    /// The (request) parts of the piece.
    pub parts: Vec<PiecePart>,
    /// The completed parts of the piece
    completed_parts: BitVec,
    /// The availability of this piece
    availability: u32,
}

impl Piece {
    /// Create a new piece with default priority.
    ///
    /// # Arguments
    ///
    /// * `hash` - The hash information of the piece, this is used to validate the piece data.
    /// * `index` - The index of the piece within the torrent.
    /// * `offset` - The beginning offset of the piece within the torrent.
    /// * `length` - The length of the piece bytes.
    pub fn new(hash: InfoHash, index: PieceIndex, offset: usize, length: usize) -> Self {
        let num_of_parts = (length + MAX_PIECE_PART_SIZE - 1) / MAX_PIECE_PART_SIZE;
        let mut parts = Vec::with_capacity(num_of_parts);
        let mut part_offset = 0;

        // create the parts of this piece
        // the parts will represent the requests to peers which need to be made to complete this piece
        for part in 0..num_of_parts {
            let mut part_length = MAX_PIECE_PART_SIZE;
            if part * MAX_PIECE_PART_SIZE > length {
                part_length = length - (part * MAX_PIECE_PART_SIZE);
            }

            parts.push(PiecePart {
                piece: index,
                part,
                begin: part_offset,
                length: part_length,
            });

            part_offset += part_length;
        }

        Self {
            hash,
            index,
            offset,
            length,
            priority: PiecePriority::default(),
            parts,
            completed_parts: BitVec::from_elem(num_of_parts, false),
            availability: 0,
        }
    }

    /// Get the known availability of this piece within the torrent peers.
    /// If no connections have been made yet to peers, this might return 0.
    pub fn availability(&self) -> u32 {
        self.availability
    }

    /// Get the number of request parts for this piece.
    pub fn num_of_parts(&self) -> usize {
        self.parts.len()
    }

    /// Get if this piece has all its parts completed.
    pub fn is_completed(&self) -> bool {
        self.completed_parts.count_ones() as usize == self.num_of_parts()
    }

    /// Get if this piece has partially completed data.
    pub fn is_partially_completed(&self) -> bool {
        self.completed_parts.count_ones() > 0
    }

    /// Get the bytes range of this piece.
    pub fn range(&self) -> std::ops::Range<usize> {
        self.offset..(self.offset + self.length)
    }

    /// Increase the availability of this piece.
    /// If a peer has completed this piece, the availability should increase.
    pub(crate) fn increase_availability(&mut self) {
        self.availability += 1;
    }

    /// Decrease the availability of this piece.
    /// If a peer that had this piece disconnected, the availability should decrease.
    pub(crate) fn decrease_availability(&mut self) {
        self.availability -= 1;
    }

    /// Mark this piece as fully completed.
    pub(crate) fn mark_completed(&mut self) {
        self.completed_parts.set_all();
    }

    /// Mark a part of this piece as completed
    pub(crate) fn part_completed(&mut self, part: PartIndex) {
        self.completed_parts.set(part, true);
    }
}

/// Identifies a piece part of a piece.
#[derive(Debug, Clone, PartialEq)]
pub struct PiecePart {
    /// The piece index to which this part belongs
    pub piece: PieceIndex,
    /// The unique index of this part within the piece
    pub part: PartIndex,
    /// The offset of bytes where this part begins within the piece
    pub begin: usize,
    /// The size in bytes of this part.
    /// This is related to the [MAX_PIECE_PART_SIZE]
    pub length: usize,
}

/// The piece chunk pool stores piece parts that have been received from peers.
#[derive(Debug)]
pub struct PieceChunkPool {
    pool: RwLock<HashMap<PieceIndex, Vec<u8>>>,
}

impl PieceChunkPool {
    pub fn new() -> Self {
        Self {
            pool: RwLock::new(HashMap::new()),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increase_availability() {
        let mut piece = Piece::new(InfoHash::default(), 0, 0, 1024);
        let expected_result = 2;

        piece.increase_availability();
        piece.increase_availability();
        let result = piece.availability();

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_decrease_availability() {
        let mut piece = Piece::new(InfoHash::default(), 0, 0, 1024);
        let expected_result = 1;

        piece.increase_availability();
        piece.decrease_availability();
        let result = piece.availability();

        assert_eq!(expected_result, result);
    }
}
