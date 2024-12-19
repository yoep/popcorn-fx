use crate::torrent::{
    File, TorrentCommandEvent, TorrentContext, TorrentOperation, TorrentOperationResult,
    TorrentState,
};
use async_trait::async_trait;
use futures::future;
use log::{debug, trace};
use std::sync::Arc;
use std::time::Instant;
use tokio::select;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

const CHUNK_VALIDATION_SIZE: usize = 400;

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
        let state = self.state.clone();
        let context = torrent.clone();
        let runtime = torrent.runtime().clone();

        torrent.runtime().spawn(async move {
            debug!(
                "Torrent {} is validating files {:?}",
                context,
                files
                    .iter()
                    .map(|e| e.path.to_string_lossy())
                    .collect::<Vec<_>>(),
            );

            let start = Instant::now();
            let cancellation_token = context.cancellation_token();
            let futures: Vec<_> = files
                .into_iter()
                .map(|file| {
                    runtime.spawn(Self::validate_file(
                        context.clone(),
                        file,
                        cancellation_token.clone(),
                    ))
                })
                .collect();

            select! {
                _ = cancellation_token.cancelled() => return,
                _ = future::join_all(futures) => {},
            }

            let time_taken = start.elapsed();
            debug!(
                "Torrent {} has completed file validation in {}.{:03} seconds",
                context,
                time_taken.as_secs(),
                time_taken.subsec_millis()
            );
            *state.lock().await = ValidationState::Validated;
            let new_state = context.determine_state().await;
            context.send_command_event(TorrentCommandEvent::State(new_state));
        });

        *self.state.lock().await = ValidationState::Validating;
    }

    async fn validate_file(
        torrent: Arc<TorrentContext>,
        file: File,
        cancellation_token: CancellationToken,
    ) {
        let mut pieces = torrent.file_pieces(&file).await;
        let total_chunks = (pieces.len() + 1) / CHUNK_VALIDATION_SIZE;
        let total_pieces = pieces.len();
        let mut valid_pieces = 0;

        trace!(
            "Torrent {} is validating {} pieces for file {:?}",
            torrent,
            pieces.len(),
            file.path
        );
        for chunk in 0..total_chunks {
            // check if the torrent is closing
            if cancellation_token.is_cancelled() {
                trace!(
                    "Torrent {} is closing, interrupting validation of {:?}",
                    torrent,
                    file.path
                );
                break;
            }

            let range_start = chunk * CHUNK_VALIDATION_SIZE;
            let range_end = (chunk + 1) * CHUNK_VALIDATION_SIZE;
            trace!(
                "Torrent {} is validating chunk [{}-{}]/{} for {:?}",
                torrent,
                range_start,
                range_end,
                total_pieces,
                file.path
            );
            let mut completed_pieces = Vec::new();
            let end = pieces.len().min(CHUNK_VALIDATION_SIZE);

            for piece in pieces.drain(..end) {
                // retrieve the piece data
                if let Ok(piece_data) = torrent.read_file_piece(&file, piece.index).await {
                    let is_valid = torrent.validate_piece_data(piece.index, &piece_data).await;

                    if is_valid {
                        valid_pieces += 1;
                        completed_pieces.push(piece.index);
                    }
                }
            }

            // do intermediate update
            if !completed_pieces.is_empty() {
                torrent.pieces_completed(completed_pieces).await;
            }
        }

        debug!(
            "Torrent {} validated {:?} with {} valid pieces",
            torrent, file.path, valid_pieces
        );
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
    use crate::torrent::operation::{TorrentCreateFilesOperation, TorrentCreatePiecesOperation};
    use crate::torrent::TorrentFlags;
    use popcorn_fx_core::init_logger;
    use popcorn_fx_core::testing::copy_test_file;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::time;

    #[test]
    fn test_execute() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        copy_test_file(
            temp_path,
            "piece-1_30.iso",
            Some("debian-12.4.0-amd64-DVD-1.iso"),
        );
        let torrent = create_torrent!("debian-udp.torrent", temp_path, TorrentFlags::None, vec![]);
        let context = torrent.instance().unwrap();
        let runtime = context.runtime();
        let operation = TorrentFileValidationOperation::new();

        // create pieces & files
        runtime.block_on(async {
            let piece_operation = TorrentCreatePiecesOperation::new();
            let file_operation = TorrentCreateFilesOperation::new();

            // create the pieces and files
            let _ = piece_operation.execute(&context).await;
            let _ = file_operation.execute(&context).await;
        });

        // validate the file
        runtime.block_on(async {
            let mut result: TorrentOperationResult;
            let mut attempts = 0;

            loop {
                result = operation.execute(&context).await;
                if result == TorrentOperationResult::Continue || attempts > 100 {
                    break;
                }

                attempts += 1;
                time::sleep(Duration::from_millis(50)).await;
            }

            assert_eq!(TorrentOperationResult::Continue, result);
        });

        runtime.block_on(async {
            let pieces = context.pieces_lock().read().await;
            for piece in 0..30 {
                assert_eq!(
                    true,
                    pieces.get(piece).unwrap().is_completed(),
                    "expected piece {} to be completed",
                    piece
                );
                assert_eq!(
                    true,
                    context.has_piece(piece).await,
                    "expected piece bitfield {} to be completed",
                    piece
                );
            }

            let result = context.stats().await;
            assert_eq!(
                30, result.completed_pieces,
                "expected completed pieces to be 30"
            );
            assert_ne!(
                0, result.total_completed_size,
                "expected total completed size to be > 0"
            );
        });
    }
}
