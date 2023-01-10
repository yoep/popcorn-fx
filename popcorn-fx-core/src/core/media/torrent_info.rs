use serde::Deserialize;

/// The available torrent information of a media item.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct TorrentInfo {
    url: String,
    provider: String,
    source: String,
    title: String,
    quality: String,
    seed: u32,
    peer: u32,
    size: String,
    filesize: String,
}

impl TorrentInfo {
    pub fn new(url: String, title: String, quality: String) -> Self {
        Self {
            url,
            provider: String::new(),
            source: String::new(),
            title,
            quality,
            seed: 0,
            peer: 0,
            size: String::new(),
            filesize: String::new(),
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

    pub fn size(&self) -> &String {
        &self.size
    }

    pub fn filesize(&self) -> &String {
        &self.filesize
    }
}