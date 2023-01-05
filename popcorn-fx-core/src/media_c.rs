use std::os::raw::c_char;

use crate::{from_c_string, to_c_string};
use crate::core::media::{Episode, MediaIdentifier, Movie, Show};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct MovieC {
    id: *const c_char,
    title: *const c_char,
}

impl MovieC {
    pub fn from(movie: Movie) -> Self {
        Self {
            id: to_c_string(movie.id().clone()),
            title: to_c_string(movie.title().clone()),
        }
    }

    pub fn to_struct(&self) -> Movie {
        Movie::new(
            from_c_string(self.id),
            from_c_string(self.title),
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct ShowC {
    id: *mut c_char,
    tvdb_id: *mut c_char,
    title: *mut c_char,
}

impl ShowC {
    pub fn to_struct(&self) -> Show {
        Show::new(
            from_c_string(self.id),
            from_c_string(self.tvdb_id),
            from_c_string(self.title),
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct EpisodeC {
    id: *mut c_char,
    title: *mut c_char,
    season: i32,
    episode: i32
}

impl EpisodeC {
    pub fn to_struct(&self) -> Episode {
        Episode::new(
            from_c_string(self.id),
            from_c_string(self.title),
            self.season.clone(),
            self.episode.clone()
        )
    }
}