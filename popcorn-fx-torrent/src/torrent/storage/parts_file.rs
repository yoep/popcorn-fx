use crate::torrent::storage::{Error, Result};
use crate::torrent::torrent_pools::PiecePool;
use crate::torrent::{Piece, PieceIndex};
use log::trace;
use std::collections::BTreeMap;
use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::sync::RwLock;

type SlotIndex = usize;

#[derive(Debug)]
pub struct PartsFile {
    filename: String,
    path: PathBuf,
    pieces: PiecePool,
    piece_slots: RwLock<BTreeMap<PieceIndex, SlotIndex>>,
    slots: RwLock<BTreeMap<SlotIndex, PartSlot>>,
}

impl PartsFile {
    /// Create a new parts file instance for the given filename and location path.
    ///
    /// The path to the parts file **should not include** the filename of the parts file.
    pub fn new<S: AsRef<str>, P: AsRef<Path>>(filename: S, path: P, pieces: PiecePool) -> Self {
        Self {
            filename: filename.as_ref().to_string(),
            path: path.as_ref().to_path_buf(),
            pieces,
            piece_slots: Default::default(),
            slots: Default::default(),
        }
    }

    pub async fn read(
        &self,
        buffer: &mut [u8],
        piece: &PieceIndex,
        offset: usize,
    ) -> Result<usize> {
        // early return when the parts file doesn't exist
        let filepath = self.absolute_filepath().await;
        if !filepath.exists() {
            return Err(Error::Unavailable);
        }

        let slot_index = {
            let piece_slots = self.piece_slots.read().await;
            *piece_slots.get(piece).ok_or(Error::Unavailable)?
        };
        let slots = self.slots.read().await;
        let slot = slots.get(&slot_index).ok_or(Error::Unavailable)?;

        let mut file = self.open(false).await?;
        let metadata = file.metadata().await?;
        let file_offset = slot.offset.saturating_add(offset);
        if file_offset > metadata.len() as usize {
            return Err(Error::OutOfBounds);
        }

        file.seek(SeekFrom::Start(file_offset as u64)).await?;
        let bytes_read = file.read(buffer).await?;

        Ok(bytes_read)
    }

    pub async fn write(&self, data: &[u8], piece: &PieceIndex, offset: usize) -> Result<usize> {
        let mut slots = self.slots.write().await;
        let mut piece_slots = self.piece_slots.write().await;
        let piece = self.pieces.get(piece).await.ok_or(Error::Unavailable)?;
        let slot = match piece_slots
            .get(&piece.index)
            .and_then(|slot_index| slots.get(slot_index))
        {
            Some(slot) => slot,
            None => {
                let slot_index = Self::get_free_slot(&piece, &mut slots);
                piece_slots.insert(piece.index, slot_index);
                trace!(
                    "Parts file {:?} created new slot {} for piece {}",
                    self.absolute_filepath().await,
                    slot_index,
                    piece.index
                );
                slots.get(&slot_index).ok_or(Error::Unavailable)?
            }
        };

        // check if the data fits within the slot
        if offset + data.len() > slot.len {
            return Err(Error::OutOfBounds);
        }

        let mut file = self.open(true).await?;
        let file_offset = slot.offset.saturating_add(offset);

        file.seek(SeekFrom::Start(file_offset as u64)).await?;
        file.write_all(data).await?;
        file.flush().await?;

        Ok(data.len())
    }

    /// Open the parts file.
    async fn open(&self, write: bool) -> Result<File> {
        let absolute_filepath = self.absolute_filepath().await;
        Ok(OpenOptions::new()
            .read(true)
            .write(write)
            .create(write)
            .open(absolute_filepath)
            .await?)
    }

    /// Get the absolute filepath of the parts file.
    async fn absolute_filepath(&self) -> PathBuf {
        self.path.join(&self.filename)
    }

    /// Get a free slot within the part file.
    /// This fn will try to reclaim any unused slots when possible,
    /// otherwise, it will append a new slot at the end.
    fn get_free_slot(piece: &Piece, slots: &mut BTreeMap<SlotIndex, PartSlot>) -> SlotIndex {
        let piece_len = piece.len();
        let (slot_index, slot) = slots
            .iter()
            .find(|(_, slot)| !slot.in_use && piece_len == slot.len)
            .map(|(index, slot)| {
                (
                    *index,
                    PartSlot {
                        offset: slot.offset,
                        len: slot.len,
                        in_use: true,
                    },
                )
            })
            .unwrap_or_else(|| {
                if let Some((index, last_slot)) = slots.last_key_value() {
                    (
                        index + 1,
                        PartSlot {
                            offset: last_slot.offset + last_slot.len + 1,
                            len: piece.len(),
                            in_use: true,
                        },
                    )
                } else {
                    (
                        0,
                        PartSlot {
                            offset: 0,
                            len: piece.len(),
                            in_use: true,
                        },
                    )
                }
            });

        slots.insert(slot_index, slot);

        slot_index
    }
}

#[derive(Debug, Clone)]
struct PartSlot {
    offset: usize,
    len: usize,
    in_use: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_logger;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_write_create_new_slot() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let filename = ".test.parts";
        let piece_index = 2 as PieceIndex;
        let piece_len = 16;
        let piece_data = "lorem".as_bytes();
        let pieces = PiecePool::from(vec![Piece {
            hash: Default::default(),
            index: piece_index,
            offset: 0,
            length: piece_len,
            priority: Default::default(),
            parts: vec![],
            completed_parts: Default::default(),
            availability: 0,
        }]);
        let part_file = PartsFile::new(filename, temp_path, pieces.clone());

        let result = part_file.write(piece_data, &piece_index, 0).await;
        assert_eq!(
            Ok(piece_data.len()),
            result,
            "expected the piece data to have been written"
        );

        let mut buffer = vec![0u8; piece_len];
        let result = part_file.read(&mut buffer, &piece_index, 0).await;
        assert_eq!(
            Ok(piece_data.len()),
            result,
            "expected the original written data to have been returned"
        );
    }
}
