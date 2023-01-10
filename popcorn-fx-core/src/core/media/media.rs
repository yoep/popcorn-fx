use std::fmt::Debug;

use downcast_rs::{Downcast, impl_downcast};

/// The media type identifier.
#[derive(Debug, Clone, PartialEq)]
pub enum MediaType {
    Unknown = -1,
    Movie = 0,
    Show = 1,
    Episode = 2,
}

/// Basic identification information about a media item.
pub trait MediaIdentifier: Debug + Downcast {
    /// Get the unique ID of the media.
    fn id(&self) -> &String;

    /// Get the type of the media.
    fn media_type(&self) -> MediaType;

    /// The title of the media item.
    /// The title should always be html decoded.
    fn title(&self) -> String;
}
impl_downcast!(MediaIdentifier);

/// Defines an object that can be watched.
pub trait Watchable: MediaIdentifier {
    /// Verify if the current object is watched.
    fn is_watched(&self) -> bool;
}

/// Defines an object that can be liked.
pub trait Favorable: MediaIdentifier {
    /// Verify if the object is liked.
    fn is_liked(&self) -> bool;
}

/// The most basic information of a media item.
/// It can be used for an overview information but won't contain any details.
pub trait MediaOverview: MediaIdentifier + Watchable + Favorable {}

/// The detailed information of a media item containing all information need.
pub trait MediaDetails: MediaOverview {
    /// Retrieve the description of the media item.
    /// The description should always be html decoded.
    fn synopsis(&self) -> String;
}

