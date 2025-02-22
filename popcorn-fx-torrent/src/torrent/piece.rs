use crate::torrent::{InfoHash, PieceError};
use bit_vec::BitVec;
use log::trace;
use std::cmp::Ordering;
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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

impl PartialOrd for PiecePriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PiecePriority {
    fn cmp(&self, other: &Self) -> Ordering {
        let a = *self as u8;
        let b = *other as u8;

        a.cmp(&b)
    }
}

impl From<u8> for PiecePriority {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Normal,
            2 => Self::High,
            3 => Self::Readahead,
            4 => Self::Next,
            5 => Self::Now,
            _ => Self::None,
        }
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
    pub(crate) completed_parts: BitVec,
    /// The availability of this piece
    pub(crate) availability: u32,
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
            // calculate the part length.
            // if this part is the last one, it might be smaller
            let part_end = (part + 1) * MAX_PIECE_PART_SIZE;
            let part_length = if part_end > length {
                length - (part * MAX_PIECE_PART_SIZE)
            } else {
                MAX_PIECE_PART_SIZE
            };

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

    /// Get the length of this piece in bytes.
    pub fn len(&self) -> usize {
        self.length
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
        self.completed_parts.all()
    }

    /// Get if this piece has partially completed data.
    pub fn is_partially_completed(&self) -> bool {
        !self.completed_parts.all() && !self.completed_parts.none()
    }

    /// Get the byte range of the piece within the torrent.
    ///
    /// # Returns
    ///
    /// It returns a `Range<usize>` indicating the piece's position in bytes within the torrent,
    /// starting from its offset and extending to its length.
    pub fn torrent_range(&self) -> std::ops::Range<usize> {
        self.offset..(self.offset + self.length)
    }

    /// Get the parts of this piece that need to be requested from a peer.
    /// This returns the parts that have not been completed yet.
    pub fn parts_to_request(&self) -> Vec<&PiecePart> {
        self.completed_parts
            .iter()
            .enumerate()
            .filter(|(_, value)| !*value)
            .map(|(index, _)| &self.parts[index])
            .collect()
    }

    /// Increase the availability of this piece.
    /// If a peer has completed this piece, the availability should increase.
    pub(crate) fn increase_availability(&mut self) {
        self.availability += 1;
    }

    /// Decrease the availability of this piece.
    /// If a peer that had this piece disconnected, the availability should decrease.
    pub(crate) fn decrease_availability(&mut self) {
        if self.availability == 0 {
            trace!(
                "Tried to decrease availability of piece {} below 0",
                self.index
            );
            return;
        }

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

    /// Reset completed parts in case the validation of the data failed.
    ///This will reset the `completed_parts` back to `false`.
    pub(crate) fn reset_completed_parts(&mut self) {
        self.completed_parts = BitVec::from_elem(self.parts.len(), false);
    }
}

/// Identifies a piece part of a piece.
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl PartialOrd for PiecePart {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.piece != other.piece {
            return None;
        }

        Some(self.part.cmp(&other.part))
    }
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
    use std::str::FromStr;

    #[test]
    fn test_piece_priority_order() {
        let priority = PiecePriority::Normal;
        let result = priority.cmp(&PiecePriority::Normal);
        assert_eq!(Ordering::Equal, result);

        let priority = PiecePriority::Normal;
        let result = priority.cmp(&PiecePriority::None);
        assert_eq!(Ordering::Greater, result);

        let priority = PiecePriority::None;
        let result = priority.cmp(&PiecePriority::Normal);
        assert_eq!(Ordering::Less, result);

        let priority = PiecePriority::High;
        let result = priority.cmp(&PiecePriority::Normal);
        assert_eq!(Ordering::Greater, result);
    }

    #[test]
    fn test_piece_parts_to_request() {
        let expected_last_part = PiecePart {
            piece: 836,
            part: 117,
            begin: 1916928,
            length: 15000,
        };
        let piece = Piece::new(InfoHash::default(), 836, 0, 1931928);

        let parts = piece.parts_to_request();
        assert_eq!(118, parts.len(), "expected to match the number of parts");

        let last_part = parts.last().unwrap();
        assert_eq!(expected_last_part, **last_part);
    }

    #[test]
    fn test_piece_increase_availability() {
        let mut piece = Piece::new(InfoHash::default(), 0, 0, 1024);
        let expected_result = 2;

        piece.increase_availability();
        piece.increase_availability();
        let result = piece.availability();

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_piece_decrease_availability() {
        let mut piece = Piece::new(InfoHash::default(), 0, 0, 1024);

        piece.increase_availability();
        piece.increase_availability();
        piece.decrease_availability();

        let result = piece.availability();
        assert_eq!(1, result);
    }

    #[test]
    fn test_piece_decrease_availability_overflow() {
        let mut piece = Piece::new(InfoHash::default(), 0, 0, 1024);

        piece.increase_availability();
        piece.decrease_availability();
        piece.decrease_availability();

        let result = piece.availability();
        assert_eq!(0, result);
    }

    #[test]
    fn test_piece_is_completed() {
        let mut piece = create_piece(0, 3);

        piece.part_completed(0);
        piece.part_completed(1);

        let result = piece.is_completed();
        assert_eq!(false, result);

        piece.part_completed(2);

        let result = piece.is_completed();
        assert_eq!(true, result);
    }

    fn create_piece(piece: PieceIndex, num_of_parts: usize) -> Piece {
        let info_hash = InfoHash::from_str("urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7")
            .expect("expected a valid hash");
        let length = num_of_parts * MAX_PIECE_PART_SIZE;

        Piece::new(info_hash, piece, 0, length)
    }
}
