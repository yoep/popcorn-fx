use std::fmt::Debug;

use derive_more::Display;
use serde::Deserialize;

/// The media type identifier.
#[derive(Debug, Clone, PartialEq)]
pub enum MediaType {
    Unknown = -1,
    Movie = 0,
    Show = 1,
    Episode = 2,
}

/// Basic identification information about a [Media] item.
pub trait MediaIdentifier: Debug {
    /// Get the unique ID of the media.
    fn id(&self) -> &String;

    /// Get the type of the media.
    fn media_type(&self) -> MediaType;

    /// The title of the media.
    fn title(&self) -> &String;
}

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

pub trait Media: MediaIdentifier + Watchable + Favorable {}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Movie {
    #[serde(rename(deserialize = "_id"))]
    id: String,
    title: String,
    imdb_id: String,
    tmdb_id: i32,
    year: String,
}

impl Movie {
    pub fn new(id: String, title: String) -> Self {
        Self {
            id,
            title,
            imdb_id: String::new(),
            tmdb_id: -1,
            year: String::new(),
        }
    }
}

impl MediaIdentifier for Movie {
    fn id(&self) -> &String {
        &self.id
    }

    fn media_type(&self) -> MediaType {
        MediaType::Movie
    }

    fn title(&self) -> &String {
        &self.title
    }
}

impl Watchable for Movie {
    fn is_watched(&self) -> bool {
        todo!()
    }
}

impl Favorable for Movie {
    fn is_liked(&self) -> bool {
        todo!()
    }
}

impl Media for Movie {}

#[derive(Debug, Clone, PartialEq)]
pub struct Show {
    id: String,
    tvdb_id: String,
    title: String,
}

impl Show {
    pub fn new(id: String, tvdb_id: String, title: String) -> Self {
        Self {
            id,
            tvdb_id,
            title,
        }
    }
}

impl MediaIdentifier for Show {
    fn id(&self) -> &String {
        &self.id
    }

    fn media_type(&self) -> MediaType {
        MediaType::Show
    }

    fn title(&self) -> &String {
        &self.title
    }
}

impl Watchable for Show {
    fn is_watched(&self) -> bool {
        todo!()
    }
}

impl Favorable for Show {
    fn is_liked(&self) -> bool {
        todo!()
    }
}

impl Media for Show {}

#[derive(Debug, Clone, Display)]
#[display(fmt = "id: {}, title: {}, season: {}, episode: {}", id, title, season, episode)]
pub struct Episode {
    id: String,
    title: String,
    season: i32,
    episode: i32,
}

impl Episode {
    pub fn new(id: String, title: String, season: i32, episode: i32) -> Self {
        Self {
            id,
            title,
            season,
            episode,
        }
    }

    pub fn season(&self) -> &i32 {
        &self.season
    }

    pub fn episode(&self) -> &i32 {
        &self.episode
    }
}

impl MediaIdentifier for Episode {
    fn id(&self) -> &String {
        &self.id
    }

    fn media_type(&self) -> MediaType {
        MediaType::Episode
    }

    fn title(&self) -> &String {
        &self.title
    }
}

impl Watchable for Episode {
    fn is_watched(&self) -> bool {
        todo!()
    }
}
