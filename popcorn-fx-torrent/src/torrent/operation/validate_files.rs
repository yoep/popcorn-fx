use crate::torrent::{File, TorrentContext, TorrentOperation, TorrentState};
use async_trait::async_trait;
use derive_more::Display;
use log::{debug, trace};
use tokio::sync::Mutex;

/// The torrent file validation operation validates existing files of the torrent and checks which pieces have been completed before/valid.
#[derive(Debug, Display)]
#[display(fmt = "torrent file validation operation")]
pub struct TorrentFileValidationOperation {
    validated: Mutex<bool>,
}

impl TorrentFileValidationOperation {
    pub fn new() -> Self {
        Self {
            validated: Mutex::new(false),
        }
    }

    async fn validate_file(&self, torrent: &TorrentContext, file: &File) {
        let pieces = torrent.file_pieces(file).await;
        let mut valid_pieces = 0;

        trace!(
            "Validating {} pieces for file {:?} of {}",
            pieces.len(),
            file,
            torrent
        );
        for piece in pieces.into_iter() {
            // retrieve the piece data
            if let Ok(piece_data) = torrent.read_file_piece(file, piece.index).await {
                let is_valid = torrent.validate_piece_data(piece.index, &piece_data).await;

                if is_valid {
                    valid_pieces += 1;
                    torrent.update_piece_completed(piece.index).await;
                }
            }
        }

        debug!(
            "File {:?} validated with {} valid pieces for {}",
            file.path, valid_pieces, torrent
        );
    }
}

#[async_trait]
impl TorrentOperation for TorrentFileValidationOperation {
    async fn execute<'a>(&self, torrent: &'a TorrentContext) -> Option<&'a TorrentContext> {
        // check if the files have already been validated
        // if so, continue the chain
        if *self.validated.lock().await {
            return Some(torrent);
        }

        let files = torrent.files().await;

        if files.len() > 0 {
            torrent.update_state(TorrentState::CheckingFiles).await;

            trace!("Validating {:?} files for {}", files, torrent);
            for file in files.into_iter() {
                if torrent.file_exists(&file) {
                    debug!("Verifying file {:?} pieces of {}", file, torrent);
                    self.validate_file(&torrent, &file).await;
                } else {
                    debug!("File {:?} not found for {}", file, self)
                }
            }

            *self.validated.lock().await = true;
            torrent.update_state(TorrentState::Downloading).await;
        }

        Some(torrent)
    }

    fn clone_boxed(&self) -> Box<dyn TorrentOperation> {
        Box::new(TorrentFileValidationOperation::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torrent::fs::DefaultTorrentFileStorage;
    use crate::torrent::operation::{TorrentFilesOperation, TorrentPiecesOperation};
    use crate::torrent::{Torrent, TorrentConfig, TorrentInfo};
    use popcorn_fx_core::init_logger;
    use popcorn_fx_core::testing::{copy_test_file, read_test_file_to_bytes};
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::runtime::Runtime;

    #[test]
    fn test_execute() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        copy_test_file(
            temp_path,
            "piece-1_10.iso",
            Some("debian-12.4.0-amd64-DVD-1.iso"),
        );
        let runtime = Arc::new(Runtime::new().unwrap());
        let torrent_info_data = read_test_file_to_bytes("debian-udp.torrent");
        let torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
        let torrent = Torrent::request()
            .metadata(torrent_info)
            .peer_listener_port(6881)
            .config(
                TorrentConfig::builder()
                    .tracker_connection_timeout(Duration::from_secs(1))
                    .build(),
            )
            .storage(Box::new(DefaultTorrentFileStorage::new(temp_path)))
            .operations(vec![])
            .runtime(runtime.clone())
            .build()
            .unwrap();

        runtime.block_on(async {
            let piece_operation = TorrentPiecesOperation::new();
            let file_operation = TorrentFilesOperation::new();
            let operation =
                Box::new(TorrentFileValidationOperation::new()) as Box<dyn TorrentOperation>;
            let inner = torrent.instance().unwrap();

            // create the pieces and files
            let _ = piece_operation.execute(&*inner).await;
            let _ = file_operation.execute(&*inner).await;

            let result = operation.execute(&*inner).await;
            assert_eq!(Some(&*inner), result);

            let pieces = inner.pieces_lock().read().await;
            for piece in 0..9 {
                assert_eq!(
                    true,
                    pieces.get(piece).unwrap().is_completed(),
                    "expected piece {} to be completed",
                    piece
                );
                assert_eq!(
                    true,
                    inner.has_piece(piece).await,
                    "expected piece bitfield {} to be completed",
                    piece
                );
            }
        });
    }
}
