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
    ///
    /// # Returns
    ///
    /// A new `Images` struct with the provided image URLs.
    pub fn new(poster: String, fanart: String, banner: String) -> Self {
        Self {
            poster,
            fanart,
            banner,
        }
    }

    /// Returns a builder to construct an `Images` struct.
    ///
    /// # Returns
    ///
    /// An `ImagesBuilder` instance.
    pub fn builder() -> ImagesBuilder {
        ImagesBuilder::new()
    }

    /// Returns an empty `Images` struct with all image URLs set to empty strings.
    ///
    /// # Returns
    ///
    /// An empty `Images` struct.
    pub fn none() -> Self {
        Self::default()
    }

    /// Returns a reference to the vertical poster image URL.
    ///
    /// # Returns
    ///
    /// A reference to the vertical poster image URL.
    pub fn poster(&self) -> &str {
        self.poster.as_str()
    }

    /// Returns a reference to the fanart image URL.
    ///
    /// # Returns
    ///
    /// A reference to the fanart image URL.
    pub fn fanart(&self) -> &str {
        self.fanart.as_str()
    }

    /// Returns a reference to the banner image URL.
    ///
    /// # Returns
    ///
    /// A reference to the banner image URL.
    pub fn banner(&self) -> &str {
        self.banner.as_str()
    }
}

/// A builder for creating an `Images` struct.
#[derive(Debug, Default)]
pub struct ImagesBuilder {
    poster: Option<String>,
    fanart: Option<String>,
    banner: Option<String>,
}

impl ImagesBuilder {
    /// Creates a new `ImagesBuilder` instance.
    ///
    /// # Returns
    ///
    /// A new `ImagesBuilder` instance with default values.
    pub fn new() -> Self {
        ImagesBuilder {
            poster: None,
            fanart: None,
            banner: None,
        }
    }

    /// Sets the poster image.
    ///
    /// # Arguments
    ///
    /// * `poster` - The URL or path to the poster image.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `ImagesBuilder` instance.
    pub fn poster<S>(&mut self, poster: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.poster = Some(poster.into());
        self
    }

    /// Sets the fanart image.
    ///
    /// # Arguments
    ///
    /// * `fanart` - The URL or path to the fanart image.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `ImagesBuilder` instance.
    pub fn fanart<S>(&mut self, fanart: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.fanart = Some(fanart.into());
        self
    }

    /// Sets the banner image.
    ///
    /// # Arguments
    ///
    /// * `banner` - The URL or path to the banner image.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `ImagesBuilder` instance.
    pub fn banner<S>(&mut self, banner: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.banner = Some(banner.into());
        self
    }

    /// Builds an `Images` struct using the configured values.
    ///
    /// # Returns
    ///
    /// An `Images` struct with the configured values.
    pub fn build(&self) -> Images {
        Images {
            poster: self.poster.clone().unwrap_or_else(|| String::new()),
            fanart: self.fanart.clone().unwrap_or_else(|| String::new()),
            banner: self.banner.clone().unwrap_or_else(|| String::new()),
        }
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

    #[test]
    fn test_images_builder() {
        let fanart = "MyFanartUrl";
        let poster = "MyPosterUrl";
        let banner = "MyBannerUrl";
        let expected_result = Images {
            poster: poster.to_string(),
            fanart: fanart.to_string(),
            banner: banner.to_string(),
        };

        let result = Images::builder()
            .fanart(fanart)
            .poster(poster)
            .banner(banner)
            .build();

        assert_eq!(expected_result, result);
    }
}
