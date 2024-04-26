use std::cmp::Ordering;
use std::fmt::{Debug, Display};
#[cfg(test)]
use std::fmt::Formatter;

use derive_more::Display;
use downcast_rs::{Downcast, DowncastSync, impl_downcast};
use log::{error, warn};
#[cfg(test)]
use mockall::automock;

use crate::core::media::{
    Category, Episode, Images, MovieDetails, MovieOverview, Rating, ShowDetails, ShowOverview,
};

/// The media type identifier.
#[derive(Debug, Copy, Clone, Eq, Display, PartialEq)]
pub enum MediaType {
    Unknown = -1,
    Movie = 0,
    Show = 1,
    Episode = 2,
}

impl PartialOrd for MediaType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self == &MediaType::Movie && other != &MediaType::Movie {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Greater)
        }
    }
}

impl Ord for MediaType {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).expect("expected an ordering")
    }
}

impl From<MediaType> for Category {
    fn from(value: MediaType) -> Self {
        match value {
            MediaType::Unknown => Category::Movies,
            MediaType::Movie => Category::Movies,
            MediaType::Show => Category::Series,
            MediaType::Episode => Category::Series,
        }
    }
}

/// Basic identification information about a media item.
#[cfg_attr(test, automock)]
pub trait MediaIdentifier: Debug + DowncastSync + Display {
    /// Retrieve an owned instance of the IMDB id.
    fn imdb_id(&self) -> &str;

    /// Get the type of the media.
    fn media_type(&self) -> MediaType;

    /// The title of the media item.
    /// The title should always be html decoded.
    fn title(&self) -> String;

    /// Clone the `MediaIdentifier` trait object.
    ///
    /// This function attempts to clone the `MediaIdentifier` trait object into a new `Box<dyn MediaIdentifier>`.
    /// If the type can be downcast to a known concrete type (e.g., `Episode`, `ShowOverview`, `MovieOverview`, etc.),
    /// it will create a cloned instance of that type and return it as `Some(Box<dyn MediaIdentifier>)`. If the type
    /// cannot be downcast or is unknown, it will log an error and return `None`.
    fn clone_identifier(&self) -> Option<Box<dyn MediaIdentifier>> {
        if let Some(e) = self.as_any().downcast_ref::<Episode>() {
            Some(Box::new(e.clone()) as Box<dyn MediaIdentifier>)
        } else if let Some(e) = self.as_any().downcast_ref::<ShowOverview>() {
            Some(Box::new(e.clone()) as Box<dyn MediaIdentifier>)
        } else if let Some(e) = self.as_any().downcast_ref::<MovieOverview>() {
            Some(Box::new(e.clone()) as Box<dyn MediaIdentifier>)
        } else if let Some(e) = self.as_any().downcast_ref::<MovieDetails>() {
            Some(Box::new(e.clone()) as Box<dyn MediaIdentifier>)
        } else if let Some(e) = self.as_any().downcast_ref::<ShowDetails>() {
            Some(Box::new(e.clone()) as Box<dyn MediaIdentifier>)
        } else {
            error!(
                "Unable to clone MediaIdentifier, unknown type {:?}",
                self.type_id()
            );
            None
        }
    }

    /// Converts the `MediaIdentifier` trait object to a `MediaOverview` trait object.
    ///
    /// This function attempts to downcast the `MediaIdentifier` trait object into a `MediaOverview` trait object.
    /// If the type can be downcast to a known concrete type (e.g., `ShowOverview`, `MovieOverview`, etc.),
    /// it will create a cloned instance of that type and return it as `Some(Box<dyn MediaOverview>)`. If the type
    /// cannot be downcast or is unknown, it will log a warning and return `None`.
    fn into_overview(&self) -> Option<Box<dyn MediaOverview>> {
        if let Some(e) = self.as_any().downcast_ref::<ShowOverview>() {
            Some(Box::new(e.clone()) as Box<dyn MediaOverview>)
        } else if let Some(e) = self.as_any().downcast_ref::<MovieOverview>() {
            Some(Box::new(e.clone()) as Box<dyn MediaOverview>)
        } else if let Some(e) = self.as_any().downcast_ref::<MovieDetails>() {
            Some(Box::new(e.clone()) as Box<dyn MediaOverview>)
        } else if let Some(e) = self.as_any().downcast_ref::<ShowDetails>() {
            Some(Box::new(e.clone()) as Box<dyn MediaOverview>)
        } else {
            warn!(
                "Unable to downcast MediaIdentifier to MediaOverview, unsupported type {:?}",
                self.type_id()
            );
            None
        }
    }
}
impl_downcast!(sync MediaIdentifier);

#[cfg(test)]
impl Display for MockMediaIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockMediaIdentifier")
    }
}

/// The most basic information of a media item.
/// It can be used for an overview information but won't contain any details.
pub trait MediaOverview: MediaIdentifier + Downcast {
    /// Retrieve the rating of the media item if available.
    fn rating(&self) -> Option<&Rating>;

    /// Retrieve the release year of the media item.
    fn year(&self) -> &String;

    /// Retrieve the images of the media item.
    fn images(&self) -> &Images;
}
impl_downcast!(sync MediaOverview);

/// The detailed information of a media item containing all information need.
pub trait MediaDetails: MediaOverview {
    /// Retrieve the description of the media item.
    /// The description should always be html decoded.
    fn synopsis(&self) -> String;

    /// Retrieve the runtime of the media item.
    fn runtime(&self) -> i32;
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;

    use super::*;

    #[test]
    fn test_media_type_ordering() {
        let equal = MediaType::Show.cmp(&MediaType::Show);
        let less = MediaType::Movie.cmp(&MediaType::Show);
        let greater = MediaType::Show.cmp(&MediaType::Movie);

        assert_eq!(Ordering::Equal, equal);
        assert_eq!(Ordering::Less, less);
        assert_eq!(Ordering::Greater, greater);
    }

    #[test]
    fn test_from_media_type() {
        assert_eq!(Category::Movies, Category::from(MediaType::Movie));
        assert_eq!(Category::Series, Category::from(MediaType::Show));
        assert_eq!(Category::Series, Category::from(MediaType::Episode));
    }

    #[test]
    fn test_clone_identifier() {
        let imdb_id = "tt123456";
        let media = MovieOverview {
            title: "Foo bar".to_string(),
            imdb_id: imdb_id.to_string(),
            year: "2012".to_string(),
            rating: None,
            images: Default::default(),
        };

        let result = media.clone_identifier();

        assert!(
            result.is_some(),
            "expected the media identifier to have been cloned"
        );
        assert_eq!(imdb_id, result.unwrap().imdb_id());
    }
}
