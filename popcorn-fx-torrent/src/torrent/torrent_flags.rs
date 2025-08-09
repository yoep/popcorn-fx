use bitmask_enum::bitmask;

/// Flags that define the behavior of a [crate::torrent::Torrent].
///
/// These flags can be combined to customize the torrent's behavior.
/// By default, a torrent is assigned the [TorrentFlags::AutoManaged] flag,
/// which enables automatic metadata retrieval and starts the download automatically.
#[bitmask(u16)]
#[bitmask_config(vec_debug, flags_iter)]
pub enum TorrentFlags {
    /// Indicates seed mode where only data is uploaded.
    SeedMode = 0b0000000000000001,
    /// Indicates if uploading data is allowed.
    UploadMode = 0b0000000000000010,
    /// Indicates if downloading data is allowed.
    DownloadMode = 0b0000000000000100,
    /// Indicates share mode.
    ShareMode = 0b0000000000001000,
    /// Applies an IP filter.
    ApplyIpFilter = 0b0000000000010000,
    /// Torrent is paused.
    Paused = 0b0000000000100000,
    /// Complete the torrent metadata from peers if needed.
    Metadata = 0b0000000001000000,
    /// Sequential download is enabled.
    SequentialDownload = 0b0000000010000100,
    /// Torrent should stop when ready.
    StopWhenReady = 0b0000000100000000,
    /// Torrent is auto-managed.
    /// This means that the torrent may be resumed at any point in time.
    AutoManaged = 0b0000001001000110,
}

impl Default for TorrentFlags {
    fn default() -> Self {
        TorrentFlags::AutoManaged
    }
}
