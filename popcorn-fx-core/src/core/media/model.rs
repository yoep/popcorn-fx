use std::fmt::Debug;

use derive_more::Display;

use crate::core::media::{MediaIdentifier, MediaType, Watchable};

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
}

impl Watchable for Episode {
    fn is_watched(&self) -> bool {
        todo!()
    }
}
