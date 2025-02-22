use crate::torrent::{
    File, Piece, PieceIndex, TorrentCommandEvent, TorrentContext, TorrentOperation,
    TorrentOperationResult, TorrentState,
};
use async_trait::async_trait;
use futures::future;
use log::{debug, error, info, trace, warn};
use std::cmp::min;
use std::ops::Range;
use std::sync::Arc;
use std::time::Instant;
use tokio::select;
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
        let state = self.state.clone();
        let context = torrent.clone();
        let runtime = torrent.runtime().clone();

        torrent.runtime().spawn(async move {
            if let Some(pieces) = context.pieces().await {
                debug!(
                    "Torrent {} is validating files {:?}",
                    context,
                    files
                        .iter()
                        .map(|e| e.torrent_path.to_string_lossy())
                        .collect::<Vec<_>>(),
                );

                let start = Instant::now();
                let (total_chunks, pieces_per_chunk) = Self::calculate_chunks(pieces.as_slice());
                let futures: Vec<_> = (0..total_chunks)
                    .into_iter()
                    .map(|chunk| {
                        runtime.spawn(Self::validate_chunk(
                            context.clone(),
                            chunk,
                            pieces_per_chunk,
                        ))
                    })
                    .collect();

                select! {
                    _ = context.cancelled() => return,
                    _ = future::join_all(futures) => {},
                }

                let time_taken = start.elapsed();
                info!(
                    "Torrent {} completed {} file validation(s) in {}.{:03} seconds",
                    context,
                    files.len(),
                    time_taken.as_secs(),
                    time_taken.subsec_millis()
                );
            } else {
                warn!(
                    "Torrent {} failed to start file validation, pieces are unknown",
                    context
                );
            }

            *state.lock().await = ValidationState::Validated;
            let new_state = context.determine_state().await;
            context.send_command_event(TorrentCommandEvent::State(new_state));
        });

        *self.state.lock().await = ValidationState::Validating;
    }

    async fn validate_chunk(torrent: Arc<TorrentContext>, chunk: usize, pieces_per_chunk: usize) {
        let mut pieces: Vec<Piece>;
        let piece_range: Range<usize>;
        let total_pieces: usize;

        {
            let mutex = torrent.pieces_lock().read().await;
            total_pieces = mutex.len();
            piece_range =
                chunk * pieces_per_chunk..((chunk + 1) * pieces_per_chunk).min(total_pieces);
            pieces = torrent.pieces_lock().read().await.as_slice()[piece_range]
                .iter()
                .map(|e| e.clone())
                .collect()
        }

        let mut valid_pieces = 0;
        let range_start = pieces.first().map(|e| e.torrent_range().start).unwrap_or(0);
        let range_end = pieces.last().map(|e| e.torrent_range().end).unwrap_or(0);
        let torrent_range = range_start..range_end;

        match torrent.read_bytes_with_padding(torrent_range.clone()).await {
            Ok(bytes) => {
                let start_time = Instant::now();
                let futures: Vec<_> = pieces
                    .into_iter()
                    .filter_map(|piece| {
                        let start = piece.offset - range_start;
                        let end = start + piece.length;
                        let piece_bytes = &bytes[start..end];

                        if piece_bytes == &vec![0u8; piece.length] {
                            return None;
                        }

                        Some((piece, piece_bytes.to_vec()))
                    })
                    .map(|(piece, piece_bytes)| {
                        torrent.runtime().spawn(async move {
                            (
                                TorrentContext::validate_piece_data(&piece, &piece_bytes),
                                piece,
                            )
                        })
                    })
                    .collect();
                // cleanup the read bytes
                drop(bytes);

                let completed_pieces: Vec<PieceIndex>;
                select! {
                    _ = torrent.cancelled() => return,
                    result = future::join_all(futures) => {
                        completed_pieces = result.into_iter()
                            .flat_map(|e| e
                                .map_err(|err| error!("Torrent {} validation error, {}", torrent, err))
                                .ok())
                            .filter(|(e, _)| *e)
                            .map(|(_, piece)| piece.index)
                            .collect();
                    },
                }
                let elapsed = start_time.elapsed();
                trace!(
                    "Torrent {} validated chunk {:?} in {}.{:03}ms",
                    torrent,
                    torrent_range,
                    elapsed.as_millis(),
                    elapsed.subsec_micros() % 1000
                );
                valid_pieces += completed_pieces.len();

                // inform the torrent about the validated pieces of this chunk
                if !completed_pieces.is_empty() {
                    torrent.pieces_completed(completed_pieces).await;
                }
            }
            Err(e) => {
                warn!(
                    "Torrent {} failed to validate chunk {:?}, {}",
                    torrent, torrent_range, e
                );
            }
        }
    }

    /// Calculate the validation chunk size for the given pieces.
    /// The maximum amount of pieces per chunk, based on memory size, will be calculated together with the amount of total chunks.
    ///
    /// It returns the total chunks and pieces per chunk.
    fn calculate_chunks(pieces: &[Piece]) -> (usize, usize) {
        let piece_length = pieces.get(0).map(|e| e.length).unwrap_or(16384);
        let max_pieces_per_chunk = CHUNK_VALIDATION_MAX_BYTE_SIZE / piece_length;

        let pieces_per_chunk = min(max_pieces_per_chunk, pieces.len());
        let total_chunks = (pieces.len() + pieces_per_chunk - 1) / pieces_per_chunk;

        (total_chunks, pieces_per_chunk)
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
    use crate::torrent::{TorrentConfig, TorrentFlags};
    use popcorn_fx_core::init_logger;
    use popcorn_fx_core::testing::copy_test_file;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::runtime::Runtime;
    use tokio::time;

    #[test]
    fn test_execute_state_validating() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![]
        );
        let context = torrent.instance().unwrap();
        let runtime = context.runtime();
        let operation = TorrentFileValidationOperation::new();

        runtime.block_on(async {
            *operation.state.lock().await = ValidationState::Validating;
        });
        let result = runtime.block_on(operation.execute(&context));

        assert_eq!(TorrentOperationResult::Stop, result);
    }

    #[test]
    fn test_execute_state_validated() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![]
        );
        let context = torrent.instance().unwrap();
        let runtime = context.runtime();
        let operation = TorrentFileValidationOperation::new();

        runtime.block_on(async {
            *operation.state.lock().await = ValidationState::Validated;
        });
        let result = runtime.block_on(operation.execute(&context));

        assert_eq!(TorrentOperationResult::Continue, result);
    }

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
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![]
        );
        let context = torrent.instance().unwrap();
        let runtime = context.runtime();
        let operation = TorrentFileValidationOperation::new();

        // create pieces & files
        create_pieces_and_files(&context, runtime);

        // validate the file
        runtime.block_on(async {
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
            };

            let result = operation.execute(&context).await;
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

    #[test]
    fn test_calculate_chunks() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![]
        );
        let context = torrent.instance().unwrap();
        let runtime = context.runtime();

        // create pieces & files
        create_pieces_and_files(&context, runtime);

        // get the pieces from the torrent and calculate the chunks
        let pieces = runtime.block_on(context.pieces_lock().read());
        let (total_chunks, pieces_per_chunk) =
            TorrentFileValidationOperation::calculate_chunks(pieces.as_slice());

        assert_eq!(81, total_chunks);
        assert_eq!(190, pieces_per_chunk);
    }

    fn create_pieces_and_files(context: &Arc<TorrentContext>, runtime: &Arc<Runtime>) {
        runtime.block_on(async {
            let piece_operation = TorrentCreatePiecesOperation::new();
            let file_operation = TorrentCreateFilesOperation::new();

            // create the pieces and files
            let _ = piece_operation.execute(&context).await;
            let _ = file_operation.execute(&context).await;
        });
    }
}
