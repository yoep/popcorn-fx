use crate::torrent::{File, TorrentContext, TorrentError, TorrentOperation};
use async_trait::async_trait;
use derive_more::Display;
use log::{debug, warn};
use std::path::PathBuf;

#[derive(Debug, Display)]
#[display(fmt = "create torrent files operation")]
pub struct TorrentFilesOperation {}

impl TorrentFilesOperation {
    pub fn new() -> Self {
        Self {}
    }

    /// Create the torrent files information.
    /// This can only be executed when the torrent metadata is known.
    async fn create_files(&self, torrent: &TorrentContext) -> bool {
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
    async fn try_create_files(
        &self,
        torrent: &TorrentContext,
    ) -> crate::torrent::Result<Vec<File>> {
        let info = torrent.metadata().await;
        let is_v2_metadata: bool = info.info_hash.has_v2();
        let metadata = info.info.ok_or(TorrentError::InvalidMetadata(
            "metadata is missing".to_string(),
        ))?;

        let mut offset = 0;
        let mut files = vec![];

        for (index, file) in metadata.files().into_iter().enumerate() {
            let file_length = file.length as usize;
            let mut path = PathBuf::new().join(metadata.name());

            for path_section in file.path() {
                path = path.join(path_section);
            }

            files.push(File {
                index,
                path,
                offset,
                length: file.length as usize,
                info: file,
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
}

#[async_trait]
impl TorrentOperation for TorrentFilesOperation {
    async fn execute<'a>(&self, torrent: &'a TorrentContext) -> Option<&'a TorrentContext> {
        // check if the files have already been created
        // if so, continue the chain
        if torrent.total_files().await > 0 {
            return Some(torrent);
        }

        // try to create the files
        if self.create_files(&torrent).await {
            return Some(torrent);
        }

        None
    }

    fn clone_boxed(&self) -> Box<dyn TorrentOperation> {
        Box::new(Self::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torrent::fs::DefaultTorrentFileStorage;
    use crate::torrent::operation::{TorrentMetadataOperation, TorrentPiecesOperation};
    use crate::torrent::{Torrent, TorrentInfo};
    use popcorn_fx_core::testing::{init_logger, read_test_file_to_bytes};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_execute() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let torrent_info_data = read_test_file_to_bytes("debian-udp.torrent");
        let torrent_info = TorrentInfo::try_from(torrent_info_data.as_slice()).unwrap();
        let torrent = Torrent::request()
            .metadata(torrent_info)
            .peer_listener_port(6881)
            .operations(vec![
                Box::new(TorrentMetadataOperation::new()),
                Box::new(TorrentPiecesOperation::new()),
            ])
            .storage(Box::new(DefaultTorrentFileStorage::new(temp_path)))
            .build()
            .unwrap();
        let operation = TorrentFilesOperation::new();
        let inner = torrent.instance().unwrap();

        let result = operation.execute(&*inner).await;

        assert_eq!(
            Some(&*inner),
            result,
            "expected the operations chain to continue"
        );
        assert_eq!(
            1,
            inner.total_files().await,
            "expected the files to have been created"
        );
    }
}
