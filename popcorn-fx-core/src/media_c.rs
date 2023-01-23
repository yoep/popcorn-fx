use std::{mem, ptr};
use std::collections::HashMap;
use std::os::raw::c_char;

use log::{error, trace};

use crate::{from_c_into_boxed, from_c_string, from_c_vec, into_c_owned, to_c_string, to_c_vec};
use crate::core::media::{Episode, Genre, Images, MediaDetails, MediaIdentifier, MediaOverview, MovieDetails, MovieOverview, Rating, ShowDetails, ShowOverview, SortBy, TorrentInfo};
use crate::core::media::favorites::FavoriteEvent;

/// Structure defining a set of media items.
/// Each media items is separated in a specific implementation array.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct MediaSetC {
    /// The movie media items array.
    pub movies: *mut MovieOverviewC,
    pub movies_len: i32,
    /// The show media items array.
    pub shows: *mut ShowOverviewC,
    pub shows_len: i32,
}

impl MediaSetC {
    pub fn from_movies(movies: Vec<MovieOverview>) -> Self {
        let (movies, movies_len) = to_c_vec(movies.into_iter()
            .map(|e| MovieOverviewC::from(e))
            .collect());

        Self {
            movies,
            movies_len,
            shows: ptr::null_mut(),
            shows_len: 0,
        }
    }

    pub fn from_shows(shows: Vec<ShowOverview>) -> Self {
        let (shows,shows_len) = to_c_vec(shows.into_iter()
            .map(|e| ShowOverviewC::from(e))
            .collect());

        Self {
            movies: ptr::null_mut(),
            movies_len: 0,
            shows,
            shows_len
        }
    }

    pub fn movies(&self) -> Vec<MovieOverview> {
        if self.movies.is_null() {
            return vec![];
        }

        let movies: Vec<MovieOverviewC> = from_c_vec(self.movies, self.movies_len);

        movies.into_iter()
            .map(|e| e.to_struct())
            .collect()
    }

    pub fn shows(&self) -> Vec<ShowOverview> {
        if self.shows.is_null() {
            return vec![];
        }

        let shows: Vec<ShowOverviewC> = from_c_vec(self.shows, self.movies_len);

        shows.into_iter()
            .map(|e| e.to_struct())
            .collect()
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct VecFavoritesC {
    pub movies: *mut MovieOverviewC,
    pub movies_len: i32,
    pub shows: *mut ShowOverviewC,
    pub shows_len: i32,
}

impl VecFavoritesC {
    pub fn from(movies: Vec<MovieOverviewC>, shows: Vec<ShowOverviewC>) -> Self {
        let (movies, movies_len) = to_c_vec(movies);
        let (shows, shows_len) = to_c_vec(shows);

        Self {
            movies,
            movies_len,
            shows,
            shows_len,
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
            let owned = from_c_into_boxed(self.rating);
            rating = Some(owned.to_struct());
            mem::forget(owned);
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
    title: *const c_char,
    imdb_id: *const c_char,
    year: *const c_char,
    rating: *mut RatingC,
    images: ImagesC,
    synopsis: *const c_char,
    runtime: i32,
    trailer: *const c_char,
    genres: *mut *const c_char,
    genres_len: i32,
    torrents: *mut TorrentEntryC,
    torrents_len: i32,
}

impl MovieDetailsC {
    pub fn from(movie: MovieDetails) -> Self {
        trace!("Converting MovieDetails to C for {{{}}}", movie);
        let (genres, genres_len) = to_c_vec(movie.genres().iter()
            .map(|e| to_c_string(e.clone()))
            .collect());
        let (torrents, torrents_len) = to_c_vec(movie.torrents().iter()
            .map(|(k, v)| TorrentEntryC::from(k, v))
            .collect());

        Self {
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
            genres,
            genres_len,
            torrents,
            torrents_len,
        }
    }

    pub fn to_struct(&self) -> MovieDetails {
        trace!("Converting MovieDetails from C {:?}", self);
        let mut rating = None;
        trace!("Converting MovieDetails genres");
        let genres = from_c_vec(self.genres, self.genres_len).into_iter()
            .map(|e| from_c_string(e))
            .collect();

        if !self.rating.is_null() {
            trace!("Converting MovieDetails rating");
            let owned = from_c_into_boxed(self.rating);
            rating = Some(owned.to_struct());
            mem::forget(owned);
        }

        MovieDetails::new_detailed(
            from_c_string(self.title),
            from_c_string(self.imdb_id),
            from_c_string(self.year),
            self.runtime.to_string(),
            genres,
            from_c_string(self.synopsis),
            rating,
            self.images.to_struct(),
            from_c_string(self.trailer),
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
        trace!("Converting Show to C {}", show);
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
            let owned = from_c_into_boxed(self.rating);
            rating = Some(owned.to_struct());
            mem::forget(owned);
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
    runtime: i32,
    status: *const c_char,
    genres: *mut *const c_char,
    genres_len: i32,
    episodes: *mut EpisodeC,
    episodes_len: i32,
}

impl ShowDetailsC {
    pub fn from(show: ShowDetails) -> Self {
        trace!("Converting ShowDetails to C {}", show);
        let (genres, genres_len) = to_c_vec(show.genres().iter()
            .map(|e| to_c_string(e.clone()))
            .collect());
        let episodes = show.episodes().iter()
            .map(|e| EpisodeC::from(e.clone()))
            .collect();
        let (episodes, episodes_len) = to_c_vec(episodes);

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
            runtime: show.runtime().clone(),
            status: to_c_string(show.status().clone()),
            genres,
            genres_len,
            episodes,
            episodes_len,
        }
    }

    pub fn to_struct(&self) -> ShowDetails {
        trace!("Converting ShowDetails from C {:?}", self);
        let mut rating = None;

        if !self.rating.is_null() {
            let owned = from_c_into_boxed(self.rating);
            rating = Some(owned.to_struct());
            mem::forget(owned);
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
}

impl EpisodeC {
    pub fn from(episode: Episode) -> Self {
        trace!("Converting Episode to C {}", episode);
        let torrents = episode.torrents().iter()
            .map(|(k, v)| TorrentQualityC::from(k, v))
            .collect();
        let (torrents, len) = to_c_vec(torrents);

        Self {
            season: episode.season().clone() as i32,
            episode: episode.episode().clone() as i32,
            first_aired: episode.first_aired().clone() as i64,
            title: to_c_string(episode.title().clone()),
            synopsis: to_c_string(episode.synopsis()),
            tvdb_id: to_c_string(episode.tvdb_id().clone()),
            torrents,
            len,
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
pub struct MediaItemC {
    pub movie_overview: *mut MovieOverviewC,
    pub movie_details: *mut MovieDetailsC,
    pub show_overview: *mut ShowOverviewC,
    pub show_details: *mut ShowDetailsC,
}

impl MediaItemC {
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
    percentage: u16,
    watching: u32,
    votes: u32,
    loved: u32,
    hated: u32,
}

impl RatingC {
    pub fn from(rating: &Rating) -> Self {
        trace!("Converting Rating to C {:?}", rating);
        Self {
            percentage: rating.percentage().clone(),
            watching: rating.watching().clone(),
            votes: rating.votes().clone(),
            loved: rating.loved().clone(),
            hated: rating.hated().clone(),
        }
    }

    fn to_struct(&self) -> Rating {
        trace!("Converting Rating from C {:?}", self);
        Rating::new_with_metadata(
            self.percentage.clone(),
            self.watching.clone(),
            self.votes.clone(),
            self.loved.clone(),
            self.hated.clone(),
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
        trace!("Converting Images to C {{{}}}", images);
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
}

impl TorrentEntryC {
    fn from(language: &String, qualities: &HashMap<String, TorrentInfo>) -> Self {
        let (qualities, len) = to_c_vec(qualities.iter()
            .map(|(k, v)| TorrentQualityC::from(k, v))
            .collect());

        Self {
            language: to_c_string(language.clone()),
            qualities,
            len,
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

#[repr(C)]
#[derive(Debug)]
pub enum FavoriteEventC {
    /// Event indicating that the like state of a media item changed.
    ///
    /// * `*const c_char`   - The imdb id of the media item that changed.
    /// * `bool`            - The new like state of the media item.
    LikedStateChanged(*const c_char, bool)
}

impl FavoriteEventC {
    pub fn from(event: FavoriteEvent) -> Self {
        trace!("Converting FavoriteEvent to C {}", &event);
        match event {
            FavoriteEvent::LikedStateChanged(id, state) => Self::LikedStateChanged(to_c_string(id.clone()), state.clone()),
        }
    }
}