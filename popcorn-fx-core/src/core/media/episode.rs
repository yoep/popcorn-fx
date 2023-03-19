use std::collections::HashMap;
use std::fmt::Debug;

use derive_more::Display;
use serde::Deserialize;

use crate::core::media::{MediaIdentifier, MediaType, TorrentInfo};

/// The episode media information of a show media item.
#[derive(Debug, Clone, PartialEq, Deserialize, Display)]
#[display(fmt = "tvdb_id: {}, title: {}, season: {}, episode: {}", tvdb_id, title, season, episode)]
pub struct Episode {
    pub season: u32,
    pub episode: u32,
    pub first_aired: u64,
    pub title: String,
    pub overview: String,
    pub tvdb_id: i32,
    #[serde(skip, default)]
    pub tvdb_id_value: String,
    /// The thumbnail of the episode if available
    pub thumb: Option<String>,
    pub torrents: HashMap<String, TorrentInfo>,
}

impl Episode {
    pub fn new(season: u32, episode: u32, first_aired: u64, title: String, overview: String, tvdb_id: i32) -> Self {
        Self {
            season,
            episode,
            first_aired,
            title,
            overview,
            tvdb_id,
            tvdb_id_value: tvdb_id.to_string(),
            thumb: None,
            torrents: HashMap::new(),
        }
    }

    pub fn new_with_torrents(season: u32, episode: u32, first_aired: u64, title: String, overview: String, tvdb_id: i32, torrents: HashMap<String, TorrentInfo>) -> Self {
        Self {
            season,
            episode,
            first_aired,
            title,
            overview,
            tvdb_id,
            tvdb_id_value: tvdb_id.to_string(),
            thumb: None,
            torrents,
        }
    }

    pub fn tvdb_id(&self) -> String {
        self.tvdb_id.to_string()
    }

    pub fn season(&self) -> &u32 {
        &self.season
    }

    pub fn episode(&self) -> &u32 {
        &self.episode
    }

    pub fn first_aired(&self) -> &u64 {
        &self.first_aired
    }

    /// Retrieve the description of the [Media] item.
    /// The description is html decoded before it's returned.
    pub fn synopsis(&self) -> String {
        html_escape::decode_html_entities(&self.overview).into_owned()
    }

    /// Retrieve the thumbnail url reference if available.
    pub fn thumb(&self) -> Option<&String> {
        self.thumb.as_ref()
    }

    pub fn torrents(&self) -> &HashMap<String, TorrentInfo> {
        &self.torrents
    }
}

impl MediaIdentifier for Episode {
    fn imdb_id(&self) -> &str {
        self.tvdb_id_value.as_str()
    }

    fn media_type(&self) -> MediaType {
        MediaType::Episode
    }

    fn title(&self) -> String {
        html_escape::decode_html_entities(&self.title).into_owned()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_id_should_return_tvdb_id() {
        let tvdb = 244587996;
        let episode = Episode::new(
            1,
            2,
            1673136000,
            "lorem".to_string(),
            "ipsum dolor".to_string(),
            tvdb.clone(),
        );
        let expected_result = tvdb.to_string();

        let result = episode.imdb_id();

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_imdb_id_should_return_tvdb_id() {
        let tvdb = 878787985;
        let episode = Episode::new(
            1,
            2,
            1673136000,
            "lorem".to_string(),
            "ipsum dolor".to_string(),
            tvdb.clone(),
        );
        let expected_result = tvdb.to_string();

        let result = episode.imdb_id();

        assert_eq!(expected_result, result)
    }
}