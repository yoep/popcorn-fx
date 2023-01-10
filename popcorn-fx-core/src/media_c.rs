use std::collections::HashMap;
use std::os::raw::c_char;
use std::ptr;

use log::trace;

use crate::{from_c_string, into_c_owned, to_c_string, to_c_vec};
use crate::core::media::{Episode, Genre, Images, MediaIdentifier, Movie, Rating, Show, SortBy, TorrentInfo};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct VecMovieC {
    pub movies: *mut MovieC,
    pub len: i32,
    pub cap: i32,
}

impl VecMovieC {
    pub fn from(movies: Vec<MovieC>) -> Self {
        let (movies, len, cap) = to_c_vec(movies);

        Self {
            movies,
            len,
            cap,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct VecShowC {
    pub shows: *mut ShowC,
    pub len: i32,
    pub cap: i32,
}

impl VecShowC {
    pub fn from(shows: Vec<ShowC>) -> Self {
        let (shows, len, cap) = to_c_vec(shows);

        Self {
            shows,
            len,
            cap,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct MovieC {
    id: *const c_char,
    title: *const c_char,
    imdb_id: *const c_char,
    year: *const c_char,
    runtime: i32,
    rating: *mut RatingC,
    images: ImagesC,
    synopsis: *const c_char,
    trailer: *const c_char,
    torrents: *mut TorrentEntryC,
    torrents_len: i32,
    torrents_cap: i32,
}

impl MovieC {
    pub fn from(movie: Movie) -> Self {
        let (torrents, torrents_len, torrents_cap) = to_c_vec(movie.torrents().iter()
            .map(|(k, v)| TorrentEntryC::from(k, v))
            .collect());

        Self {
            id: to_c_string(movie.id().clone()),
            title: to_c_string(movie.title()),
            imdb_id: to_c_string(movie.imdb_id().clone()),
            year: to_c_string(movie.year().clone()),
            runtime: movie.runtime().clone(),
            rating: match movie.rating() {
                None => ptr::null_mut(),
                Some(e) => into_c_owned(RatingC::from(e))
            },
            images: ImagesC::from(movie.images()),
            synopsis: to_c_string(movie.synopsis().clone()),
            trailer: to_c_string(movie.trailer().clone()),
            torrents,
            torrents_len,
            torrents_cap,
        }
    }

    pub fn to_struct(&self) -> Movie {
        Movie::new(
            from_c_string(self.id),
            from_c_string(self.title),
            from_c_string(self.imdb_id),
            from_c_string(self.year),
            self.runtime.clone(),
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct ShowC {
    id: *const c_char,
    imdb_id: *const c_char,
    tvdb_id: *const c_char,
    title: *const c_char,
    year: *const c_char,
    runtime: i32,
    num_seasons: i32,
    images: ImagesC,
    rating: *mut RatingC,
    synopsis: *const c_char,
}

impl ShowC {
    pub fn from(show: Show) -> Self {
        Self {
            id: to_c_string(show.id().clone()),
            imdb_id: to_c_string(show.imdb_id().clone()),
            tvdb_id: to_c_string(show.tvdb_id().clone()),
            title: to_c_string(show.title()),
            year: to_c_string(show.year().clone()),
            runtime: 0,
            num_seasons: show.number_of_seasons().clone(),
            images: ImagesC::from(show.images()),
            rating: match show.rating() {
                None => ptr::null_mut(),
                Some(e) => into_c_owned(RatingC::from(e))
            },
            synopsis: to_c_string(String::new()),
        }
    }

    pub fn to_struct(&self) -> Show {
        trace!("Converting Show from C {:?}", self);
        Show::new(
            from_c_string(self.id),
            from_c_string(self.tvdb_id),
            from_c_string(self.title),
            from_c_string(self.imdb_id),
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct EpisodeC {
    id: *const c_char,
    title: *const c_char,
    season: i32,
    episode: i32,
}

impl EpisodeC {
    pub fn to_struct(&self) -> Episode {
        trace!("Converting Episode from C {:?}", self);
        Episode::new(
            from_c_string(self.id),
            from_c_string(self.title),
            self.season.clone(),
            self.episode.clone(),
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct GenreC {
    key: *const c_char,
    text: *const c_char,
}

impl GenreC {
    pub fn to_struct(&self) -> Genre {
        trace!("Converting Genre from C {:?}", self);
        Genre::new(
            from_c_string(self.key),
            from_c_string(self.text),
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct SortByC {
    key: *const c_char,
    text: *const c_char,
}

impl SortByC {
    pub fn to_struct(&self) -> SortBy {
        trace!("Converting SortBy from C {:?}", self);
        SortBy::new(
            from_c_string(self.key),
            from_c_string(self.text),
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RatingC {
    percentage: i32,
    watching: i32,
    votes: i32,
    loved: i32,
    hated: i32,
}

impl RatingC {
    pub fn from(rating: &Rating) -> Self {
        Self {
            percentage: rating.percentage().clone() as i32,
            watching: rating.watching().clone() as i32,
            votes: rating.votes().clone() as i32,
            loved: rating.loved().clone() as i32,
            hated: rating.hated().clone() as i32,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct ImagesC {
    poster: *const c_char,
    fanart: *const c_char,
    banner: *const c_char,
}

impl ImagesC {
    pub fn from(images: &Images) -> Self {
        Self {
            poster: to_c_string(images.poster().clone()),
            fanart: to_c_string(images.fanart().clone()),
            banner: to_c_string(images.banner().clone()),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TorrentEntryC {
    language: *const c_char,
    qualities: *mut TorrentQualityC,
    len: i32,
    cap: i32,
}

impl TorrentEntryC {
    fn from(language: &String, qualities: &HashMap<String, TorrentInfo>) -> Self {
        let (qualities, len, cap) = to_c_vec(qualities.iter()
            .map(|(k, v)| TorrentQualityC::from(k, v))
            .collect());

        Self {
            language: to_c_string(language.clone()),
            qualities,
            len,
            cap,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TorrentQualityC {
    quality: *const c_char,
    torrent: TorrentInfoC,
}

impl TorrentQualityC {
    fn from(quality: &String, info: &TorrentInfo) -> Self {
        Self {
            quality: to_c_string(quality.clone()),
            torrent: TorrentInfoC::from(info),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TorrentInfoC {
    url: *const c_char,
    provider: *const c_char,
    source: *const c_char,
    title: *const c_char,
    quality: *const c_char,
    seed: u32,
    peer: u32,
    size: *const c_char,
    filesize: *const c_char,
}

impl TorrentInfoC {
    fn from(info: &TorrentInfo) -> Self {
        Self {
            url: to_c_string(info.url().clone()),
            provider: to_c_string(info.provider().clone()),
            source: to_c_string(info.source().clone()),
            title: to_c_string(info.title().clone()),
            quality: to_c_string(info.quality().clone()),
            seed: info.seed().clone(),
            peer: info.peer().clone(),
            size: to_c_string(info.size().clone()),
            filesize: to_c_string(info.filesize().clone()),
        }
    }
}