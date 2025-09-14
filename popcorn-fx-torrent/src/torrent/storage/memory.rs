use crate::torrent::storage::{Error, Metrics, Result, Storage};
use crate::torrent::{PieceIndex, Sha1Hash, Sha256Hash};
use async_trait::async_trait;
use sha1::{Digest, Sha1};
use sha2::Sha256;
use std::cmp::min;
use std::collections::BTreeMap;
use std::io;
use std::path::Path;
use tokio::sync::RwLock;

/// Fast in-memory storage of torrent piece data.
/// This storage type is not recommended for large torrents.
#[derive(Debug)]
pub struct MemoryStorage {
    pieces: RwLock<BTreeMap<PieceIndex, Vec<u8>>>,
    metrics: Metrics,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            pieces: Default::default(),
            metrics: Default::default(),
        }
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn read(&self, buffer: &mut [u8], piece: &PieceIndex, offset: usize) -> Result<usize> {
        let mut cursor = 0usize;
        let buffer_len = buffer.len();
        let pieces = self.pieces.read().await;
        let index = *piece;

        while cursor < buffer_len {
            match pieces.get(&index) {
                None => break,
                Some(piece) => {
                    let remaining_bytes = buffer_len.saturating_sub(cursor);
                    let copy_len = min(remaining_bytes, piece.len().saturating_sub(offset));
                    buffer[cursor..cursor + copy_len]
                        .copy_from_slice(&piece[offset..offset + copy_len]);

                    cursor += copy_len;
                    self.metrics.bytes_read.inc_by(copy_len as u64);
                }
            }
        }

        Ok(cursor)
    }

    async fn write(&self, data: &[u8], piece: &PieceIndex, offset: usize) -> Result<usize> {
        let mut pieces = self.pieces.write().await;
        let piece = if !pieces.contains_key(&piece) {
            pieces.insert(*piece, vec![0u8; data.len() + offset]);
            pieces.get_mut(&piece)
        } else {
            pieces.get_mut(&piece)
        }
        .ok_or(Error::Unavailable)?;

        let end = data.len().saturating_add(offset);
        piece[offset..end].copy_from_slice(data);
        self.metrics.bytes_written.inc_by(data.len() as u64);

        Ok(data.len())
    }

    async fn hash_v1(&self, piece: &PieceIndex) -> Result<Sha1Hash> {
        let pieces = self.pieces.read().await;
        let bytes = pieces.get(&piece).map(|e| &e[..]).unwrap_or(&[]);

        Sha1Hash::try_from(Sha1::digest(bytes))
            .map_err(|e| Error::Io(io::Error::new(io::ErrorKind::Other, e.to_string())))
    }

    async fn hash_v2(&self, piece: &PieceIndex) -> Result<Sha256Hash> {
        let pieces = self.pieces.read().await;
        let bytes = pieces.get(&piece).map(|e| &e[..]).unwrap_or(&[]);

        Sha256Hash::try_from(Sha256::digest(bytes))
            .map_err(|e| Error::Io(io::Error::new(io::ErrorKind::Other, e.to_string())))
    }

    async fn move_storage(&self, _: &Path) -> Result<()> {
        Ok(())
    }

    fn metrics(&self) -> &Metrics {
        &self.metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torrent::operation::TorrentCreatePiecesOperation;
    use crate::torrent::tests::read_test_file_to_bytes;
    use crate::torrent::{TorrentOperation, TorrentOperationResult};
    use crate::{create_torrent, init_logger};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_read() {
        init_logger!();
        let piece = 1 as PieceIndex;
        let data = read_test_file_to_bytes("piece-1.iso");
        let storage = MemoryStorage::new();

        // write the piece data
        let result = storage.write(&data, &piece, 0).await;
        assert_eq!(
            Ok(data.len()),
            result,
            "expected the piece to have been written to memory"
        );

        // read the piece data
        let mut buffer = vec![0u8; data.len()];
        let result = storage.read(&mut buffer, &piece, 0).await;
        assert_eq!(
            Ok(data.len()),
            result,
            "expected the piece to have been read from memory"
        );
        assert_eq!(buffer, data, "expected the piece data to match");
    }

    #[tokio::test]
    async fn test_hash_v1() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let piece: PieceIndex = 0;
        let data = read_test_file_to_bytes("piece-1.iso");
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::builder().path(temp_path).build(),
            vec![],
            vec![]
        );
        let operation = TorrentCreatePiecesOperation::new();
        let context = &torrent.instance().unwrap();
        let storage = MemoryStorage::new();

        // write the piece data
        let result = storage.write(&data, &piece, 0).await;
        assert_eq!(
            Ok(data.len()),
            result,
            "expected the piece to have been written to memory"
        );

        // create the pieces
        let result = operation.execute(context).await;
        assert_eq!(
            TorrentOperationResult::Continue,
            result,
            "expected the pieces to have been created"
        );
        let piece_hash = context
            .piece_pool()
            .get(&piece)
            .await
            .expect("expected the piece to have been found")
            .hash;
        let expected_hash = piece_hash
            .hash_v1()
            .expect("expected the v1 hash to be present within the piece");

        // hash the piece
        let result = storage
            .hash_v1(&piece)
            .await
            .expect("expected the hash to have been calculated");
        assert_eq!(
            expected_hash, result,
            "expected the hash to equal the piece hash"
        );
    }
}
