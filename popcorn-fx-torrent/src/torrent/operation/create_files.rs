use crate::torrent::errors::Result;
use crate::torrent::{
    File, TorrentContext, TorrentError, TorrentFileInfo, TorrentMetadataInfo, TorrentOperation,
    TorrentOperationResult, TorrentState,
};
use async_trait::async_trait;
use log::{debug, warn};
use std::sync::Arc;

#[derive(Debug)]
pub struct TorrentCreateFilesOperation;

impl TorrentCreateFilesOperation {
    pub fn new() -> Self {
        Self {}
    }

    /// Create the torrent files information.
    /// This can only be executed when the torrent metadata is known.
    async fn create_files(&self, torrent: &TorrentContext) -> bool {
        torrent.update_state(TorrentState::Initializing).await;

        match self.try_create_files(torrent).await {
            Ok(files) => {
                let total_files = files.len();
                torrent.update_files(files).await;
                debug!(
                    "A total of {} files have been created for {}",
                    total_files, torrent
                );
                true
            }
            Err(e) => {
                warn!("Failed to create torrent files of {}, {}", torrent, e);
                false
            }
        }
    }

    /// Try to create the file information of the torrent.
    /// This operation doesn't store the created files within this torrent.
    async fn try_create_files(&self, torrent: &TorrentContext) -> Result<Vec<File>> {
        let info = torrent.metadata_lock().read().await;
        let is_v2_metadata: bool = info.info_hash.has_v2();
        let metadata = info.info.as_ref().ok_or(TorrentError::InvalidMetadata(
            "metadata is missing".to_string(),
        ))?;

        let mut offset = 0usize;
        let mut files = vec![];

        for (index, file_info) in metadata.files().into_iter().enumerate() {
            let file_length = file_info.length as usize;

            files.push(Self::create_file(file_info, &metadata, index, offset));

            if is_v2_metadata {
                offset = (offset + metadata.piece_length as usize - 1)
                    / metadata.piece_length as usize
                    * metadata.piece_length as usize;
            } else {
                offset += file_length;
            }
        }

        Ok(files)
    }

    fn create_file(
        file_info: TorrentFileInfo,
        metadata: &TorrentMetadataInfo,
        index: usize,
        offset: usize,
    ) -> File {
        let file_length = file_info.length as usize;
        let piece_len = metadata.piece_length as usize;
        let torrent_path = metadata.path(&file_info);
        let file_piece_start = offset / piece_len;
        let file_piece_end = offset.saturating_add(file_length + 1) / piece_len;

        File {
            index,
            torrent_path,
            torrent_offset: offset,
            info: file_info,
            priority: Default::default(),
            pieces: file_piece_start..file_piece_end + 1, // as the range is exclusive, add 1 to the end range
        }
    }
}

#[async_trait]
impl TorrentOperation for TorrentCreateFilesOperation {
    fn name(&self) -> &str {
        "create torrent files operation"
    }

    async fn execute(&self, torrent: &Arc<TorrentContext>) -> TorrentOperationResult {
        // check if the files have already been created
        // if so, continue the chain
        if torrent.total_files().await > 0 || self.create_files(&torrent).await {
            return TorrentOperationResult::Continue;
        }

        TorrentOperationResult::Stop
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_torrent;
    use crate::init_logger;
    use crate::torrent::operation::TorrentCreatePiecesOperation;
    use crate::torrent::{PieceIndex, TorrentConfig, TorrentFlags};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_execute() {
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
        let create_pieces = TorrentCreatePiecesOperation::new();
        let operation = TorrentCreateFilesOperation::new();

        let result = create_pieces.execute(&context).await;
        assert_eq!(
            TorrentOperationResult::Continue,
            result,
            "expected the pieces to have been created"
        );

        let result = operation.execute(&context).await;
        assert_eq!(
            TorrentOperationResult::Continue,
            result,
            "expected the operations chain to continue"
        );
        assert_eq!(
            1,
            context.total_files().await,
            "expected the files to have been created"
        );
    }

    #[tokio::test]
    async fn test_execute_no_metadata() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let uri = "magnet:?xt=urn:btih:EADAF0EFEA39406914414D359E0EA16416409BD7&dn=debian-12.4.0-amd64-DVD-1.iso&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce&tr=udp%3A%2F%2Ftracker.bittor.pw%3A1337%2Fannounce&tr=udp%3A%2F%2Fpublic.popcorn-tracker.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969&tr=udp%3A%2F%2Fopen.demonii.com%3A1337%2Fannounce";
        let torrent = create_torrent!(
            uri,
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![],
            vec![]
        );
        let context = torrent.instance().unwrap();
        let operation = TorrentCreateFilesOperation::new();

        let result = operation.execute(&context).await;

        assert_eq!(
            TorrentOperationResult::Stop,
            result,
            "expected the operations chain to stop"
        );
    }

    #[tokio::test]
    async fn test_try_create_files() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "multifile.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![],
            vec![]
        );
        let pieces_operation = TorrentCreatePiecesOperation::new();
        let operation = TorrentCreateFilesOperation::new();
        let context = torrent.instance().unwrap();

        // create the torrent pieces
        let result = pieces_operation.execute(&context).await;
        assert_eq!(
            TorrentOperationResult::Continue,
            result,
            "expected the pieces operation to have succeeded"
        );
        let pieces = context
            .pieces()
            .await
            .expect("expected the pieces to have been created");

        // create the torrent files
        let files = operation
            .try_create_files(&context)
            .await
            .expect("failed to create torrent files");

        let mut previous_end_piece = 0 as PieceIndex;
        for file in files {
            let start_piece = file.pieces.start;
            let end_piece = file.pieces.end;

            // the file should either start at the previous and piece or the next one
            assert!(
                start_piece == previous_end_piece.saturating_sub(1usize)
                    || start_piece == previous_end_piece,
                "expected the start piece {} to continue from previous end piece {}",
                start_piece,
                previous_end_piece
            );
            previous_end_piece = end_piece;
        }

        // check if the piece range from the last file ends at the last piece of the torrent
        assert_eq!(
            pieces[pieces.len() - 1].index,
            previous_end_piece.saturating_sub(1usize),
            "expected the last file to end on the last piece of the torrent"
        );
    }

    #[tokio::test]
    async fn test_create_file() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent = create_torrent!(
            "multifile.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![],
            vec![]
        );
        let context = torrent.instance().unwrap();
        let metadata = context.metadata().await;
        let metadata_info = metadata
            .info
            .as_ref()
            .expect("expected the metadata info to be present");
        let metadata_files = metadata_info.files();

        let result = TorrentCreateFilesOperation::create_file(
            metadata_files.get(1).unwrap().clone(),
            &metadata_info,
            1,
            3364128518,
        );

        assert_eq!(
            401, result.pieces.start,
            "expected the starting piece to match"
        );
        assert_eq!(725, result.pieces.end, "expected the ending piece to match");
    }
}
