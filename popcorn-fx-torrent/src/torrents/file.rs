use crate::torrents::{Piece, PiecePriority, TorrentFileInfo};
use std::hash::Hash;
use std::ops::Range;
use std::path::PathBuf;

/// The unique index of the file within the torrent.
pub type FileIndex = usize;

/// Alias name for the piece priority of a file.
pub type FilePriority = PiecePriority;

/// The file information of a torrent.
/// Torrents can contain one or more files.
#[derive(Debug, Clone)]
pub struct File {
    /// The index of the file within the torrent.
    pub index: FileIndex,
    /// The path of the file within the torrent.
    pub path: PathBuf,
    /// The offset of the file within the torrent.
    pub offset: usize,
    /// The total size of the file within the torrent.
    pub length: usize,
    /// The original metadata info of the file from the torrent.
    pub info: TorrentFileInfo,
    /// The priority of the file.
    pub priority: FilePriority,
}

impl File {
    /// Check if the file contains some bytes from the given piece.
    /// It returns true when at least 1 byte overlaps with the given piece, else false.
    pub fn contains(&self, piece: &Piece) -> bool {
        let file_range = self.torrent_byte_range();
        let piece_range = piece.torrent_byte_range();

        overlapping_range(file_range, piece_range).is_some()
    }

    /// Get the overlapping byte range of the file with the given piece, still relative to the torrent bytes.
    /// It returns the writing offset to start from within the file and the applicable byte range to this file.
    ///
    /// # Returns
    ///
    /// Returns the applicable byte range with offset for this file, or [None] if there is no overlap.
    pub fn torrent_piece_byte_range(&self, piece: &Piece) -> Option<(usize, Range<usize>)> {
        let file_offset = piece.offset.checked_sub(self.offset).unwrap_or(0);

        overlapping_range(self.torrent_byte_range(), piece.torrent_byte_range())
            .map(|range| (file_offset, range))
    }

    /// Get the overlapping byte range of the file with the given piece.
    /// The returned range is relative to a data bytes slice of the piece.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io::SeekFrom;
    /// use tokio::io::{AsyncSeekExt, AsyncWriteExt};
    /// use popcorn_fx_torrent::torrents::{File, Piece};
    ///
    /// async fn write_file(fs_file: &mut tokio::fs::File, file: File, piece: Piece, data: Vec<u8>) {
    ///     let (offset, range) = file.data_byte_range(&piece).unwrap();
    ///
    ///     fs_file.seek(SeekFrom::Start(offset as u64)).await;
    ///     fs_file.write(&data[range]).await;
    /// }
    /// ```
    ///
    /// # Returns
    ///
    /// Returns the applicable byte range with offset for this file, or [None] if there is no overlap.
    pub fn data_byte_range(&self, piece: &Piece) -> Option<(usize, Range<usize>)> {
        self.torrent_piece_byte_range(piece)
            .map(|(offset, torrent_byte_range)| {
                let file_data_offset = offset.saturating_sub(self.offset);
                let data_byte_start = torrent_byte_range.start.saturating_sub(piece.offset);
                let data_byte_end = torrent_byte_range.end.saturating_sub(piece.offset);

                (file_data_offset, data_byte_start..data_byte_end)
            })
    }

    /// Get the range of the file in bytes relative to the torrent.
    /// It returns the byte range of the file within the torrent.
    pub fn torrent_byte_range(&self) -> Range<usize> {
        self.offset..(self.offset + self.length)
    }
}

impl PartialEq<Self> for File {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.offset == other.offset && self.length == other.length
    }
}

impl Eq for File {}

impl Hash for File {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

/// Get the overlapping range of two ranges.
/// It returns the overlapping range if there is one, else [None].
pub fn overlapping_range<T>(r1: Range<T>, r2: Range<T>) -> Option<Range<T>>
where
    T: Ord + Copy,
{
    let start = r1.start.max(r2.start);
    let end = r1.end.min(r2.end);

    if start < end {
        Some(start..end)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torrents::InfoHash;

    #[test]
    fn test_contains() {
        let piece = Piece::new(InfoHash::default(), 0, 0, 64);
        let file = new_file(0, 128);
        let result = file.contains(&piece);
        assert_eq!(true, result);

        let piece = Piece::new(InfoHash::default(), 0, 0, 128);
        let file = new_file(100, 64);
        let result = file.contains(&piece);
        assert_eq!(true, result);

        let piece = Piece::new(InfoHash::default(), 0, 1024, 128);
        let file = new_file(512, 128);
        let result = file.contains(&piece);
        assert_eq!(false, result);
    }

    #[test]
    fn test_torrent_byte_range() {
        let file = new_file(64, 128);

        let result = file.torrent_byte_range();

        assert_eq!(64..192, result);
    }

    #[test]
    fn test_torrent_piece_byte_range() {
        let piece = Piece::new(InfoHash::default(), 0, 0, 64);
        let file = new_file(0, 128);
        let result = file.torrent_piece_byte_range(&piece);
        assert_eq!(Some((0, 0..64)), result);

        let piece = Piece::new(InfoHash::default(), 0, 64, 512);
        let file = new_file(0, 128);
        let result = file.torrent_piece_byte_range(&piece);
        assert_eq!(Some((64, 64..128)), result);

        let piece = Piece::new(InfoHash::default(), 0, 64, 448);
        let file = new_file(496, 128);
        let result = file.torrent_piece_byte_range(&piece);
        assert_eq!(Some((0, 496..512)), result);

        let piece = Piece::new(InfoHash::default(), 0, 128, 128);
        let file = new_file(64, 128);
        let result = file.torrent_piece_byte_range(&piece);
        assert_eq!(Some((64, 128..192)), result);
    }

    #[test]
    fn test_data_bytes_range() {
        let piece = Piece::new(InfoHash::default(), 0, 128, 128);
        let file = new_file(64, 128);
        let result = file.data_byte_range(&piece);
        assert_eq!(Some((0, 0..64)), result);

        let piece = Piece::new(InfoHash::default(), 0, 128, 512);
        let file = new_file(0, 256);
        let result = file.data_byte_range(&piece);
        assert_eq!(Some((128, 0..128)), result);
    }

    #[test]
    fn test_overlap_range() {
        let r1 = 0..10;
        let r2 = 5..15;
        let result = overlapping_range(r1, r2);
        assert_eq!(Some(5..10), result);

        let r1 = 16..32;
        let r2 = 30..64;
        let result = overlapping_range(r1, r2);
        assert_eq!(Some(30..32), result);

        let r1 = 128..256;
        let r2 = 512..1024;
        let result = overlapping_range(r1, r2);
        assert_eq!(None, result);
    }

    fn new_file(offset: usize, length: usize) -> File {
        File {
            index: 0,
            path: PathBuf::new(),
            offset,
            length,
            info: TorrentFileInfo {
                length: 0,
                path: None,
                path_utf8: None,
                md5sum: None,
                attr: None,
                symlink_path: None,
                sha1: None,
            },
            priority: FilePriority::default(),
        }
    }
}
