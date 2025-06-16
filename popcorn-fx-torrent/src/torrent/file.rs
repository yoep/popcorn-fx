use crate::torrent::{overlapping_range, FileAttributeFlags, PiecePriority, TorrentFileInfo};
use log::warn;
use std::hash::Hash;
use std::ops::Range;
use std::path::PathBuf;

/// The unique index of the file within the torrent.
pub type FileIndex = usize;

/// Alias name for the piece priority of a file.
pub type FilePriority = PiecePriority;

/// The information of one of the file(s) from a torrent.
///
/// ## Ranges
///
/// A torrent file has 2 distinct byte ranges,
/// one that represents the range of bytes within the torrent and one that represents the range of bytes within the io storage device.
///
/// Use [File::io_range] to get the byte range of the file on the io storage device.
///
/// USe [File::torrent_range] to get the byte range of the file within the torrent.
#[derive(Debug, Clone)]
pub struct File {
    /// The index of the file within the torrent.
    pub index: FileIndex,
    /// The path of the file within the torrent.
    pub torrent_path: PathBuf,
    /// The absolute filepath of the file on the storage device.
    pub io_path: PathBuf,
    /// The offset of the file within the torrent.
    pub offset: usize,
    /// The original metadata info of the file from the torrent.
    pub info: TorrentFileInfo,
    /// The priority of the file.
    pub priority: FilePriority,
}

impl File {
    /// Get the filename of the file.
    pub fn filename(&self) -> String {
        Self::filename_from_path(&self.torrent_path)
            .unwrap_or_else(|| Self::filename_from_path(&self.info.path()).unwrap_or(String::new()))
    }

    /// Get the total bytes of the torrent file.
    pub fn length(&self) -> usize {
        self.info.length as usize
    }

    /// Check if the file contains some bytes from the given torrent byte range.
    ///
    /// # Returns
    ///
    /// It returns `true` when at least 1 byte overlaps with the given range, else `false`.
    pub fn contains(&self, range: &Range<usize>) -> bool {
        let file_range = self.torrent_range();
        overlapping_range(file_range, range).is_some()
    }

    /// Get the byte range of the file within the io storage device.
    ///
    /// # Returns
    ///
    /// It returns a `Range<usize>` representing the start and end byte positions of the file
    /// within the storage device.
    pub fn io_range(&self) -> Range<usize> {
        0..self.length()
    }

    /// Get the byte range of the file within the torrent.
    ///
    /// # Returns
    ///
    /// It returns a `Range<usize>` indicating the file's position in bytes within the torrent,
    /// starting from its offset and extending to its length.
    pub fn torrent_range(&self) -> Range<usize> {
        self.offset..(self.offset + self.length())
    }

    /// Get the portion of the given torrent byte range that corresponds to the file's storage range.
    ///
    /// # Arguments
    ///
    /// * `torrent_bytes`: The byte range within the torrent.
    ///
    /// # Returns
    ///
    /// It returns an `Option<Range<usize>>` containing the portion of the given range that overlaps with the file's
    /// storage range. If there is no overlap, `None` is returned.
    pub fn io_applicable_byte_range(&self, torrent_bytes: &Range<usize>) -> Option<Range<usize>> {
        overlapping_range(self.torrent_range(), &torrent_bytes).map(|e| {
            let start_offset = e.start.saturating_sub(self.offset);
            start_offset..e.end - self.offset
        })
    }

    /// Get the portion of the given torrent byte range that overlaps with the file's range in the torrent.
    ///
    /// # Arguments
    ///
    /// * `torrent_bytes`: The byte range within the torrent.
    ///
    /// # Returns
    ///
    /// It returns an `Option<Range<usize>>` containing the portion of `torrent_bytes` that overlaps with the file's
    /// range in the torrent. If there is no overlap, `None` is returned.
    pub fn torrent_applicable_byte_range(
        &self,
        torrent_bytes: &Range<usize>,
    ) -> Option<Range<usize>> {
        overlapping_range(self.torrent_range(), torrent_bytes)
    }

    /// Get the byte slice of the given torrent bytes that are applicable to this file.
    ///
    /// # Arguments
    ///
    /// * `torrent_bytes`: The range of the bytes within the torrent.
    /// * `bytes`: The data bytes of the torrent corresponding to the given `torrent_bytes`.
    ///
    /// # Returns
    ///
    /// It returns the slice of bytes that are relevant to this file.
    /// If there is no overlap, `None` is returned.
    pub fn torrent_applicable_bytes<'a>(
        &self,
        torrent_bytes: &Range<usize>,
        bytes: &'a [u8],
    ) -> Option<&'a [u8]> {
        if torrent_bytes.len() != bytes.len() {
            warn!("Torrent file \"{:?}\" is unable to calculate applicable bytes, invalid range provided", self.torrent_path);
            return None;
        }

        if let Some(overlapping_range) = self.torrent_applicable_byte_range(torrent_bytes) {
            let start = overlapping_range.start.saturating_sub(torrent_bytes.start);
            let end = overlapping_range.end.saturating_sub(torrent_bytes.start);
            return Some(&bytes[start..end]);
        }

        None
    }

    /// Get the file attributes of the torrent file.
    pub fn attributes(&self) -> FileAttributeFlags {
        self.info.attr.unwrap_or(FileAttributeFlags::default())
    }

    fn filename_from_path(path: &PathBuf) -> Option<String> {
        if let Some(filename) = path.file_name() {
            filename.to_str().map(|e| e.to_string())
        } else {
            None
        }
    }
}

impl PartialEq<Self> for File {
    fn eq(&self, other: &Self) -> bool {
        self.torrent_path == other.torrent_path
            && self.offset == other.offset
            && self.length() == other.length()
    }
}

impl Eq for File {}

impl Hash for File {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.torrent_path.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torrent::{InfoHash, Piece};

    #[test]
    fn test_contains() {
        let piece = Piece::new(InfoHash::default(), 0, 0, 64);
        let file = new_file(0, 128);
        let result = file.contains(&piece.torrent_range());
        assert_eq!(true, result);

        let piece = Piece::new(InfoHash::default(), 0, 0, 128);
        let file = new_file(100, 64);
        let result = file.contains(&piece.torrent_range());
        assert_eq!(true, result);

        let piece = Piece::new(InfoHash::default(), 0, 1024, 128);
        let file = new_file(512, 128);
        let result = file.contains(&piece.torrent_range());
        assert_eq!(false, result);
    }

    #[test]
    fn test_file_io_range() {
        let file = new_file(64, 128);
        let expected_result = 0..128;

        let result = file.io_range();

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_file_torrent_range() {
        let file = new_file(64, 128);
        let expected_result = 64..192;

        let result = file.torrent_range();

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_file_io_applicable_byte_range() {
        let file = new_file(0, 128);
        let result = file.io_applicable_byte_range(&(0..64));
        assert_eq!(Some(0..64), result);

        let file = new_file(0, 128);
        let result = file.io_applicable_byte_range(&(64..512));
        assert_eq!(Some(64..128), result);

        let file = new_file(496, 128);
        let result = file.io_applicable_byte_range(&(64..512));
        assert_eq!(Some(0..16), result);

        let file = new_file(64, 128);
        let result = file.io_applicable_byte_range(&(128..512));
        assert_eq!(Some(64..128), result);

        let file = new_file(128, 64);
        let result = file.io_applicable_byte_range(&(0..192));
        assert_eq!(Some(0..64), result);
    }

    #[test]
    fn test_file_torrent_applicable_byte_range() {
        let file = new_file(0, 128);
        let result = file.torrent_applicable_byte_range(&(0..64));
        assert_eq!(Some(0..64), result);

        let file = new_file(0, 128);
        let result = file.torrent_applicable_byte_range(&(64..512));
        assert_eq!(Some(64..128), result);

        let file = new_file(496, 128);
        let result = file.torrent_applicable_byte_range(&(64..512));
        assert_eq!(Some(496..512), result);

        let file = new_file(64, 64);
        let result = file.torrent_applicable_byte_range(&(64..512));
        assert_eq!(Some(64..128), result);

        let file = new_file(1028, 512);
        let result = file.torrent_applicable_byte_range(&(0..1028));
        assert_eq!(None, result);
    }

    #[test]
    fn test_file_torrent_applicable_bytes() {
        let file = new_file(0, 128);
        let data = (0..64).map(|i| i as u8).collect::<Vec<u8>>();
        let result = file.torrent_applicable_bytes(&(0..64), data.as_slice());
        assert_eq!(Some(data.as_slice()), result);

        let file = new_file(0, 128);
        let data = (0..448).map(|i| i as u8).collect::<Vec<u8>>();
        let result = file.torrent_applicable_bytes(&(64..512), data.as_slice());
        assert_eq!(Some(&data[0..64]), result);
    }

    fn new_file(offset: usize, length: usize) -> File {
        File {
            index: 0,
            torrent_path: Default::default(),
            io_path: Default::default(),
            offset,
            info: TorrentFileInfo {
                length: length as u64,
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
