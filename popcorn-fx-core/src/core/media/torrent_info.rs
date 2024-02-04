use serde::{Deserialize, Serialize};

/// Represents the available torrent information for a media item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TorrentInfo {
    /// The URL of the torrent.
    url: String,
    /// The provider of the torrent.
    provider: String,
    /// The source of the torrent.
    source: String,
    /// The title associated with the torrent.
    title: String,
    /// The quality of the torrent.
    quality: String,
    /// The number of seeds for the torrent.
    #[serde(alias = "seeds")]
    seed: u32,
    /// The number of peers for the torrent.
    #[serde(alias = "peers")]
    peer: u32,
    /// The size of the torrent in bytes, if known.
    /// This is typically available for movies and may be `None` for episodes.
    size: Option<String>,
    /// The filesize of the torrent in human-readable format, if known.
    /// This is typically available for movies and may be `None` for episodes.
    filesize: Option<String>,
    /// The file to use from within a torrent collection, if present.
    /// This field is available when the torrent is a collection; otherwise, the primary media file
    /// from the torrent info should be used.
    file: Option<String>,
}

impl TorrentInfo {
    /// Creates a new `TorrentInfo` instance with the specified details.
    pub fn new(
        url: String,
        provider: String,
        source: String,
        title: String,
        quality: String,
        seed: u32,
        peer: u32,
        size: Option<String>,
        filesize: Option<String>,
        file: Option<String>,
    ) -> Self {
        Self {
            url,
            provider,
            source,
            title,
            quality,
            seed,
            peer,
            size,
            filesize,
            file,
        }
    }

    /// Gets the URL of the torrent.
    pub fn url(&self) -> &str {
        self.url.as_str()
    }

    /// Gets the provider of the torrent.
    pub fn provider(&self) -> &String {
        &self.provider
    }

    /// Gets the source of the torrent.
    pub fn source(&self) -> &String {
        &self.source
    }

    /// Gets the title associated with the torrent.
    pub fn title(&self) -> &String {
        &self.title
    }

    /// Gets the quality of the torrent.
    pub fn quality(&self) -> &String {
        &self.quality
    }

    /// Gets the number of seeds for the torrent.
    pub fn seed(&self) -> &u32 {
        &self.seed
    }

    /// Gets the number of peers for the torrent.
    pub fn peer(&self) -> &u32 {
        &self.peer
    }

    /// Retrieves the size of the torrent in bytes, if known.
    /// This is typically available for movies and may be `None` for episodes.
    pub fn size(&self) -> Option<&String> {
        self.size.as_ref()
    }

    /// Retrieves the filesize of the torrent in human-readable format, if known.
    /// This is typically available for movies and may be `None` for episodes.
    pub fn filesize(&self) -> Option<&String> {
        self.filesize.as_ref()
    }

    /// Retrieves the file to use from within a torrent collection, if present.
    /// This field is available when the torrent is a collection; otherwise, the primary media file
    /// from the torrent info should be used.
    pub fn file(&self) -> Option<&String> {
        self.file.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let torrent_info = TorrentInfo::new(
            "https://example.com/torrent".to_string(),
            "Provider Name".to_string(),
            "Source Name".to_string(),
            "Torrent Title".to_string(),
            "High Quality".to_string(),
            100,  // Seed count
            50,   // Peer count
            Some("100 MB".to_string()),  // Size (Optional)
            Some("500 MB".to_string()),  // Filesize (Optional)
            Some("sample.torrent".to_string()),  // File (Optional)
        );

        assert_eq!(torrent_info.url, "https://example.com/torrent");
        assert_eq!(torrent_info.provider, "Provider Name");
        assert_eq!(torrent_info.source, "Source Name");
        assert_eq!(torrent_info.title, "Torrent Title");
        assert_eq!(torrent_info.quality, "High Quality");
        assert_eq!(torrent_info.seed, 100);
        assert_eq!(torrent_info.peer, 50);
        assert_eq!(torrent_info.size, Some("100 MB".to_string()));
        assert_eq!(torrent_info.filesize, Some("500 MB".to_string()));
        assert_eq!(torrent_info.file, Some("sample.torrent".to_string()));
    }
}