use derive_more::Display;
use serde::{Deserialize, Serialize};

/// The available images for a media item.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, Display)]
#[display(fmt = "poster: {}, fanart: {}, banner: {}", poster, fanart, banner)]
pub struct Images {
    pub poster: String,
    pub fanart: String,
    pub banner: String,
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
        Self::default()
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_none() {
        let result = Images::none();

        assert_eq!("".to_string(), result.poster);
        assert_eq!("".to_string(), result.banner);
        assert_eq!("".to_string(), result.fanart);
    }
}