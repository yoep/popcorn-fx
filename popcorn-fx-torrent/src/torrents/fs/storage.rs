use crate::torrents::file::File;
use crate::torrents::fs::{Error, Result};
use crate::torrents::{Piece, PiecePart};
use async_trait::async_trait;
use std::fmt::Debug;
use std::io::SeekFrom;
use std::ops::Range;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

/// Trait for handling the torrent file storage.
/// A storage is always specific to a single torrent.
#[async_trait]
pub trait TorrentFileStorage: Debug + Send + Sync {
    /// Check if the given file exists within the storage.
    /// This doesn't check if the file contains any actual data.
    fn exists(&self, file: &File) -> bool;

    /// Try to create/open the file.
    /// It returns an error if the specified location couldn't be accessed.
    async fn open(&self, file: &File) -> Result<()>;

    /// Write the given data chunk to the torrent file.
    /// The offset is the offset to start from within the file.
    ///
    /// # Arguments
    ///
    /// * `file` - The torrent file to write to.
    /// * `offset` - The offset to start from within the file.
    /// * `data` - The data to write to the file.
    async fn write_piece(&self, file: &File, offset: usize, data: &[u8]) -> Result<()>;

    /// Read a piece from the file storage.
    async fn read_piece(&self, file: &File, piece: &Piece) -> Result<Vec<u8>>;

    /// Read a piece part from the file storage.
    async fn read_piece_part(&self, file: &File, part: &PiecePart) -> Result<Vec<u8>>;

    /// Read the given range of bytes from the file.
    async fn read_bytes(&self, file: &File, range: Range<usize>) -> Result<Vec<u8>>;
}

#[derive(Debug)]
pub struct DefaultTorrentFileStorage {
    base_path: PathBuf,
}

impl DefaultTorrentFileStorage {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// Get the filepath for the given file within the storage.
    /// It returns the absolute path to the file.
    fn get_filepath(&self, file: &File) -> PathBuf {
        self.base_path.join(file.path.clone())
    }

    async fn internal_open(&self, file: &File, writeable: bool) -> Result<tokio::fs::File> {
        let filepath = self.get_filepath(file);

        Ok(tokio::fs::OpenOptions::new()
            .create(true)
            .write(writeable)
            .open(filepath)
            .await?)
    }
}

#[async_trait]
impl TorrentFileStorage for DefaultTorrentFileStorage {
    fn exists(&self, file: &File) -> bool {
        self.get_filepath(file).exists()
    }

    async fn open(&self, file: &File) -> Result<()> {
        let _ = self.internal_open(file, false).await?;
        Ok(())
    }

    async fn write_piece(&self, file: &File, offset: usize, data: &[u8]) -> Result<()> {
        let mut fs_file = self.internal_open(file, true).await?;
        let total_length = offset + data.len();

        if file.length < total_length {
            return Err(Error::Io(format!(
                "data (offset {}, len {}) exceeds the {} file size",
                offset,
                data.len(),
                file.length
            )));
        }

        fs_file.seek(SeekFrom::Start(offset as u64)).await?;
        fs_file.write(data).await?;
        fs_file.flush().await?;

        Ok(())
    }

    async fn read_piece(&self, file: &File, piece: &Piece) -> Result<Vec<u8>> {
        todo!()
    }

    async fn read_piece_part(&self, file: &File, part: &PiecePart) -> Result<Vec<u8>> {
        todo!()
    }

    async fn read_bytes(&self, file: &File, range: Range<usize>) -> Result<Vec<u8>> {
        let mut fs_file = self.internal_open(file, false).await?;
        let mut buffer = Vec::with_capacity(range.len());

        fs_file.seek(SeekFrom::Start(range.start as u64)).await?;
        fs_file.read_exact(&mut buffer).await?;

        Ok(buffer)
    }
}
