use serde::{Deserialize, Serialize};

/// The available torrent information of a media item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TorrentInfo {
    url: String,
    provider: String,
    source: String,
    title: String,
    quality: String,
    #[serde(alias = "seeds")]
    seed: u32,
    #[serde(alias = "peers")]
    peer: u32,
    size: Option<String>,
    filesize: Option<String>,
    file: Option<String>,
}

impl TorrentInfo {
    pub fn new(url: String, title: String, quality: String, file: Option<String>) -> Self {
        Self {
            url,
            provider: String::new(),
            source: String::new(),
            title,
            quality,
            seed: 0,
            peer: 0,
            size: None,
            filesize: None,
            file
        }
    }

    pub fn url(&self) -> &String {
        &self.url
    }

    pub fn provider(&self) -> &String {
        &self.provider
    }

    pub fn source(&self) -> &String {
        &self.source
    }

    pub fn title(&self) -> &String {
        &self.title
    }

    pub fn quality(&self) -> &String {
        &self.quality
    }

    pub fn seed(&self) -> &u32 {
        &self.seed
    }

    pub fn peer(&self) -> &u32 {
        &self.peer
    }

    /// Retrieve the size in bytes of the torrent if known.
    /// This is most of the time only known for movies, episodes will return [None] most of the time.
    pub fn size(&self) -> Option<&String> {
        match &self.size {
            None => None,
            Some(e) => Some(e)
        }
    }

    /// Retrieve the filesize in human readable format of the torrent if known.
    /// This is most of the time only known for movies, episodes will return [None] most of the time.
    pub fn filesize(&self) -> Option<&String> {
        match &self.filesize {
            None => None,
            Some(e) => Some(e)
        }
    }

    /// Retrieve the file to use from within a torrent collection.
    /// This field is present when the torrent is a collection, otherwise, the only available media file
    /// should be used from the torrent info.
    pub fn file(&self) -> Option<&String> {
        match &self.file {
            None => None,
            Some(e) => Some(e)
        }
    }
}