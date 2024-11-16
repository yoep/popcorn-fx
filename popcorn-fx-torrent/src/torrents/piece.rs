use crate::torrents::InfoHash;

/// The alias type used to identify piece indexes.
pub type PieceIndex = usize;

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
    /// The piece length in bytes
    pub length: usize,
    /// Indicates if the piece is completed
    pub have: bool,
    /// The priority of this piece
    pub priority: PiecePriority,
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
    /// * `length` - The length of the piece bytes.
    pub fn new(hash: InfoHash, index: PieceIndex, length: usize) -> Self {
        Self {
            hash,
            index,
            length,
            have: false,
            priority: PiecePriority::default(),
            availability: 0,
        }
    }

    /// Get the known availability of this piece within the torrent peers.
    /// If no connections have been made yet to peers, this might return 0.
    pub fn availability(&self) -> u32 {
        self.availability
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increase_availability() {
        let mut piece = Piece::new(InfoHash::default(), 0, 1024);
        let expected_result = 2;

        piece.increase_availability();
        piece.increase_availability();
        let result = piece.availability();

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_decrease_availability() {
        let mut piece = Piece::new(InfoHash::default(), 0, 1024);
        let expected_result = 1;

        piece.increase_availability();
        piece.decrease_availability();
        let result = piece.availability();

        assert_eq!(expected_result, result);
    }
}
