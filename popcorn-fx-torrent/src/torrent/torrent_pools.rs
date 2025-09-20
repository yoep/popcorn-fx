use crate::torrent::{
    File, FileAttributeFlags, FileIndex, FilePriority, PartIndex, Piece, PieceIndex, PiecePriority,
};
use bit_vec::BitVec;
use itertools::Itertools;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// The torrent piece management pool.
#[derive(Debug, Clone)]
pub struct PiecePool {
    inner: Arc<InnerPiecePool>,
}

impl PiecePool {
    /// Create a new empty piece pool instance.
    pub fn new() -> Self {
        Self::from(Vec::with_capacity(0))
    }

    /// Returns the number of pieces within the pool.
    pub async fn len(&self) -> usize {
        self.inner.pieces.read().await.len()
    }

    /// Get the piece for the given index.
    pub async fn get(&self, piece: &PieceIndex) -> Option<Piece> {
        self.inner.pieces.read().await.get(piece).cloned()
    }

    /// Check if the given piece is present within the pool.
    pub async fn contains(&self, piece: &PieceIndex) -> bool {
        let pieces = self.inner.pieces.read().await;
        pieces.get(piece).is_some()
    }

    /// Get all pieces present within the pool.
    pub async fn pieces(&self) -> Vec<Piece> {
        self.inner.pieces.read().await.values().cloned().collect()
    }

    /// Set the pieces of the pool.
    /// This will replace all existing pieces within the pool.
    pub async fn set_pieces(&self, pieces: Vec<Piece>) {
        let pieces_len = pieces.len();
        let pieces = pieces
            .into_iter()
            .map(|piece| (piece.index, piece))
            .collect();

        {
            let mut mutex = self.inner.pieces.write().await;
            *mutex = pieces;
        }

        {
            let mut bitfield = self.inner.completed_pieces.write().await;
            *bitfield = BitVec::from_elem(pieces_len, false);
        }
    }

    /// Get the pieces bitfield, indicating which piece has completed.
    pub async fn bitfield(&self) -> BitVec {
        self.inner.completed_pieces.read().await.clone()
    }

    /// Check if all interested pieces have been downloaded by the torrent.
    /// This means that every piece with anything but a [PiecePriority::None] have
    /// downloaded and validated their data.
    pub async fn is_completed(&self) -> bool {
        let bitfield = self.inner.completed_pieces.read().await;
        let pieces = self.inner.pieces.read().await;

        pieces
            .iter()
            .filter(|(_, piece)| piece.priority != PiecePriority::None)
            .all(|(index, _)| bitfield.get(*index).unwrap_or(false))
    }

    /// Check if the given piece has completed downloading the data.
    ///
    /// Returns `true` when the data is downloaded & validated for the given piece, else `false`.
    pub async fn is_piece_completed(&self, piece: &PieceIndex) -> bool {
        self.inner
            .completed_pieces
            .read()
            .await
            .get(*piece)
            .unwrap_or_default()
    }

    /// Set the completion state of the given piece index.
    pub async fn set_completed(&self, piece: &PieceIndex, completed: bool) {
        {
            let mut pieces = self.inner.pieces.write().await;
            if let Some(piece) = pieces.get_mut(piece) {
                if completed {
                    piece.mark_completed();
                } else {
                    piece.reset_completed_parts();
                }
            }
        }

        {
            let mut bitfield = self.inner.completed_pieces.write().await;
            bitfield.set(*piece, completed)
        }
    }

    /// Get the amount of completed pieces within the pool.
    pub async fn num_completed(&self) -> usize {
        self.inner.completed_pieces.read().await.count_ones() as usize
    }

    /// Get the priorities of the pieces for the torrent.
    pub async fn priorities(&self) -> BTreeMap<PieceIndex, PiecePriority> {
        let pieces = self.inner.pieces.read().await;
        pieces
            .iter()
            .map(|(index, piece)| (*index, piece.priority.clone()))
            .collect()
    }

    /// Set the priorities for the given pieces in the torrent.
    pub async fn set_priorities(&self, priorities: &[(PieceIndex, PiecePriority)]) {
        let mut pieces = self.inner.pieces.write().await;
        for (index, priority) in priorities {
            if let Some(piece) = pieces.get_mut(index) {
                piece.priority = *priority;
            }
        }
    }

    /// Get all piece indexes in which the torrent is interested.
    /// This filters out all pieces which have priority [PiecePriority::None].
    pub async fn interested_pieces(&self) -> Vec<PieceIndex> {
        let pieces = self.inner.pieces.read().await;
        pieces
            .iter()
            .filter(|(_, piece)| piece.priority != PiecePriority::None)
            .map(|(piece, _)| *piece)
            .collect()
    }

    /// Check if the given piece is wanted by the torrent.
    pub async fn is_wanted(&self, piece: &PieceIndex) -> bool {
        let bitfield = self.inner.completed_pieces.read().await;
        let pieces = self.inner.pieces.read().await;

        pieces
            .get(piece)
            .filter(|piece| Self::is_wanted_piece(&*bitfield, piece))
            .is_some()
    }

    /// Check if the given piece index is wanted by the torrent.
    /// Returns `true` if the piece has  not been completed yet and the priority is not [PiecePriority::None].
    pub async fn is_piece_wanted(&self, piece: &PieceIndex) -> bool {
        let is_completed = {
            let bitfield = self.inner.completed_pieces.read().await;
            bitfield.get(*piece).unwrap_or_default()
        };

        if !is_completed {
            let pieces = self.inner.pieces.read().await;

            return pieces
                .get(piece)
                .map(|piece| piece.priority != PiecePriority::None)
                .unwrap_or_default();
        }

        false
    }

    /// Get the pieces which are still wanted (need to be downloaded) by the torrent.
    /// The list is sorted based on the piece priority.
    pub async fn wanted_pieces(&self) -> Vec<Piece> {
        let bitfield = self.inner.completed_pieces.read().await;
        let pieces = self.inner.pieces.read().await;
        pieces
            .iter()
            .filter(|(_, piece)| Self::is_wanted_piece(&*bitfield, piece))
            .map(|(_, piece)| piece.clone())
            .sorted_by(|a, b| b.priority.cmp(&a.priority))
            .collect()
    }

    /// Get the amount of bytes in which the torrent is interested.
    pub async fn interested_size(&self) -> usize {
        let pieces = self.inner.pieces.read().await;
        pieces
            .iter()
            .filter(|(_, piece)| piece.priority == PiecePriority::None)
            .map(|(_, piece)| piece.len())
            .sum()
    }

    /// Update the availability of the given piece.
    pub async fn update_availability(&self, piece: &PieceIndex, change: i32) {
        let mut pieces = self.inner.pieces.write().await;
        if let Some(piece) = pieces.get_mut(piece) {
            if change >= 0 {
                piece.availability = piece.availability.saturating_add(change as u32);
            } else {
                piece.availability = piece.availability.saturating_sub(change.unsigned_abs());
            }
        }
    }

    /// Set the given piece part as completed.
    pub async fn set_part_completed(&self, piece: &PieceIndex, part: &PartIndex) {
        let mut pieces = self.inner.pieces.write().await;
        if let Some(piece) = pieces.get_mut(piece) {
            piece.part_completed(part);

            if piece.is_completed() {
                let mut bitfield = self.inner.completed_pieces.write().await;
                bitfield.set(piece.index.clone(), true);
            }
        }
    }

    /// Check if the piece is wanted by the torrent.
    /// In such a case, the piece priority should not be [PiecePriority::None]
    /// and the piece should not have been completed yet.
    fn is_wanted_piece(bitfield: &BitVec, piece: &Piece) -> bool {
        piece.priority != PiecePriority::None
            && bitfield.get(piece.index).unwrap_or_default() == false
    }
}

impl<S: AsRef<[Piece]>> From<S> for PiecePool {
    fn from(value: S) -> Self {
        let slice = value.as_ref();
        Self {
            inner: Arc::new(InnerPiecePool {
                completed_pieces: RwLock::new(BitVec::from_elem(slice.len(), false)),
                pieces: RwLock::new(
                    slice
                        .iter()
                        .map(|piece| (piece.index.clone(), piece.clone()))
                        .collect(),
                ),
            }),
        }
    }
}

#[derive(Debug)]
struct InnerPiecePool {
    completed_pieces: RwLock<BitVec>,
    pieces: RwLock<BTreeMap<PieceIndex, Piece>>,
}

/// The torrent file management pool.
#[derive(Debug, Clone)]
pub struct FilePool {
    inner: Arc<InnerFilePool>,
}

impl FilePool {
    /// Create a new file pool.
    pub fn new(piece_pool: PiecePool) -> Self {
        Self {
            inner: Arc::new(InnerFilePool {
                piece_pool,
                files: Default::default(),
            }),
        }
    }

    /// Get the underlying pieces of the file pool.
    pub fn pieces(&self) -> &PiecePool {
        &self.inner.piece_pool
    }

    /// Returns the number of files within the pool.
    /// Files with attribute [FileAttributeFlags::PaddingFile] are not counted.
    pub async fn len(&self) -> usize {
        self.inner
            .files
            .read()
            .await
            .iter()
            .filter(|(_, file)| !file.attributes().contains(FileAttributeFlags::PaddingFile))
            .count()
    }

    /// Get the file for the given index.
    pub async fn get(&self, file: &FileIndex) -> Option<File> {
        self.inner.files.read().await.get(file).cloned()
    }

    /// Get the available files within the pool.
    pub async fn files(&self) -> Vec<File> {
        self.inner.files.read().await.values().cloned().collect()
    }

    /// Set the files of the pool.
    /// This will replace all existing files within the pool.
    pub async fn set_files(&self, files: Vec<File>) {
        *self.inner.files.write().await =
            files.into_iter().map(|file| (file.index, file)).collect();
    }

    /// Get the file index at the given torrent offset.
    pub async fn file_index_at_offset(&self, offset: usize) -> Option<FileIndex> {
        let files = self.inner.files.read().await;

        files
            .iter()
            .find(|(_, file)| {
                let file_end = file.torrent_offset + file.len();
                offset >= file.torrent_offset && offset <= file_end
            })
            .map(|(index, _)| *index)
    }

    /// Get the file index which contains the start byte for the given piece.
    ///
    /// Returns the file containing the start byte for the piece.
    pub async fn file_index_for(&self, piece: &PieceIndex) -> Option<FileIndex> {
        let files = self.inner.files.read().await;
        let piece = self.inner.piece_pool.get(piece).await?;

        files
            .iter()
            .find(|(_, file)| {
                piece.offset >= file.torrent_offset
                    && piece.offset <= file.torrent_offset + file.len()
            })
            .map(|(index, _)| *index)
    }

    /// Set the priorities of for the given file indexes.
    pub async fn set_priorities(&self, priorities: &[(FileIndex, FilePriority)]) {
        let mut files = self.inner.files.write().await;
        let mut piece_priorities = BTreeMap::new();

        for (index, file_priority) in priorities {
            if let Some(file) = files.get_mut(index) {
                file.priority = *file_priority;

                for piece in file.pieces.clone() {
                    let piece_priority = piece_priorities
                        .entry(piece)
                        .or_insert(*file_priority as PiecePriority);
                    *piece_priority = (*piece_priority).max(*file_priority);
                }
            }
        }

        self.inner
            .piece_pool
            .set_priorities(
                &piece_priorities
                    .into_iter()
                    .map(|(k, v)| (k, v))
                    .collect::<Vec<_>>(),
            )
            .await;
    }
}

#[derive(Debug)]
struct InnerFilePool {
    piece_pool: PiecePool,
    files: RwLock<BTreeMap<FileIndex, File>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod piece_pool {
        use super::*;

        mod is_piece_wanted {
            use super::*;

            #[tokio::test]
            async fn test_piece_completed() {
                let piece = 0;
                let pool = PiecePool::from(vec![
                    Piece {
                        hash: Default::default(),
                        index: piece,
                        offset: 0,
                        length: 1024,
                        priority: PiecePriority::Normal,
                        parts: vec![],
                        completed_parts: Default::default(),
                        availability: 0,
                    },
                    Piece {
                        hash: Default::default(),
                        index: 1,
                        offset: 1024,
                        length: 1024,
                        priority: PiecePriority::None,
                        parts: vec![],
                        completed_parts: Default::default(),
                        availability: 0,
                    },
                ]);

                let result = pool.is_piece_wanted(&piece).await;
                assert_eq!(true, result, "expected the piece to have been wanted");

                // set the piece as completed
                pool.set_completed(&piece, true).await;

                let result = pool.is_piece_wanted(&piece).await;
                assert_eq!(false, result, "expected the piece to be no longer wanted");
            }

            #[tokio::test]
            async fn test_piece_priority_none() {
                let piece = 1;
                let pool = PiecePool::from(vec![
                    Piece {
                        hash: Default::default(),
                        index: 0,
                        offset: 0,
                        length: 1024,
                        priority: PiecePriority::Normal,
                        parts: vec![],
                        completed_parts: Default::default(),
                        availability: 0,
                    },
                    Piece {
                        hash: Default::default(),
                        index: piece,
                        offset: 1024,
                        length: 1024,
                        priority: PiecePriority::None,
                        parts: vec![],
                        completed_parts: Default::default(),
                        availability: 0,
                    },
                ]);

                let result = pool.is_piece_wanted(&piece).await;
                assert_eq!(false, result, "expected the piece to not have been wanted");

                // set the piece as completed
                pool.set_completed(&piece, true).await;

                let result = pool.is_piece_wanted(&piece).await;
                assert_eq!(false, result, "expected the piece to not have been wanted");
            }
        }

        mod is_completed {
            use super::*;

            #[tokio::test]
            async fn test_piece_set_completed() {
                let pool = PiecePool::from(vec![
                    Piece {
                        hash: Default::default(),
                        index: 0,
                        offset: 0,
                        length: 1024,
                        priority: PiecePriority::Normal,
                        parts: vec![],
                        completed_parts: Default::default(),
                        availability: 0,
                    },
                    Piece {
                        hash: Default::default(),
                        index: 1,
                        offset: 1024,
                        length: 1024,
                        priority: PiecePriority::Normal,
                        parts: vec![],
                        completed_parts: Default::default(),
                        availability: 0,
                    },
                ]);

                pool.set_completed(&0, true).await;
                let result = pool.is_completed().await;
                assert_eq!(
                    false, result,
                    "expected the torrent to not have been completed yet"
                );

                pool.set_completed(&1, true).await;
                let result = pool.is_completed().await;
                assert_eq!(true, result, "expected the torrent to have been completed");
            }
        }
    }
}
