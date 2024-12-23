use crate::torrent::{
    InfoHash, Piece, PieceError, PieceIndex, TorrentCommandEvent, TorrentContext,
    TorrentMetadataInfo, TorrentOperation, TorrentOperationResult, TorrentState,
};
use async_trait::async_trait;
use bit_vec::BitVec;
use derive_more::Display;
use log::{debug, trace, warn};
use std::sync::Arc;

#[derive(Debug)]
pub struct TorrentCreatePiecesOperation;

impl TorrentCreatePiecesOperation {
    pub fn new() -> Self {
        Self {}
    }

    /// Create the pieces information for the torrent.
    /// This operation can only be done when the metadata of the torrent is known.
    async fn create_pieces(&self, torrent: &TorrentContext) -> bool {
        torrent.update_state(TorrentState::Initializing).await;

        match self.try_create_pieces(torrent).await {
            Ok(pieces) => {
                trace!(
                    "Torrent {} created a total of {} pieces",
                    torrent,
                    pieces.len()
                );
                torrent.update_pieces(pieces).await;
                true
            }
            Err(e) => {
                warn!("Torrent {} failed to create torrent pieces, {}", torrent, e);
                false
            }
        }
    }

    /// Try to create the pieces of the torrent.
    /// This operation doesn't store the pieces results.
    ///
    /// # Returns
    ///
    /// Returns the pieces result for the torrent if available, else the error.
    async fn try_create_pieces(&self, data: &TorrentContext) -> crate::torrent::Result<Vec<Piece>> {
        let info_hash: InfoHash;
        let num_pieces: usize;
        let metadata: TorrentMetadataInfo;

        {
            let mutex = data.metadata().await;
            info_hash = mutex.info_hash.clone();
            metadata = mutex
                .info
                .clone()
                .ok_or(PieceError::UnableToDeterminePieces(
                    "metadata is unavailable".to_string(),
                ))?;
            num_pieces = mutex
                .total_pieces()
                .ok_or(PieceError::UnableToDeterminePieces(
                    "failed to calculate number of pieces".to_string(),
                ))?;
        }

        let sha1_pieces = if info_hash.has_v1() {
            metadata.sha1_pieces()
        } else {
            Vec::new()
        };
        let sha256_pieces = if info_hash.has_v2() {
            metadata.sha256_pieces()
        } else {
            Vec::new()
        };
        let mut pieces = Vec::with_capacity(num_pieces);
        let total_file_size = metadata.total_size();
        let piece_length = metadata.piece_length as usize;
        let mut last_piece_length = total_file_size % piece_length;
        let mut offset = 0;

        if last_piece_length == 0 {
            last_piece_length = piece_length;
        }

        for piece_index in 0..num_pieces {
            let hash = if info_hash.has_v2() {
                InfoHash::try_from_bytes(sha256_pieces.get(piece_index).unwrap())?
            } else {
                InfoHash::try_from_bytes(sha1_pieces.get(piece_index).unwrap())?
            };
            let length = if piece_index != num_pieces - 1 {
                piece_length
            } else {
                last_piece_length
            };

            pieces.push(Piece::new(hash, piece_index as PieceIndex, offset, length));
            offset += length;
        }

        Ok(pieces)
    }
}

#[async_trait]
impl TorrentOperation for TorrentCreatePiecesOperation {
    fn name(&self) -> &str {
        "create pieces operation"
    }

    async fn execute(&self, torrent: &Arc<TorrentContext>) -> TorrentOperationResult {
        // check if the pieces have already been created
        // if so, continue the chain
        if torrent.total_pieces().await > 0 {
            return TorrentOperationResult::Continue;
        }

        // try to create the pieces
        if self.create_pieces(&torrent).await {
            TorrentOperationResult::Continue
        } else {
            TorrentOperationResult::Stop
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torrent::fs::DefaultTorrentFileStorage;
    use crate::torrent::{Torrent, TorrentFlags, TorrentMetadata};
    use popcorn_fx_core::init_logger;
    use popcorn_fx_core::testing::read_test_file_to_bytes;
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio::runtime::Runtime;

    #[tokio::test]
    async fn test_execute_create_pieces() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent_info_data = read_test_file_to_bytes("debian-udp.torrent");
        let torrent_info = TorrentMetadata::try_from(torrent_info_data.as_slice()).unwrap();
        let runtime = Arc::new(Runtime::new().unwrap());
        let torrent = Torrent::request()
            .metadata(torrent_info)
            .options(TorrentFlags::none())
            .peer_listener_port(9666)
            .storage(Box::new(DefaultTorrentFileStorage::new(temp_path)))
            .runtime(runtime.clone())
            .build()
            .unwrap();
        let inner = torrent.instance().unwrap();
        let operation = Box::new(TorrentCreatePiecesOperation::new()) as Box<dyn TorrentOperation>;

        let result = operation.execute(&inner).await;

        assert_eq!(TorrentOperationResult::Continue, result);
        assert_eq!(
            15237,
            torrent.total_pieces().await,
            "expected the pieces to have been created"
        );
    }

    #[tokio::test]
    async fn test_execute_pieces_already_exist() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent_info_data = read_test_file_to_bytes("debian-udp.torrent");
        let torrent_info = TorrentMetadata::try_from(torrent_info_data.as_slice()).unwrap();
        let torrent = Torrent::request()
            .metadata(torrent_info.clone())
            .options(TorrentFlags::none())
            .peer_listener_port(8080)
            .storage(Box::new(DefaultTorrentFileStorage::new(temp_path)))
            .build()
            .unwrap();
        let inner = torrent.instance().unwrap();
        let operation = TorrentCreatePiecesOperation::new();

        inner
            .update_pieces(vec![Piece::new(torrent_info.info_hash.clone(), 0, 0, 1024)])
            .await;
        let result = operation.execute(&inner).await;

        assert_eq!(TorrentOperationResult::Continue, result);
        assert_eq!(
            1,
            torrent.total_pieces().await,
            "expected the pieces to not have been updated"
        );
    }
}
