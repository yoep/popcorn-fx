use crate::torrent::errors::Result;
use crate::torrent::{
    File, TorrentContext, TorrentError, TorrentOperation, TorrentOperationResult, TorrentState,
};
use async_trait::async_trait;
use log::{debug, warn};
use std::path::PathBuf;
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

    /// Try to create the files of the torrent.
    /// This operation doesn't store the created files within this torrent.
    async fn try_create_files(&self, torrent: &TorrentContext) -> Result<Vec<File>> {
        let info = torrent.metadata().await;
        let is_v2_metadata: bool = info.info_hash.has_v2();
        let metadata = info.info.ok_or(TorrentError::InvalidMetadata(
            "metadata is missing".to_string(),
        ))?;

        let mut offset = 0;
        let mut files = vec![];

        for (index, file_info) in metadata.files().into_iter().enumerate() {
            let file_length = file_info.length as usize;
            let torrent_path = metadata.path(&file_info);
            let io_path = Self::io_path(torrent, &torrent_path);

            files.push(File {
                index,
                torrent_path,
                io_path,
                offset,
                info: file_info,
                priority: Default::default(),
            });

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

    /// Get the filepath of the file within the storage device.
    /// This returns the absolute path of the file within the storage device.
    fn io_path(torrent: &TorrentContext, torrent_path: &PathBuf) -> PathBuf {
        PathBuf::from(torrent.storage_path()).join(torrent_path.as_path())
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
    use crate::torrent::{TorrentConfig, TorrentFlags};
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
    async fn test_io_path() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent_path = PathBuf::from("MyTorrentDir/TorrentFile.mp4");
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::default(),
            vec![],
            vec![]
        );
        let context = torrent.instance().unwrap();
        let expected_result = temp_dir.path().join(torrent_path.as_path());

        let result = TorrentCreateFilesOperation::io_path(&*context, &torrent_path);

        assert_eq!(expected_result, result);
    }
}
