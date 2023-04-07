use derive_more::Display;
use serde::{Deserialize, Serialize};

/// The available images for a media item, including the vertical poster image, fanart image, and banner image.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, Display)]
#[display(fmt = "poster: {}, fanart: {}, banner: {}", poster, fanart, banner)]
pub struct Images {
    /// The vertical rectangle poster image URL.
    pub poster: String,
    /// The background fanart image URL.
    pub fanart: String,
    /// The banner image URL.
    pub banner: String,
}

impl Images {
    /// Creates a new `Images` struct with the given image URLs.
    ///
    /// # Arguments
    ///
    /// * `poster` - The vertical poster image URL.
    /// * `fanart` - The fanart image URL.
    /// * `banner` - The banner image URL.
    pub fn new(poster: String, fanart: String, banner: String) -> Self {
        Self {
            poster,
            fanart,
            banner,
        }
    }

    /// Returns an empty `Images` struct with all image URLs set to empty strings.
    pub fn none() -> Self {
        Self::default()
    }

    /// Returns a reference to the vertical poster image URL.
    pub fn poster(&self) -> &str {
        self.poster.as_str()
    }

    /// Returns a reference to the fanart image URL.
    pub fn fanart(&self) -> &str {
        self.fanart.as_str()
    }

    /// Returns a reference to the banner image URL.
    pub fn banner(&self) -> &str {
        self.banner.as_str()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_images_new() {
        let poster = "https://example.com/poster.png".to_string();
        let fanart = "https://example.com/fanart.png".to_string();
        let banner = "https://example.com/banner.png".to_string();

        let images = Images::new(poster.clone(), fanart.clone(), banner.clone());

        assert_eq!(images.poster, poster);
        assert_eq!(images.fanart, fanart);
        assert_eq!(images.banner, banner);
    }

    #[test]
    fn test_none() {
        let result = Images::none();

        assert_eq!("".to_string(), result.poster);
        assert_eq!("".to_string(), result.banner);
        assert_eq!("".to_string(), result.fanart);
    }
}