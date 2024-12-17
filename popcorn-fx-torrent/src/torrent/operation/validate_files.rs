use crate::torrent::{
    File, TorrentCommandEvent, TorrentContext, TorrentOperation, TorrentOperationResult,
    TorrentState,
};
use async_trait::async_trait;
use futures::future;
use log::{debug, trace};
use tokio::sync::Mutex;

/// The torrent file validation operation validates existing files of the torrent and checks which pieces have been completed before/valid.
#[derive(Debug)]
pub struct TorrentFileValidationOperation {
    validated: Mutex<bool>,
}

impl TorrentFileValidationOperation {
    pub fn new() -> Self {
        Self {
            validated: Mutex::new(false),
        }
    }

    async fn validate_files(torrent: &TorrentContext, files: Vec<File>) {
        debug!(
            "Validating a total of {} files for {}",
            files.len(),
            torrent
        );
        let futures: Vec<_> = files
            .into_iter()
            .filter(|e| torrent.file_exists(e))
            .map(|e| Self::validate_file(torrent, e))
            .collect();

        future::join_all(futures).await;
    }

    async fn validate_file(torrent: &TorrentContext, file: File) {
        let pieces = torrent.file_pieces(&file).await;
        let mut completed_pieces = Vec::new();
        let mut valid_pieces = 0;

        trace!(
            "Validating {} pieces for file {:?} of {}",
            pieces.len(),
            file,
            torrent
        );
        for piece in pieces.into_iter() {
            // retrieve the piece data
            if let Ok(piece_data) = torrent.read_file_piece(&file, piece.index).await {
                let is_valid = torrent.validate_piece_data(piece.index, &piece_data).await;

                if is_valid {
                    valid_pieces += 1;
                    completed_pieces.push(piece.index);
                }
            }
        }

        debug!(
            "Torrent file {:?} validated with {} valid pieces for {}",
            file.path, valid_pieces, torrent
        );
        torrent.pieces_completed(completed_pieces).await;
    }
}

#[async_trait]
impl TorrentOperation for TorrentFileValidationOperation {
    fn name(&self) -> &str {
        "torrent file validation operation"
    }

    async fn execute(&self, torrent: &TorrentContext) -> TorrentOperationResult {
        // check if the files have already been validated
        // if so, continue the chain
        if *self.validated.lock().await {
            return TorrentOperationResult::Continue;
        }

        let files = torrent.files().await;

        if files.len() > 0 {
            torrent.update_state(TorrentState::CheckingFiles).await;

            Self::validate_files(torrent, files).await;
            *self.validated.lock().await = true;
            debug!("Torrent files of {} have been validated", torrent);

            let new_state = torrent.determine_state().await;
            torrent.send_command_event(TorrentCommandEvent::State(new_state));
        }

        TorrentOperationResult::Continue
    }

    fn clone_boxed(&self) -> Box<dyn TorrentOperation> {
        Box::new(TorrentFileValidationOperation::new())
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
    use tempfile::tempdir;

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

        // create pieces & files
        runtime.block_on(async {
            let piece_operation = TorrentCreatePiecesOperation::new();
            let file_operation = TorrentCreateFilesOperation::new();

            // create the pieces and files
            let _ = piece_operation.execute(&*context).await;
            let _ = file_operation.execute(&*context).await;
        });

        // validate the file
        runtime.block_on(async {
            let operation =
                Box::new(TorrentFileValidationOperation::new()) as Box<dyn TorrentOperation>;

            let result = operation.execute(&*context).await;
            assert_eq!(TorrentOperationResult::Continue, result);

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
