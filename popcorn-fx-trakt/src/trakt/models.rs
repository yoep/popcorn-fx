use derive_more::Display;
use serde::Deserialize;

use popcorn_fx_core::core::media::{MediaIdentifier, MediaType};

/// Represents an item in a watch list.
#[derive(Debug, Clone, Deserialize)]
pub struct WatchListItem {
    /// The rank of the item in the watch list.
    pub rank: i32,
    /// The time at which the item was listed.
    pub listed_at: String,
    /// The type of the item.
    #[serde(rename = "type")]
    pub trakt_type: TraktType,
    /// Information about the associated movie, if the item is a movie.
    pub movie: Option<Movie>,
    /// Information about the associated show, if the item is a show.
    pub show: Option<Show>,
}

/// Represents the type of an item in a watch list.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TraktType {
    /// Indicates that the item is a movie.
    Movie,
    /// Indicates that the item is a show.
    Show,
    /// Indicates that the item is a season of a show.
    Season,
    /// Indicates that the item is an episode of a show.
    Episode,
}

/// Represents a watched movie.
#[derive(Debug, Display, Clone, Deserialize, PartialEq)]
#[display(fmt = "imdb_id: {}, title: {}", "movie.ids.imdb", "movie.title")]
pub struct WatchedMovie {
    /// The movie being watched.
    pub movie: Movie,
}

impl MediaIdentifier for WatchedMovie {
    /// Gets the IMDb ID of the watched movie.
    fn imdb_id(&self) -> &str {
        self.movie.ids.imdb.as_str()
    }

    /// Gets the media type, which is `MediaType::Movie`.
    fn media_type(&self) -> MediaType {
        MediaType::Movie
    }

    /// Gets the title of the watched movie.
    fn title(&self) -> String {
        self.movie.title.clone()
    }
}

/// Represents information about a movie.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Movie {
    /// The title of the movie.
    pub title: String,
    /// The release year of the movie.
    pub year: i32,
    /// Unique identifiers for the movie.
    pub ids: MovieId
}

/// Represents unique identifiers for a movie.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct MovieId {
    /// The Trakt ID of the movie.
    pub trakt: i32,
    /// The slug of the movie.
    pub slug: String,
    /// The IMDb ID of the movie.
    pub imdb: String,
    /// The TMDb ID of the movie.
    pub tmdb: i32,
}

/// Represents information about a show.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Show {
    /// The title of the show.
    pub title: String,
    /// The release year of the show.
    pub year: i32,
    /// Unique identifiers for the show.
    pub ids: ShowId,
}

/// Represents unique identifiers for a show.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ShowId {
    /// The Trakt ID of the show.
    pub trakt: i32,
    /// The slug of the show.
    pub slug: String,
    /// The IMDb ID of the show.
    pub imdb: String,
    /// The TMDb ID of the show.
    pub tmdb: i32,
    /// The TVDB ID of the show.
    pub tvdb: i32,
}