use crate::torrents::InfoHash;

const METADATA_MISSING_ERR: &str = "missing info metadata";

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
    /// The hash information of this piece
    pub hash: InfoHash,
    /// The index of the piece
    pub index: usize,
    /// The piece length in bytes
    pub length: usize,
    /// Indicates if the piece is completed
    pub have: bool,
    /// The priority of this piece
    pub priority: PiecePriority,
}
