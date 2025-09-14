use crate::torrent::storage::parts_file::PartsFile;
use crate::torrent::storage::{Error, Metrics, Result, Storage};
use crate::torrent::torrent_pools::FilePool;
use crate::torrent::{
    FileAttributeFlags, FileIndex, FilePriority, InfoHash, PieceIndex, Sha1Hash, Sha256Hash,
};
use async_trait::async_trait;
use sha1::{Digest, Sha1};
use sha2::Sha256;
use std::cmp::min;
use std::io;
use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use tokio::fs::{create_dir_all, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::sync::RwLock;

/// File system storage for the torrent piece data.
#[derive(Debug)]
pub struct DiskStorage {
    path: RwLock<PathBuf>,
    files: FilePool,
    part_file: PartsFile,
    metrics: Metrics,
}

impl DiskStorage {
    pub fn new<P: AsRef<Path>>(info_hash: InfoHash, path: P, files: FilePool) -> Self {
        let part_filename = format!(".{}.parts", hex::encode(info_hash.short_info_hash_bytes()));
        let piece_pool = files.pieces().clone();

        Self {
            path: RwLock::new(path.as_ref().to_path_buf()),
            files,
            part_file: PartsFile::new(part_filename, path, piece_pool),
            metrics: Default::default(),
        }
    }

    /// Get the absolute filepath for the given filepath within the storage.
    async fn absolute_filepath<P: AsRef<Path>>(&self, filepath: P) -> PathBuf {
        self.path.read().await.join(filepath.as_ref())
    }

    /// Get the amount of bytes for the given piece.
    async fn piece_len(&self, piece: &PieceIndex) -> usize {
        self.files
            .pieces()
            .get(piece)
            .await
            .map(|e| e.len())
            .unwrap_or_default()
    }

    /// Get the amount of bytes within the torrent.
    async fn torrent_len(&self) -> usize {
        let last_piece_index = self.files.pieces().len().await.saturating_sub(1);
        self.files
            .pieces()
            .get(&last_piece_index)
            .await
            .map(|piece| piece.offset.saturating_add(piece.len()))
            .unwrap_or_default()
    }

    /// Check if the given filepath is valid within the storage.
    /// If the target filepath leaves the storage path, it returns false.
    async fn is_valid_filepath<P: AsRef<Path>>(&self, filepath: P) -> bool {
        let base = Self::canonicalize_unchecked(&*self.path.read().await);
        let target = Self::canonicalize_unchecked(filepath.as_ref());

        target.starts_with(&base)
    }

    /// Assert if the given filepath is valid within the storage.
    /// This prevents file paths from traversing upwards/leaving the storage path.
    async fn assert_valid_filepath<P: AsRef<Path>>(&self, filepath: P) -> Result<()> {
        if !self.is_valid_filepath(&filepath).await {
            return Err(Error::InvalidFilepath(filepath.as_ref().to_path_buf()));
        }

        Ok(())
    }

    /// Try to open the given torrent file from the disk storage.
    async fn open<P: AsRef<Path>>(&self, filepath: P, write: bool) -> Result<File> {
        let absolute_path = self.absolute_filepath(filepath).await;
        self.assert_valid_filepath(&absolute_path).await?;

        if write {
            create_dir_all(absolute_path.parent().unwrap_or(&self.path.read().await))
                .await
                .map_err(|e| Error::Io(e))?;
        }

        Ok(OpenOptions::new()
            .read(true)
            .write(write)
            .create(write)
            .open(absolute_path)
            .await?)
    }

    /// The readwrite file index and torrent offset to start from for the given piece and offset.
    async fn readwrite(
        &self,
        piece: &PieceIndex,
        offset: usize,
        buffer_len: usize,
    ) -> Result<(FileIndex, usize)> {
        let torrent_offset = self
            .files
            .pieces()
            .get(piece)
            .await
            .map(|piece| piece.offset.saturating_add(offset))
            .ok_or(Error::Unavailable)?;

        // check if the requested range is still within the torrent range
        let torrent_len = self.torrent_len().await;
        if torrent_offset + buffer_len > torrent_len {
            return Err(Error::OutOfBounds);
        }

        let file_index = self
            .files
            .file_index_at_offset(torrent_offset)
            .await
            .ok_or(Error::Unavailable)?;

        Ok((file_index, torrent_offset))
    }

    /// Get the canonicalized path for the given path.
    /// This function traverses the path components and resolves ".." and "." appropriately.
    /// It returns the resulting path.
    fn canonicalize_unchecked(path: &Path) -> PathBuf {
        let components = path.components();
        let mut result = PathBuf::new();

        // Traverse the path components and resolve ".." and "." appropriately
        for component in components {
            match component {
                // Ignore "." (current directory)
                std::path::Component::CurDir => {}
                // Remove the last component for ".." (parent directory)
                std::path::Component::ParentDir => {
                    result.pop();
                }
                // Add other components as normal
                std::path::Component::Normal(part) => {
                    result.push(part);
                }
                // Handle other components, like RootDir, if necessary
                _ => {}
            }
        }

        result
    }
}

#[async_trait]
impl Storage for DiskStorage {
    async fn read(&self, buffer: &mut [u8], piece: &PieceIndex, offset: usize) -> Result<usize> {
        let mut cursor = 0;
        let buffer_len = buffer.len();
        let (mut file_index, mut torrent_offset) =
            self.readwrite(piece, offset, buffer_len).await?;

        while cursor < buffer_len {
            let file = self
                .files
                .get(&file_index)
                .await
                .ok_or(Error::Unavailable)?;
            let bytes_remaining = buffer_len - cursor;

            // check if the file is a padding file
            // if so, we skip the bytes as they all yield zero
            if file.attributes().contains(FileAttributeFlags::PaddingFile) {
                cursor += file.len();
                torrent_offset += file.len();
                file_index += 1;
                continue;
            }

            // check if we need to read from the parts file
            if file.priority == FilePriority::None {
                let parts_len = min(bytes_remaining, file.len());
                let mut parts_buffer = vec![0u8; parts_len];

                let bytes_read = self
                    .part_file
                    .read(
                        &mut parts_buffer,
                        &file.pieces.start,
                        torrent_offset.saturating_sub(file.torrent_offset),
                    )
                    .await?;
                buffer[cursor..cursor + bytes_read].copy_from_slice(&parts_buffer[..bytes_read]);
                cursor += bytes_read;
                torrent_offset += bytes_read;
                file_index += 1;
                self.metrics.bytes_read.inc_by(bytes_read as u64);
                continue;
            }

            // try to open the torrent file
            let mut fs_file = self.open(&file.torrent_path, false).await?;
            let start_offset = torrent_offset.saturating_sub(file.torrent_offset);
            fs_file.seek(SeekFrom::Start(start_offset as u64)).await?;
            let bytes_read = fs_file
                .read(&mut buffer[cursor..cursor + bytes_remaining])
                .await?;

            cursor += bytes_read;
            torrent_offset += bytes_read;
            file_index += 1;
            self.metrics.bytes_read.inc_by(bytes_read as u64);

            // check if all bytes of the file are available
            // if not, don't read the next file
            if start_offset + bytes_read < file.len() {
                break;
            }
        }

        Ok(cursor)
    }

    async fn write(&self, data: &[u8], piece: &PieceIndex, offset: usize) -> Result<usize> {
        let mut cursor = 0;
        let data_len = data.len();
        let (mut file_index, mut torrent_offset) = self.readwrite(piece, offset, data_len).await?;

        while cursor < data_len {
            let file = self
                .files
                .get(&file_index)
                .await
                .ok_or(Error::Unavailable)?;
            let bytes_remaining = data_len - cursor;

            // check if the file is a padding file
            // if so, we skip the bytes as they all yield zero
            if file.attributes().contains(FileAttributeFlags::PaddingFile) {
                cursor += file.len();
                torrent_offset += file.len();
                file_index += 1;
                continue;
            }

            // check if we need to write to the parts file
            if file.priority == FilePriority::None {
                let parts_len = min(bytes_remaining, file.len());
                let bytes_written = self
                    .part_file
                    .write(
                        &data[cursor..cursor + parts_len],
                        &file.pieces.start,
                        torrent_offset.saturating_sub(file.torrent_offset),
                    )
                    .await?;

                cursor += bytes_written;
                torrent_offset += bytes_written;
                file_index += 1;
                self.metrics.bytes_written.inc_by(bytes_written as u64);
                continue;
            }

            // try to open the torrent file
            let mut fs_file = self.open(&file.torrent_path, true).await?;
            let file_len = min(bytes_remaining, file.len());
            let start_offset = torrent_offset.saturating_sub(file.torrent_offset);
            fs_file.seek(SeekFrom::Start(start_offset as u64)).await?;
            fs_file.write_all(&data[cursor..cursor + file_len]).await?;
            fs_file.flush().await?;

            cursor += file_len;
            torrent_offset += file_len;
            file_index += 1;
            self.metrics.bytes_written.inc_by(file_len as u64);
        }

        Ok(cursor)
    }

    async fn hash_v1(&self, piece: &PieceIndex) -> Result<Sha1Hash> {
        let len = self.piece_len(piece).await;
        let mut buffer = vec![0u8; len];
        let bytes_read = self.read(&mut buffer, piece, 0).await?;
        if bytes_read != len {
            return Err(Error::Unavailable);
        }

        Sha1Hash::try_from(Sha1::digest(buffer.as_slice()))
            .map_err(|e| Error::Io(io::Error::new(io::ErrorKind::Other, e.to_string())))
    }

    async fn hash_v2(&self, piece: &PieceIndex) -> Result<Sha256Hash> {
        let len = self.piece_len(piece).await;
        let mut buffer = vec![0u8; len];
        let bytes_read = self.read(&mut buffer, piece, 0).await?;
        if bytes_read != len {
            return Err(Error::Unavailable);
        }

        Sha256Hash::try_from(Sha256::digest(buffer.as_slice()))
            .map_err(|e| Error::Io(io::Error::new(io::ErrorKind::Other, e.to_string())))
    }

    async fn move_storage(&self, new_path: &Path) -> Result<()> {
        todo!()
    }

    fn metrics(&self) -> &Metrics {
        &self.metrics
    }
}

impl Drop for DiskStorage {
    fn drop(&mut self) {
        // TODO: cleanup parts file
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torrent::operation::{TorrentCreateFilesOperation, TorrentCreatePiecesOperation};
    use crate::torrent::tests::read_test_file_to_bytes;
    use crate::torrent::{TorrentContext, TorrentOperation};
    use crate::{create_torrent, init_logger};
    use std::sync::Arc;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_read() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let data = read_test_file_to_bytes("piece-1_30.iso");
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::builder().path(temp_path).build(),
            vec![],
            vec![]
        );
        let context = &torrent.instance().unwrap();
        let storage = DiskStorage::new(
            context.metadata_lock().read().await.info_hash.clone(),
            temp_path,
            context.file_pool().clone(),
        );

        // create pieces & files
        create_pieces_and_files(context).await;

        // write the piece data
        {
            let result = storage.write(&data, &0, 0).await;
            assert_eq!(
                Ok(data.len()),
                result,
                "expected all data to have been written to the storage"
            );
        }

        // read the starting piece with offset
        {
            let piece: PieceIndex = 0;
            let offset = 32;
            let piece_len = context.piece_pool().get(&piece).await.unwrap().length;
            let mut buffer = vec![0u8; piece_len];
            let result = storage.read(&mut buffer, &piece, offset).await;
            assert_eq!(
                Ok(piece_len),
                result,
                "expected the buffer to have been fully filled"
            );
            assert_eq!(
                &data[offset..offset + piece_len],
                &buffer[..],
                "expected the buffer to match the original data"
            );
        }

        // read non-starting piece without offset
        {
            let piece: PieceIndex = 7;
            let piece_len = context.piece_pool().get(&piece).await.unwrap().length;
            let mut buffer = vec![0u8; piece_len];
            let result = storage.read(&mut buffer, &piece, 0).await;
            assert_eq!(
                Ok(piece_len),
                result,
                "expected the full piece to have been read"
            );
        }
    }

    #[tokio::test]
    async fn test_hash_v1() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let piece: PieceIndex = 0;
        let data = read_test_file_to_bytes("piece-1.iso");
        let torrent = create_torrent!(
            "debian-udp.torrent",
            temp_path,
            TorrentFlags::none(),
            TorrentConfig::builder().path(temp_path).build(),
            vec![],
            vec![]
        );
        let context = &torrent.instance().unwrap();
        let storage = DiskStorage::new(
            context.metadata_lock().read().await.info_hash.clone(),
            temp_path,
            context.file_pool().clone(),
        );

        // create pieces & files
        create_pieces_and_files(context).await;

        // write the piece data
        {
            let result = storage.write(&data, &piece, 0).await;
            assert_eq!(
                Ok(data.len()),
                result,
                "expected all data to have been written to the storage"
            );
        }

        // get the hash result from the piece
        {
            let piece_hash = context
                .piece_pool()
                .get(&piece)
                .await
                .expect("expected the piece to have been found")
                .hash;
            let expected_hash = piece_hash
                .hash_v1()
                .expect("expected the v1 hash to be present within the piece");

            let result = storage
                .hash_v1(&piece)
                .await
                .expect("expected the hash to have been calculated");
            assert_eq!(
                expected_hash, result,
                "expected the hash to equal the piece hash"
            );
        }
    }

    async fn create_pieces_and_files(context: &Arc<TorrentContext>) {
        let piece_operation = TorrentCreatePiecesOperation::new();
        let file_operation = TorrentCreateFilesOperation::new();
        let _ = piece_operation.execute(context).await;
        let _ = file_operation.execute(context).await;
    }
}
