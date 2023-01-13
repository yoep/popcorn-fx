use std::collections::HashMap;
use std::os::raw::c_char;
use std::ptr;

use log::{error, trace};

use crate::{from_c_owned, from_c_string, into_c_owned, to_c_string, to_c_vec};
use crate::core::media::{Episode, Genre, Images, MediaDetails, MediaIdentifier, MediaOverview, MovieDetails, MovieOverview, Rating, ShowDetails, ShowOverview, SortBy, TorrentInfo};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct VecMovieC {
    pub movies: *mut MovieOverviewC,
    pub len: i32,
    pub cap: i32,
}

impl VecMovieC {
    pub fn from(movies: Vec<MovieOverviewC>) -> Self {
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
    pub shows: *mut ShowOverviewC,
    pub len: i32,
    pub cap: i32,
}

impl VecShowC {
    pub fn from(shows: Vec<ShowOverviewC>) -> Self {
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
pub struct VecFavoritesC {
    pub movies: *mut MovieOverviewC,
    pub movies_len: i32,
    pub movies_cap: i32,
    pub shows: *mut ShowOverviewC,
    pub shows_len: i32,
    pub shows_cap: i32,
}

impl VecFavoritesC {
    pub fn from(movies: Vec<MovieOverviewC>, shows: Vec<ShowOverviewC>) -> Self {
        let (movies, movies_len, movies_cap) = to_c_vec(movies);
        let (shows, shows_len, shows_cap) = to_c_vec(shows);

        Self {
            movies,
            movies_len,
            movies_cap,
            shows,
            shows_len,
            shows_cap,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct MovieOverviewC {
    title: *const c_char,
    imdb_id: *const c_char,
    year: *const c_char,
    rating: *mut RatingC,
    images: ImagesC,
}

impl MovieOverviewC {
    pub fn from(movie: MovieOverview) -> Self {
        Self {
            title: to_c_string(movie.title()),
            imdb_id: to_c_string(movie.imdb_id().clone()),
            year: to_c_string(movie.year().clone()),
            rating: match movie.rating() {
                None => ptr::null_mut(),
                Some(e) => into_c_owned(RatingC::from(e))
            },
            images: ImagesC::from(movie.images()),
        }
    }

    pub fn to_struct(&self) -> MovieOverview {
        trace!("Converting MovieOverview from C {:?}", self);
        let mut rating = None;

        if !self.rating.is_null() {
            let c = from_c_owned(self.rating);
            rating = Some(c.to_struct());
        }

        MovieOverview::new_detailed(
            from_c_string(self.title),
            from_c_string(self.imdb_id),
            from_c_string(self.year),
            rating,
            self.images.to_struct(),
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct MovieDetailsC {
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

impl MovieDetailsC {
    pub fn from(movie: MovieDetails) -> Self {
        let (torrents, torrents_len, torrents_cap) = to_c_vec(movie.torrents().iter()
            .map(|(k, v)| TorrentEntryC::from(k, v))
            .collect());

        Self {
            id: to_c_string(movie.id()),
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

    pub fn to_struct(&self) -> MovieDetails {
        trace!("Converting MovieDetails from C {:?}", self);
        MovieDetails::new(
            from_c_string(self.id),
            from_c_string(self.title),
            from_c_string(self.imdb_id),
            from_c_string(self.year),
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct ShowOverviewC {
    imdb_id: *const c_char,
    tvdb_id: *const c_char,
    title: *const c_char,
    year: *const c_char,
    num_seasons: i32,
    images: ImagesC,
    rating: *mut RatingC,
}

impl ShowOverviewC {
    pub fn from(show: ShowOverview) -> Self {
        Self {
            imdb_id: to_c_string(show.imdb_id().clone()),
            tvdb_id: to_c_string(show.tvdb_id().clone()),
            title: to_c_string(show.title()),
            year: to_c_string(show.year().clone()),
            num_seasons: show.number_of_seasons().clone(),
            images: ImagesC::from(show.images()),
            rating: match show.rating() {
                None => ptr::null_mut(),
                Some(e) => into_c_owned(RatingC::from(e))
            },
        }
    }

    pub fn to_struct(&self) -> ShowOverview {
        trace!("Converting Show from C {:?}", self);
        let mut rating: Option<Rating> = None;

        if !self.rating.is_null() {
            let rating_c = from_c_owned(self.rating);
            rating = Some(rating_c.to_struct());
        }

        ShowOverview::new(
            from_c_string(self.imdb_id),
            from_c_string(self.tvdb_id),
            from_c_string(self.title),
            from_c_string(self.year),
            self.num_seasons.clone(),
            self.images.to_struct(),
            rating,
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct ShowDetailsC {
    imdb_id: *const c_char,
    tvdb_id: *const c_char,
    title: *const c_char,
    year: *const c_char,
    num_seasons: i32,
    images: ImagesC,
    rating: *mut RatingC,
    synopsis: *const c_char,
    runtime: *const c_char,
    status: *const c_char,
    episodes: *mut EpisodeC,
    episodes_len: i32,
    episodes_cap: i32,
}

impl ShowDetailsC {
    pub fn from(show: ShowDetails) -> Self {
        trace!("Converting ShowDetails to C {}", show);
        let episodes = show.episodes().iter()
            .map(|e| EpisodeC::from(e.clone()))
            .collect();
        let (episodes, episodes_len, episodes_cap) = to_c_vec(episodes);

        Self {
            imdb_id: to_c_string(show.imdb_id().clone()),
            tvdb_id: to_c_string(show.tvdb_id().clone()),
            title: to_c_string(show.title()),
            year: to_c_string(show.year().clone()),
            num_seasons: show.number_of_seasons().clone(),
            images: ImagesC::from(show.images()),
            rating: match show.rating() {
                None => ptr::null_mut(),
                Some(e) => into_c_owned(RatingC::from(e))
            },
            synopsis: to_c_string(show.synopsis().clone()),
            runtime: to_c_string(show.runtime().clone()),
            status: to_c_string(show.status().clone()),
            episodes,
            episodes_len,
            episodes_cap,
        }
    }

    pub fn to_struct(&self) -> ShowDetails {
        let mut rating = None;

        if !self.rating.is_null() {
            rating = Some(from_c_owned(self.rating).to_struct());
        }

        ShowDetails::new(
            from_c_string(self.imdb_id),
            from_c_string(self.tvdb_id),
            from_c_string(self.title),
            from_c_string(self.year),
            self.num_seasons.clone(),
            self.images.to_struct(),
            rating,
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct EpisodeC {
    season: i32,
    episode: i32,
    first_aired: i64,
    title: *const c_char,
    synopsis: *const c_char,
    tvdb_id: *const c_char,
    torrents: *mut TorrentQualityC,
    len: i32,
    cap: i32,
}

impl EpisodeC {
    pub fn from(episode: Episode) -> Self {
        trace!("Converting Episode to C {}", episode);
        let torrents = episode.torrents().iter()
            .map(|(k, v)| TorrentQualityC::from(k, v))
            .collect();
        let (torrents, len, cap) = to_c_vec(torrents);

        Self {
            season: episode.season().clone() as i32,
            episode: episode.episode().clone() as i32,
            first_aired: episode.first_aired().clone() as i64,
            title: to_c_string(episode.title().clone()),
            synopsis: to_c_string(episode.synopsis()),
            tvdb_id: to_c_string(episode.tvdb_id().clone()),
            torrents,
            len,
            cap,
        }
    }

    pub fn to_struct(&self) -> Episode {
        trace!("Converting Episode from C {:?}", self);
        let tvdb_id = match from_c_string(self.tvdb_id).parse::<i32>() {
            Ok(e) => e,
            Err(e) => {
                error!("Episode TVDB ID is invalid, {}", e);
                -1
            }
        };

        Episode::new(
            self.season.clone() as u32,
            self.episode.clone() as u32,
            self.first_aired.clone() as u64,
            from_c_string(self.title),
            from_c_string(self.synopsis),
            tvdb_id,
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FavoriteC {
    pub movie_overview: *mut MovieOverviewC,
    pub movie_details: *mut MovieDetailsC,
    pub show_overview: *mut ShowDetailsC,
    pub show_details: *mut ShowDetailsC,
}

impl FavoriteC {
    pub fn from_movie(media: MovieOverview) -> Self {
        Self {
            movie_overview: into_c_owned(MovieOverviewC::from(media)),
            movie_details: ptr::null_mut(),
            show_overview: ptr::null_mut(),
            show_details: ptr::null_mut(),
        }
    }

    pub fn from_movie_details(media: MovieDetails) -> Self {
        Self {
            movie_overview: ptr::null_mut(),
            movie_details: into_c_owned(MovieDetailsC::from(media)),
            show_overview: ptr::null_mut(),
            show_details: ptr::null_mut(),
        }
    }

    pub fn from_show_details(media: ShowDetails) -> Self {
        Self {
            movie_overview: ptr::null_mut(),
            movie_details: ptr::null_mut(),
            show_overview: ptr::null_mut(),
            show_details: into_c_owned(ShowDetailsC::from(media)),
        }
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

    fn to_struct(&self) -> Rating {
        Rating::new_with_metadata(
            self.percentage.clone() as u16,
            self.watching.clone() as u32,
            self.votes.clone() as u32,
            self.loved.clone() as u32,
            self.hated.clone() as u32,
        )
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
        trace!("Converting Images to C {}", images);
        Self {
            poster: to_c_string(images.poster().clone()),
            fanart: to_c_string(images.fanart().clone()),
            banner: to_c_string(images.banner().clone()),
        }
    }

    fn to_struct(&self) -> Images {
        trace!("Converting Images from C {:?}", self);
        Images::new(
            from_c_string(self.poster),
            from_c_string(self.fanart),
            from_c_string(self.banner),
        )
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
    file: *const c_char,
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
            size: match info.size() {
                None => ptr::null(),
                Some(e) => to_c_string(e.clone())
            },
            filesize: match info.filesize() {
                None => ptr::null(),
                Some(e) => to_c_string(e.clone())
            },
            file: match info.file() {
                None => ptr::null(),
                Some(e) => to_c_string(e.clone())
            },
        }
    }
}