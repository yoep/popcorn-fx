use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;

use derive_more::Display;
use log::warn;
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
pub trait MediaIdentifier: Debug + Any {
    /// Get the unique ID of the media.
    fn id(&self) -> &String;

    /// Get the type of the media.
    fn media_type(&self) -> MediaType;

    /// The title of the media item.
    /// The title is html decoded before it's returned.
    fn title(&self) -> String;

    fn as_any(&self) -> &dyn Any;
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

pub trait Media: MediaIdentifier + Watchable + Favorable {
}

/// The rating information of a [Media] item.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Rating {
    percentage: u16,
    watching: u32,
    votes: u32,
    loved: u32,
    hated: u32,
}

impl Rating {
    pub fn new(percentage: u16) -> Self {
        Self {
            percentage,
            watching: 0,
            votes: 0,
            loved: 0,
            hated: 0,
        }
    }

    pub fn new_with_metadata(percentage: u16, watching: u32, votes: u32, loved: u32, hated: u32) -> Self {
        Self {
            percentage,
            watching,
            votes,
            loved,
            hated,
        }
    }

    pub fn percentage(&self) -> &u16 {
        &self.percentage
    }

    pub fn watching(&self) -> &u32 {
        &self.watching
    }

    pub fn votes(&self) -> &u32 {
        &self.votes
    }

    pub fn loved(&self) -> &u32 {
        &self.loved
    }

    pub fn hated(&self) -> &u32 {
        &self.hated
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Images {
    poster: String,
    fanart: String,
    banner: String,
}

impl Images {
    pub fn new(poster: String, fanart: String, banner: String) -> Self {
        Self {
            poster,
            fanart,
            banner,
        }
    }

    /// Retrieve an empty [Images] struct which contains all empty strings.
    pub fn none() -> Self {
        Self {
            poster: String::new(),
            fanart: String::new(),
            banner: String::new(),
        }
    }

    pub fn poster(&self) -> &String {
        &self.poster
    }

    pub fn fanart(&self) -> &String {
        &self.fanart
    }

    pub fn banner(&self) -> &String {
        &self.banner
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct TorrentInfo {
    url: String,
    provider: String,
    source: String,
    title: String,
    quality: String,
    seed: u32,
    peer: u32,
    size: String,
    filesize: String,
}

impl TorrentInfo {
    pub fn new(url: String, title: String, quality: String) -> Self {
        Self {
            url,
            provider: String::new(),
            source: String::new(),
            title,
            quality,
            seed: 0,
            peer: 0,
            size: String::new(),
            filesize: String::new(),
        }
    }

    pub fn url(&self) -> &String {
        &self.url
    }

    pub fn provider(&self) -> &String {
        &self.provider
    }

    pub fn source(&self) -> &String {
        &self.source
    }

    pub fn title(&self) -> &String {
        &self.title
    }

    pub fn quality(&self) -> &String {
        &self.quality
    }

    pub fn seed(&self) -> &u32 {
        &self.seed
    }

    pub fn peer(&self) -> &u32 {
        &self.peer
    }

    pub fn size(&self) -> &String {
        &self.size
    }

    pub fn filesize(&self) -> &String {
        &self.filesize
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Display)]
#[display(fmt = "id: {}, title: {}, imdb_id: {}", id, title, imdb_id)]
pub struct Movie {
    #[serde(rename(deserialize = "_id"))]
    id: String,
    title: String,
    imdb_id: String,
    tmdb_id: i32,
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

impl Movie {
    pub fn new(id: String, title: String, imdb_id: String, year: String, runtime: i32) -> Self {
        Self {
            id,
            title,
            imdb_id,
            tmdb_id: -1,
            year,
            original_language: "en".to_string(),
            runtime: runtime.to_string(),
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

    /// Retrieve the description of the [Media] item.
    /// The description is html decoded before it's returned.
    pub fn synopsis(&self) -> String {
        html_escape::decode_html_entities(&self.synopsis).into_owned()
    }

    pub fn trailer(&self) -> &String {
        &self.trailer
    }

    pub fn torrents(&self) -> &HashMap<String, HashMap<String, TorrentInfo>> {
        &self.torrents
    }
}

impl MediaIdentifier for Movie {
    fn id(&self) -> &String {
        &self.id
    }

    fn media_type(&self) -> MediaType {
        MediaType::Movie
    }

    fn title(&self) -> String {
        html_escape::decode_html_entities(&self.title).into_owned()
    }

    fn as_any(&self) -> &dyn Any {
        self
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
    imdb_id: String,
    year: String,
    runtime: String,
    rating: Option<Rating>,
    images: Images,
    synopsis: String
}

impl Show {
    pub fn new(id: String, tvdb_id: String, title: String, imdb_id: String) -> Self {
        Self {
            id,
            tvdb_id,
            title,
            imdb_id,
            year: String::new(),
            runtime: String::new(),
            rating: None,
            images: Images::none(),
            synopsis: "".to_string(),
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

    fn title(&self) -> String {
        html_escape::decode_html_entities(&self.title).into_owned()
    }

    fn as_any(&self) -> &dyn Any {
        self
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

    fn title(&self) -> String {
        html_escape::decode_html_entities(&self.title).into_owned()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Watchable for Episode {
    fn is_watched(&self) -> bool {
        todo!()
    }
}
