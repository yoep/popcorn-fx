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
    pub fn builder() -> TorrentInfoBuilder {
        TorrentInfoBuilder::builder()
    }

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

/// Builder for constructing `TorrentInfo` instances.
#[derive(Debug, Default)]
pub struct TorrentInfoBuilder {
    url: Option<String>,
    provider: Option<String>,
    source: Option<String>,
    title: Option<String>,
    quality: Option<String>,
    seed: Option<u32>,
    peer: Option<u32>,
    size: Option<String>,
    filesize: Option<String>,
    file: Option<String>,
}

impl TorrentInfoBuilder {
    /// Creates a new `TorrentInfoBuilder`.
    pub fn builder() -> Self {
        Self::default()
    }

    /// Sets the URL for the builder.
    pub fn url<T: ToString>(mut self, url: T) -> Self {
        self.url = Some(url.to_string());
        self
    }

    /// Sets the provider for the builder.
    pub fn provider<T: ToString>(mut self, provider: T) -> Self {
        self.provider = Some(provider.to_string());
        self
    }

    /// Sets the source for the builder.
    pub fn source<T: ToString>(mut self, source: T) -> Self {
        self.source = Some(source.to_string());
        self
    }

    /// Sets the title for the builder.
    pub fn title<T: ToString>(mut self, title: T) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Sets the quality for the builder.
    pub fn quality<T: ToString>(mut self, quality: T) -> Self {
        self.quality = Some(quality.to_string());
        self
    }

    /// Sets the seed for the builder.
    pub fn seed(mut self, seed: u32) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Sets the peer for the builder.
    pub fn peer(mut self, peer: u32) -> Self {
        self.peer = Some(peer);
        self
    }

    /// Sets the size for the builder.
    pub fn size<T: ToString>(mut self, size: T) -> Self {
        self.size = Some(size.to_string());
        self
    }

    /// Sets the filesize for the builder.
    pub fn filesize<T: ToString>(mut self, filesize: T) -> Self {
        self.filesize = Some(filesize.to_string());
        self
    }

    /// Sets the file for the builder.
    pub fn file<T: ToString>(mut self, file: T) -> Self {
        self.file = Some(file.to_string());
        self
    }

    /// Builds the `TorrentInfo` instance.
    ///
    /// # Panics
    ///
    /// This function will panic if any of the mandatory fields (`url`, `provider`, `source`, `title`, `quality`, `seed`, `peer`) are not set.
    pub fn build(self) -> TorrentInfo {
        TorrentInfo {
            url: self.url.expect("url is not set"),
            provider: self.provider.expect("provider is not set"),
            source: self.source.expect("source is not set"),
            title: self.title.expect("title is not set"),
            quality: self.quality.expect("quality is not set"),
            seed: self.seed.expect("seed is not set"),
            peer: self.peer.expect("peer is not set"),
            size: self.size,
            filesize: self.filesize,
            file: self.file,
        }
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
            100,                                // Seed count
            50,                                 // Peer count
            Some("100 MB".to_string()),         // Size (Optional)
            Some("500 MB".to_string()),         // Filesize (Optional)
            Some("sample.torrent".to_string()), // File (Optional)
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

    #[test]
    fn test_builder() {
        let expected_result = TorrentInfo {
            url: "MyUrl".to_string(),
            provider: "MyProvider".to_string(),
            source: "MySource".to_string(),
            title: "MyTitle".to_string(),
            quality: "480p".to_string(),
            seed: 18,
            peer: 5,
            size: None,
            filesize: None,
            file: None,
        };

        let result = TorrentInfo::builder()
            .url("MyUrl")
            .provider("MyProvider")
            .source("MySource")
            .title("MyTitle")
            .quality("480p")
            .seed(18)
            .peer(5)
            .build();

        assert_eq!(expected_result, result)
    }
}
