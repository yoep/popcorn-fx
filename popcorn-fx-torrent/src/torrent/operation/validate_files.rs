use crate::torrent::{
    File, Piece, PieceIndex, TorrentCommandEvent, TorrentContext, TorrentOperation,
    TorrentOperationResult, TorrentState,
};
use async_trait::async_trait;
use futures::future;
use log::{debug, error, info, trace, warn};
use std::cmp::min;
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
            debug!(
                "Torrent {} is validating files {:?}",
                context,
                files
                    .iter()
                    .map(|e| e.path.to_string_lossy())
                    .collect::<Vec<_>>(),
            );

            let start = Instant::now();
            let total_files = files.len();
            let futures: Vec<_> = files
                .into_iter()
                .filter(|file| context.file_exists(file))
                .map(|file| runtime.spawn(Self::validate_file(context.clone(), file)))
                .collect();

            select! {
                _ = context.cancelled() => return,
                _ = future::join_all(futures) => {},
            }

            let time_taken = start.elapsed();
            info!(
                "Torrent {} completed {} file validation(s) in {}.{:03} seconds",
                context,
                total_files,
                time_taken.as_secs(),
                time_taken.subsec_millis()
            );
            *state.lock().await = ValidationState::Validated;
            let new_state = context.determine_state().await;
            context.send_command_event(TorrentCommandEvent::State(new_state));
        });

        *self.state.lock().await = ValidationState::Validating;
    }

    /// Validate the given file of the torrent.
    /// The file will be validated into multiple parallel chunks.
    async fn validate_file(torrent: Arc<TorrentContext>, file: File) {
        let mut pieces = torrent.file_pieces(&file).await;
        let (total_chunks, pieces_per_chunk) = Self::calculate_chunks(pieces.as_slice());
        let total_pieces = pieces.len();

        trace!(
            "Torrent {} is validating {} pieces for file {:?}",
            torrent,
            pieces.len(),
            file.path
        );

        let mut total_valid_pieces = 0;
        for chunk in 0..total_chunks {
            let end = pieces.len().min(pieces_per_chunk);

            select! {
                _ = torrent.cancelled() => return,
                valid_pieces = Self::validate_file_chunk(
                    &torrent,
                    &file,
                    pieces_per_chunk * chunk,
                    pieces_per_chunk,
                    total_pieces,
                    pieces.drain(..end).collect(),
                ) => total_valid_pieces += valid_pieces,
            }
        }

        debug!(
            "Torrent {} validated {:?} with {} valid pieces",
            torrent, file.path, total_valid_pieces
        );
    }

    /// Validate a chunk of pieces for the given file.
    /// At the end of the chunk validation, an intermediate update will be sent to the torrent.
    async fn validate_file_chunk(
        context: &Arc<TorrentContext>,
        file: &File,
        chunk_offset: usize,
        pieces_per_chunk: usize,
        total_pieces: usize,
        pieces: Vec<Piece>,
    ) -> usize {
        let range_start = pieces.first().map(|e| e.offset).unwrap_or(0);
        let range_end = pieces
            .last()
            .map(|e| e.offset + e.length)
            .unwrap_or(0)
            .min(file.length); // make sure we don't exceed the file size if the last piece overlaps with multiple files
        let pieces_end = chunk_offset + pieces_per_chunk.min(pieces.len());

        match context
            .read_file_bytes_with_padding(file, range_start..range_end)
            .await
        {
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
                        context.runtime().spawn(async move {
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
                    _ = context.cancelled() => return 0,
                    result = future::join_all(futures) => {
                        completed_pieces = result.into_iter()
                            .flat_map(|e| e
                                .map_err(|err| error!("Torrent {} validation error, {}", context, err))
                                .ok())
                            .filter(|(e, _)| *e)
                            .map(|(_, piece)| piece.index)
                            .collect();
                    },
                }
                let elapsed = start_time.elapsed();
                trace!(
                    "Torrent {} validated chunk [{}-{}]/{} for {:?} in {}.{:03}ms",
                    context,
                    chunk_offset,
                    pieces_end,
                    total_pieces,
                    file.path,
                    elapsed.as_millis(),
                    elapsed.subsec_micros() % 1000
                );
                let valid_pieces = completed_pieces.len();

                // inform the torrent about the validated pieces of this chunk
                if !completed_pieces.is_empty() {
                    context.pieces_completed(completed_pieces).await;
                }

                valid_pieces
            }
            Err(e) => {
                warn!(
                    "Torrent {} failed to validate chunk [{}-{}]/{} for {:?}, {}",
                    context, chunk_offset, pieces_end, total_pieces, file.path, e
                );
                0
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
