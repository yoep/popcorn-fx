use std::cmp::Ordering;
use std::fmt::{Debug, Display};
#[cfg(test)]
use std::fmt::Formatter;

use derive_more::Display;
use downcast_rs::{DowncastSync, impl_downcast};
#[cfg(test)]
use mockall::automock;

use crate::core::media::Rating;

/// The media type identifier.
#[derive(Debug, Copy, Clone, PartialOrd, Eq, Display, PartialEq)]
pub enum MediaType {
    Unknown = -1,
    Movie = 0,
    Show = 1,
    Episode = 2,
}

impl Ord for MediaType {
    fn cmp(&self, other: &Self) -> Ordering {
        return if self == other {
            Ordering::Equal
        } else if self == &MediaType::Movie && other != &MediaType::Movie {
            Ordering::Less
        } else {
            Ordering::Greater
        };
    }
}

/// Basic identification information about a media item.
#[cfg_attr(test, automock)]
pub trait MediaIdentifier: Debug + DowncastSync + Display {
    /// Retrieve an owned instance of the IMDB id.
    fn imdb_id(&self) -> String;

    /// Get the type of the media.
    fn media_type(&self) -> MediaType;

    /// The title of the media item.
    /// The title should always be html decoded.
    fn title(&self) -> String;
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
pub trait MediaOverview: MediaIdentifier {
    /// Retrieve the rating of the media item if available.
    fn rating(&self) -> Option<&Rating>;

    /// Retrieve the release year of the media item.
    fn year(&self) -> &String;
}

/// The detailed information of a media item containing all information need.
pub trait MediaDetails: MediaOverview {
    /// Retrieve the description of the media item.
    /// The description should always be html decoded.
    fn synopsis(&self) -> String;

    /// Retrieve the runtime of the media item.
    fn runtime(&self) -> i32;
}
