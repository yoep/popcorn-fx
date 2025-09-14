use crate::torrent::{
    File, Piece, PieceIndex, TorrentCommandEvent, TorrentContext, TorrentOperation,
    TorrentOperationResult, TorrentState,
};
use async_trait::async_trait;
use futures::{stream, StreamExt};
use log::{debug, info, warn};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

/// The maximum number of bytes to validate at once
const CHUNK_VALIDATION_MAX_BYTE_SIZE: usize = 50 * 1000 * 1000; // 50MB

#[derive(Debug, PartialEq)]
enum ValidationState {
    None,
    Validating,
    Validated,
}

/// The torrent file validation operation validates existing files of the torrent and checks which pieces have been completed before/valid.
#[derive(Debug)]
pub struct TorrentFileValidationOperation {
    state: Arc<Mutex<ValidationState>>,
}

impl TorrentFileValidationOperation {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ValidationState::None)),
        }
    }

    async fn validate_files(&self, torrent: &Arc<TorrentContext>, files: Vec<File>) {
        let info_hash = torrent.metadata_lock().read().await.info_hash.clone();
        let state = self.state.clone();
        let context = torrent.clone();

        // stop announcing the torrent
        torrent.tracker_manager().stop_announcing(&info_hash);

        tokio::spawn(async move {
            let pieces = context.piece_pool().pieces().await;
            let piece_len = pieces.get(0).map(|e| e.len()).unwrap_or_default();

            if pieces.len() > 0 {
                debug!(
                    "Torrent {} is validating files {:?}",
                    context,
                    files
                        .iter()
                        .map(|e| e.torrent_path.to_string_lossy())
                        .collect::<Vec<_>>(),
                );

                let max_parallel = CHUNK_VALIDATION_MAX_BYTE_SIZE / piece_len;

                let start = Instant::now();
                let futures: Vec<_> = pieces
                    .into_iter()
                    .map(|piece| Self::validate_piece(context.clone(), piece))
                    .collect();

                let valid_pieces = stream::iter(futures)
                    .buffer_unordered(max_parallel)
                    .collect::<Vec<_>>()
                    .await
                    .into_iter()
                    .flat_map(|e| e)
                    .collect::<Vec<_>>();

                let time_taken = start.elapsed();
                info!(
                    "Torrent {} completed {} file validation(s) ({} valid chunks) in {}.{:03} seconds",
                    context,
                    files.len(),
                    valid_pieces.len(),
                    time_taken.as_secs(),
                    time_taken.subsec_millis()
                );

                context.pieces_completed(valid_pieces).await;
            } else {
                warn!(
                    "Torrent {} failed to start file validation, pieces are unknown",
                    context
                );
            }

            // start announcing the torrent again
            context.tracker_manager().start_announcing(&info_hash);

            *state.lock().await = ValidationState::Validated;
            let new_state = context.determine_state().await;
            context.send_command_event(TorrentCommandEvent::State(new_state));
        });

        *self.state.lock().await = ValidationState::Validating;
    }

    /// Validate the piece data stored within the [crate::torrent::storage::Storage] of the torrent.
    /// Returns the [PieceIndex] when the stored piece data is valid, else [None].
    async fn validate_piece(context: Arc<TorrentContext>, piece: Piece) -> Option<PieceIndex> {
        if let Some(hash_v1) = piece.hash.hash_v1() {
            return context
                .storage()
                .hash_v1(&piece.index)
                .await
                .ok()
                .and_then(|hash| Some(piece.index).filter(|_| hash == hash_v1));
        }

        if let Some(hash_v2) = piece.hash.hash_v2() {
            return context
                .storage()
                .hash_v2(&piece.index)
                .await
                .ok()
                .and_then(|hash| Some(piece.index).filter(|_| hash_v2 == hash));
        }

        debug!(
            "Torrent {} is unable to validate piece {}, piece hash is missing or invalid",
            context, piece.index
        );
        None
    }
}

#[async_trait]
impl TorrentOperation for TorrentFileValidationOperation {
    fn name(&self) -> &str {
        "torrent file validation operation"
    }

    async fn execute(&self, torrent: &Arc<TorrentContext>) -> TorrentOperationResult {
        // check the current state of the validator
        match *self.state.lock().await {
            ValidationState::Validated => return TorrentOperationResult::Continue,
            ValidationState::Validating => return TorrentOperationResult::Stop,
            _ => {}
        }

        let files = torrent.files().await;

        if files.len() > 0 {
            torrent.update_state(TorrentState::CheckingFiles).await;
            self.validate_files(torrent, files).await;
            return TorrentOperationResult::Stop;
        }

        TorrentOperationResult::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_torrent;
    use crate::init_logger;
    use crate::torrent::operation::{TorrentCreateFilesOperation, TorrentCreatePiecesOperation};
    use popcorn_fx_core::testing::copy_test_file;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::{select, time};

    #[tokio::test]
    async fn test_execute_state_validating() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![],
            vec![]
        );
        let context = torrent.instance().unwrap();
        let operation = TorrentFileValidationOperation::new();

        *operation.state.lock().await = ValidationState::Validating;
        let result = operation.execute(&context).await;

        assert_eq!(TorrentOperationResult::Stop, result);
    }

    #[tokio::test]
    async fn test_execute_state_validated() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![],
            vec![]
        );
        let context = torrent.instance().unwrap();
        let operation = TorrentFileValidationOperation::new();

        *operation.state.lock().await = ValidationState::Validated;
        let result = operation.execute(&context).await;

        assert_eq!(TorrentOperationResult::Continue, result);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_execute() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        copy_test_file(
            temp_path,
            "piece-1_30.iso",
            Some("debian-12.4.0-amd64-DVD-1.iso"),
        );
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![],
            vec![]
        );
        let context = torrent.instance().unwrap();
        let operation = TorrentFileValidationOperation::new();

        // create pieces & files
        create_pieces_and_files(&context).await;

        // validate the file
        select! {
            _ = time::sleep(Duration::from_secs(25)) => {},
            _ = async {
                loop {
                    if operation.execute(&context).await == TorrentOperationResult::Continue {
                        break;
                    }
                    time::sleep(Duration::from_millis(50)).await;
                }
            } => {},
        }

        let result = operation.execute(&context).await;
        assert_eq!(TorrentOperationResult::Continue, result);

        let pieces = context.piece_pool().pieces().await;
        for piece in 0..30 {
            assert_eq!(
                true,
                pieces.get(piece).unwrap().is_completed(),
                "expected piece {} to be completed",
                piece
            );
            assert_eq!(
                true,
                context.piece_pool().is_piece_completed(&piece).await,
                "expected piece bitfield {} to be completed",
                piece
            );

            let result = context.metrics().await;
            assert_eq!(
                30,
                result.completed_pieces.get(),
                "expected completed pieces to be 30"
            );
            assert_ne!(
                0,
                result.completed_size.get(),
                "expected total completed size to be > 0"
            );
        }
    }

    async fn create_pieces_and_files(context: &Arc<TorrentContext>) {
        let piece_operation = TorrentCreatePiecesOperation::new();
        let file_operation = TorrentCreateFilesOperation::new();

        // create the pieces and files
        let _ = piece_operation.execute(&context).await;
        let _ = file_operation.execute(&context).await;
    }
}
