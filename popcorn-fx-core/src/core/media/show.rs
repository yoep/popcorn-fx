use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::core::media::{Episode, Images, MediaDetails, MediaIdentifier, MediaOverview, MediaType, Rating};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display)]
#[display(fmt = "{{ShowOverview: imdb_id: {}, tvdb_id: {}, title: {}}}", imdb_id, tvdb_id, title)]
pub struct ShowOverview {
    imdb_id: String,
    tvdb_id: String,
    title: String,
    year: String,
    num_seasons: i32,
    images: Images,
    rating: Option<Rating>,
}

impl ShowOverview {
    pub fn new(imdb_id: String, tvdb_id: String, title: String, year: String,
               num_seasons: i32, images: Images, rating: Option<Rating>) -> Self {
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

    pub fn images(&self) -> &Images {
        &self.images
    }
}

impl MediaIdentifier for ShowOverview {
    fn imdb_id(&self) -> String {
        self.imdb_id.clone()
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
}

#[derive(Debug, Clone, PartialEq, Deserialize, Display)]
#[display(fmt = "{{ShowDetails: imdb_id: {}, tvdb_id: {}, title: {}}}", imdb_id, tvdb_id, title)]
pub struct ShowDetails {
    imdb_id: String,
    tvdb_id: String,
    title: String,
    year: String,
    num_seasons: i32,
    images: Images,
    rating: Option<Rating>,
    #[serde(rename(deserialize = "contextLocale"))]
    context_locale: String,
    synopsis: String,
    runtime: String,
    status: String,
    genres: Vec<String>,
    episodes: Vec<Episode>,
    #[serde(skip)]
    liked: Option<bool>,
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

    pub fn images(&self) -> &Images {
        &self.images
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

    pub fn runtime(&self) -> &String {
        &self.runtime
    }
}

impl MediaIdentifier for ShowDetails {
    fn imdb_id(&self) -> String {
        self.imdb_id.clone()
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
}

impl MediaDetails for ShowDetails {
    fn synopsis(&self) -> String {
        html_escape::decode_html_entities(&self.synopsis).into_owned()
    }
}