use derive_more::Display;
use log::warn;
use serde::{Deserialize, Serialize};

use crate::core::media::{Episode, Images, MediaDetails, MediaIdentifier, MediaOverview, MediaType, Rating};

/// The show media information of a specific serie.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display)]
#[display(fmt = "{{ShowOverview: imdb_id: {}, tvdb_id: {}, title: {}}}", imdb_id, tvdb_id, title)]
pub struct ShowOverview {
    pub imdb_id: String,
    pub tvdb_id: String,
    pub title: String,
    pub year: String,
    pub num_seasons: i32,
    pub images: Images,
    pub rating: Option<Rating>,
}

impl ShowOverview {
    pub fn new(imdb_id: String, tvdb_id: String, title: String, year: String,
               num_seasons: i32, images: Images, rating: Option<Rating>) -> Self {
        if imdb_id.is_empty() {
            panic!("Show IMDB ID cannot be empty")
        }

        Self {
            imdb_id,
            tvdb_id,
            title,
            year,
            num_seasons,
            images,
            rating,
        }
    }

    pub fn tvdb_id(&self) -> &String {
        &self.tvdb_id
    }

    /// The currently known number of seasons for the show.
    pub fn number_of_seasons(&self) -> &i32 {
        &self.num_seasons
    }
}

impl MediaIdentifier for ShowOverview {
    fn imdb_id(&self) -> &str {
        self.imdb_id.as_str()
    }

    fn media_type(&self) -> MediaType {
        MediaType::Show
    }

    fn title(&self) -> String {
        html_escape::decode_html_entities(&self.title).into_owned()
    }
}

impl MediaOverview for ShowOverview {
    fn rating(&self) -> Option<&Rating> {
        match &self.rating {
            None => None,
            Some(e) => Some(e)
        }
    }

    fn year(&self) -> &String {
        &self.year
    }

    fn images(&self) -> &Images {
        &self.images
    }
}

/// The details of a show/serie which contains one or more [Episode] items.
#[derive(Debug, Clone, PartialEq, Deserialize, Display)]
#[display(fmt = "{{ShowDetails: imdb_id: {}, tvdb_id: {}, title: {}}}", imdb_id, tvdb_id, title)]
pub struct ShowDetails {
    pub imdb_id: String,
    pub tvdb_id: String,
    pub title: String,
    pub year: String,
    pub num_seasons: i32,
    pub images: Images,
    pub rating: Option<Rating>,
    #[serde(rename(deserialize = "contextLocale"))]
    pub context_locale: String,
    pub synopsis: String,
    pub runtime: String,
    pub status: String,
    pub genres: Vec<String>,
    pub episodes: Vec<Episode>,
    #[serde(skip)]
    pub liked: Option<bool>,
}

impl ShowDetails {
    pub fn new(imdb_id: String, tvdb_id: String, title: String, year: String, num_seasons: i32,
               images: Images, rating: Option<Rating>) -> Self {
        Self {
            tvdb_id,
            title,
            imdb_id,
            year,
            rating,
            images,
            num_seasons,
            context_locale: "".to_string(),
            synopsis: "".to_string(),
            runtime: "".to_string(),
            status: "".to_string(),
            genres: vec![],
            episodes: vec![],
            liked: None,
        }
    }

    pub fn tvdb_id(&self) -> &String {
        &self.tvdb_id
    }

    /// The currently known number of seasons for the show.
    pub fn number_of_seasons(&self) -> &i32 {
        &self.num_seasons
    }

    pub fn status(&self) -> &String {
        &self.status
    }

    pub fn genres(&self) -> &Vec<String> {
        &self.genres
    }

    pub fn episodes(&self) -> &Vec<Episode> {
        &self.episodes
    }

    pub fn to_overview(&self) -> ShowOverview {
        ShowOverview::new(
            self.imdb_id.clone(),
            self.tvdb_id.clone(),
            self.title.clone(),
            self.year.clone(),
            self.num_seasons.clone(),
            self.images.clone(),
            self.rating.clone(),
        )
    }
}

impl MediaIdentifier for ShowDetails {
    fn imdb_id(&self) -> &str {
        self.imdb_id.as_str()
    }

    fn media_type(&self) -> MediaType {
        MediaType::Show
    }

    fn title(&self) -> String {
        html_escape::decode_html_entities(&self.title).into_owned()
    }
}

impl MediaOverview for ShowDetails {
    fn rating(&self) -> Option<&Rating> {
        match &self.rating {
            None => None,
            Some(e) => Some(e)
        }
    }

    fn year(&self) -> &String {
        &self.year
    }

    fn images(&self) -> &Images {
        &self.images
    }
}

impl MediaDetails for ShowDetails {
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