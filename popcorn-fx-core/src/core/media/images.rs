use serde::Deserialize;

/// The available images for a media item.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Images {
    poster: String,
    fanart: String,
    banner: String,
}

impl Images {
    pub fn new(poster: String, fanart: String, banner: String) -> Self {
        Self {
            poster,
            fanart,
            banner,
        }
    }

    /// Retrieve an empty [Images] struct which contains all empty strings.
    pub fn none() -> Self {
        Self {
            poster: String::new(),
            fanart: String::new(),
            banner: String::new(),
        }
    }

    pub fn poster(&self) -> &String {
        &self.poster
    }

    pub fn fanart(&self) -> &String {
        &self.fanart
    }

    pub fn banner(&self) -> &String {
        &self.banner
    }
}