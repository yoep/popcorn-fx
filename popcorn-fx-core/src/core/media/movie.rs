use std::collections::HashMap;

use derive_more::Display;
use log::warn;
use serde::Deserialize;

use crate::core::media::{Favorable, Images, MediaDetails, MediaIdentifier, MediaOverview, MediaType, Rating, TorrentInfo, Watchable};

/// The simple version of a media item representing a movie.
/// It contains only the basic information needed for search results.
#[derive(Debug, Clone, PartialEq, Deserialize, Display)]
#[display(fmt = "id: {}, title: {}, imdb_id: {}", id, title, imdb_id)]
pub struct MovieOverview {
    #[serde(rename(deserialize = "_id"))]
    id: String,
    title: String,
    imdb_id: String,
    year: String,
    rating: Option<Rating>,
    images: Images,
}

impl MovieOverview {
    pub fn new(id: String, title: String, imdb_id: String, year: String) -> Self {
        Self {
            id,
            title,
            imdb_id,
            year,
            rating: None,
            images: Images::none(),
        }
    }

    pub fn new_detailed(id: String, title: String, imdb_id: String, year: String,
                        rating: Option<Rating>, images: Images, ) -> Self {
        Self {
            id,
            title,
            imdb_id,
            year,
            rating,
            images,
        }
    }
}

impl MediaIdentifier for MovieOverview {
    fn id(&self) -> &String {
        &self.id
    }

    fn media_type(&self) -> MediaType {
        MediaType::Movie
    }

    fn title(&self) -> String {
        html_escape::decode_html_entities(&self.title).into_owned()
    }
}

impl Watchable for MovieOverview {
    fn is_watched(&self) -> bool {
        todo!()
    }
}

impl Favorable for MovieOverview {
    fn is_liked(&self) -> bool {
        todo!()
    }
}

impl MediaOverview for MovieOverview {}

/// The detailed version of a media item representing a movie.
/// It contains all information need for a movie description.
#[derive(Debug, Clone, PartialEq, Deserialize, Display)]
#[display(fmt = "id: {}, title: {}, imdb_id: {}", id, title, imdb_id)]
pub struct MovieDetails {
    #[serde(rename(deserialize = "_id"))]
    id: String,
    title: String,
    imdb_id: String,
    year: String,
    original_language: String,
    runtime: String,
    genres: Vec<String>,
    synopsis: String,
    rating: Option<Rating>,
    images: Images,
    released: i32,
    trailer: String,
    torrents: HashMap<String, HashMap<String, TorrentInfo>>,
}

impl MovieDetails {
    pub fn new(id: String, title: String, imdb_id: String, year: String) -> Self {
        Self {
            id,
            title,
            imdb_id,
            year,
            original_language: "en".to_string(),
            runtime: String::new(),
            genres: vec![],
            synopsis: String::new(),
            rating: None,
            images: Images::none(),
            released: 0,
            trailer: String::new(),
            torrents: HashMap::new(),
        }
    }

    pub fn imdb_id(&self) -> &String {
        &self.imdb_id
    }

    pub fn year(&self) -> &String {
        &self.year
    }

    pub fn runtime(&self) -> i32 {
        match self.runtime.parse::<i32>() {
            Ok(e) => e,
            Err(e) => {
                warn!("Runtime value {} is invalid, {}", &self.runtime, e);
                0
            }
        }
    }

    /// The rating of the movie if available.
    pub fn rating(&self) -> Option<&Rating> {
        match &self.rating {
            None => None,
            Some(e) => Some(e)
        }
    }

    pub fn images(&self) -> &Images {
        &self.images
    }

    pub fn trailer(&self) -> &String {
        &self.trailer
    }

    pub fn torrents(&self) -> &HashMap<String, HashMap<String, TorrentInfo>> {
        &self.torrents
    }
}

impl MediaIdentifier for MovieDetails {
    fn id(&self) -> &String {
        &self.id
    }

    fn media_type(&self) -> MediaType {
        MediaType::Movie
    }

    fn title(&self) -> String {
        html_escape::decode_html_entities(&self.title).into_owned()
    }
}

impl Watchable for MovieDetails {
    fn is_watched(&self) -> bool {
        todo!()
    }
}

impl Favorable for MovieDetails {
    fn is_liked(&self) -> bool {
        todo!()
    }
}

impl MediaOverview for MovieDetails {}

impl MediaDetails for MovieDetails {
    /// Retrieve the description of the [Media] item.
    /// The description is html decoded before it's returned.
    fn synopsis(&self) -> String {
        html_escape::decode_html_entities(&self.synopsis).into_owned()
    }
}