use std::collections::HashMap;

use derive_more::Display;
use log::warn;
use serde::{Deserialize, Serialize};

use crate::core::media::{Images, MediaDetails, MediaIdentifier, MediaOverview, MediaType, Rating, TorrentInfo};

/// The simple version of a media item representing a movie.
/// It contains only the basic information needed for search results.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display)]
#[display(fmt = "MovieOverview: {{imdb_id: {}, title: {}, year: {}}}", imdb_id, title, year)]
pub struct MovieOverview {
    /// The title of the movie
    pub title: String,
    /// The IMDB identifier of the movie
    pub imdb_id: String,
    /// The year the movie has been released
    pub year: String,
    pub rating: Option<Rating>,
    pub images: Images,
}

impl MovieOverview {
    pub fn new(title: String, imdb_id: String, year: String) -> Self {
        Self {
            title,
            imdb_id,
            year,
            rating: None,
            images: Images::none(),
        }
    }

    pub fn new_detailed(title: String, imdb_id: String, year: String, rating: Option<Rating>, images: Images) -> Self {
        Self {
            title,
            imdb_id,
            year,
            rating,
            images,
        }
    }

    pub fn images(&self) -> &Images {
        &self.images
    }
}

impl MediaIdentifier for MovieOverview {
    fn imdb_id(&self) -> &str {
        self.imdb_id.as_str()
    }

    fn media_type(&self) -> MediaType {
        MediaType::Movie
    }

    fn title(&self) -> String {
        html_escape::decode_html_entities(&self.title).into_owned()
    }
}

impl MediaOverview for MovieOverview {
    fn rating(&self) -> Option<&Rating> {
        match &self.rating {
            None => None,
            Some(e) => Some(e)
        }
    }

    fn year(&self) -> &String {
        &self.year
    }
}

/// The detailed version of a media item representing a movie.
/// It contains all information need for a movie description.
#[derive(Debug, Clone, PartialEq, Deserialize, Display)]
#[display(fmt = "MovieDetails: {{imdb_id: {}, title: {}, year: {}, runtime: {}}}", imdb_id, title, year, runtime)]
pub struct MovieDetails {
    pub title: String,
    pub imdb_id: String,
    pub year: String,
    pub runtime: String,
    pub genres: Vec<String>,
    pub synopsis: String,
    pub rating: Option<Rating>,
    pub images: Images,
    pub trailer: String,
    pub torrents: HashMap<String, HashMap<String, TorrentInfo>>,
}

impl MovieDetails {
    pub fn new(title: String, imdb_id: String, year: String) -> Self {
        Self {
            title,
            imdb_id,
            year,
            runtime: String::new(),
            genres: vec![],
            synopsis: String::new(),
            rating: None,
            images: Images::none(),
            trailer: String::new(),
            torrents: HashMap::new(),
        }
    }

    pub fn new_detailed(title: String, imdb_id: String, year: String, runtime: String, genres: Vec<String>,
                        synopsis: String, rating: Option<Rating>, images: Images, trailer: String) -> Self {
        Self {
            title,
            imdb_id,
            year,
            runtime,
            genres,
            synopsis,
            rating,
            images,
            trailer,
            torrents: HashMap::new(),
        }
    }

    pub fn images(&self) -> &Images {
        &self.images
    }

    pub fn trailer(&self) -> &String {
        &self.trailer
    }

    pub fn genres(&self) -> &Vec<String> {
        &self.genres
    }

    pub fn torrents(&self) -> &HashMap<String, HashMap<String, TorrentInfo>> {
        &self.torrents
    }

    pub fn to_overview(&self) -> MovieOverview {
        MovieOverview::new_detailed(
            self.title.clone(),
            self.imdb_id.clone(),
            self.year.clone(),
            self.rating.clone(),
            self.images.clone(),
        )
    }
}

impl MediaIdentifier for MovieDetails {
    fn imdb_id(&self) -> &str {
        self.imdb_id.as_str()
    }

    fn media_type(&self) -> MediaType {
        MediaType::Movie
    }

    fn title(&self) -> String {
        html_escape::decode_html_entities(&self.title).into_owned()
    }
}

impl MediaOverview for MovieDetails {
    fn rating(&self) -> Option<&Rating> {
        match &self.rating {
            None => None,
            Some(e) => Some(e)
        }
    }

    fn year(&self) -> &String {
        &self.year
    }
}

impl MediaDetails for MovieDetails {
    /// Retrieve the description of the [Media] item.
    /// The description is html decoded before it's returned.
    fn synopsis(&self) -> String {
        html_escape::decode_html_entities(&self.synopsis).into_owned()
    }

    fn runtime(&self) -> i32 {
        match self.runtime.parse::<i32>() {
            Ok(e) => e,
            Err(e) => {
                warn!("Runtime value {} is invalid, {}", &self.runtime, e);
                0
            }
        }
    }
}