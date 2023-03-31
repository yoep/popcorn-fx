use std::{mem, ptr};
use std::collections::HashMap;
use std::os::raw::c_char;

use log::{error, trace};
use thiserror::Error;

use popcorn_fx_core::{from_c_into_boxed, from_c_string, from_c_vec, into_c_owned, into_c_string, to_c_vec};
use popcorn_fx_core::core::media::{Episode, Genre, Images, MediaDetails, MediaIdentifier, MediaOverview, MovieDetails, MovieOverview, Rating, ShowDetails, ShowOverview, SortBy, TorrentInfo};
use popcorn_fx_core::core::media::favorites::FavoriteEvent;
use popcorn_fx_core::core::media::watched::WatchedEvent;

/// The C compatible media error types.
#[repr(i32)]
#[derive(Debug, Error)]
pub enum MediaErrorC {
    #[error("failed to retrieve media items")]
    Failed = 0,
    #[error("no media items are available")]
    NoItemsFound = 1
}

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
        let (shows, shows_len) = to_c_vec(shows.into_iter()
            .map(|e| ShowOverviewC::from(e))
            .collect());

        Self {
            movies: ptr::null_mut(),
            movies_len: 0,
            shows,
            shows_len,
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
            title: into_c_string(movie.title()),
            imdb_id: into_c_string(movie.imdb_id().to_string()),
            year: into_c_string(movie.year().clone()),
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

/// The C compatible [MovieDetails] representation
///
/// Use the [MovieDetails::from] to convert the C instance back to a rust struct.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct MovieDetailsC {
    pub title: *const c_char,
    pub imdb_id: *const c_char,
    pub year: *const c_char,
    pub rating: *mut RatingC,
    pub images: ImagesC,
    pub synopsis: *const c_char,
    pub runtime: i32,
    pub trailer: *const c_char,
    pub genres: *mut *const c_char,
    pub genres_len: i32,
    pub torrents: *mut TorrentEntryC,
    pub torrents_len: i32,
}

impl MovieDetailsC {
    pub fn from(movie: MovieDetails) -> Self {
        trace!("Converting MovieDetails to C for {{{}}}", movie);
        let (genres, genres_len) = to_c_vec(movie.genres().iter()
            .map(|e| into_c_string(e.clone()))
            .collect());
        let (torrents, torrents_len) = to_c_vec(movie.torrents().iter()
            .map(|(k, v)| TorrentEntryC::from(k, v))
            .collect());

        Self {
            title: into_c_string(movie.title()),
            imdb_id: into_c_string(movie.imdb_id().to_string()),
            year: into_c_string(movie.year().clone()),
            runtime: movie.runtime().clone(),
            rating: match movie.rating() {
                None => ptr::null_mut(),
                Some(e) => into_c_owned(RatingC::from(e))
            },
            images: ImagesC::from(movie.images()),
            synopsis: into_c_string(movie.synopsis().clone()),
            trailer: into_c_string(movie.trailer().clone()),
            genres,
            genres_len,
            torrents,
            torrents_len,
        }
    }
}

impl From<&MovieDetailsC> for MovieDetails {
    fn from(value: &MovieDetailsC) -> Self {
        trace!("Converting MovieDetails from C {:?}", value);
        let mut rating = None;
        let genres = if !value.genres.is_null() && value.genres_len > 0 {
            trace!("Converting MovieDetails genres {:?}", value.genres);
            from_c_vec(value.genres, value.genres_len).into_iter()
                .map(|e| from_c_string(e))
                .collect()
        } else {
            trace!("MovieDetails genres is empty, using empty array");
            vec![]
        };

        if !value.rating.is_null() {
            trace!("Converting MovieDetails rating");
            let owned = from_c_into_boxed(value.rating);
            rating = Some(owned.to_struct());
            mem::forget(owned);
        }

        MovieDetails::new_detailed(
            from_c_string(value.title.clone()),
            from_c_string(value.imdb_id.clone()),
            from_c_string(value.year.clone()),
            value.runtime.to_string(),
            genres,
            from_c_string(value.synopsis.clone()),
            rating,
            value.images.to_struct(),
            from_c_string(value.trailer.clone()),
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
            imdb_id: into_c_string(show.imdb_id().to_string()),
            tvdb_id: into_c_string(show.tvdb_id().clone()),
            title: into_c_string(show.title()),
            year: into_c_string(show.year().clone()),
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
            .map(|e| into_c_string(e.clone()))
            .collect());
        let episodes = show.episodes().iter()
            .map(|e| EpisodeC::from(e.clone()))
            .collect();
        let (episodes, episodes_len) = to_c_vec(episodes);

        Self {
            imdb_id: into_c_string(show.imdb_id().to_string()),
            tvdb_id: into_c_string(show.tvdb_id().clone()),
            title: into_c_string(show.title()),
            year: into_c_string(show.year().clone()),
            num_seasons: show.number_of_seasons().clone(),
            images: ImagesC::from(show.images()),
            rating: match show.rating() {
                None => ptr::null_mut(),
                Some(e) => into_c_owned(RatingC::from(e))
            },
            synopsis: into_c_string(show.synopsis().clone()),
            runtime: show.runtime().clone(),
            status: into_c_string(show.status().clone()),
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

/// The C compatible [Episode] media information.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct EpisodeC {
    pub season: i32,
    pub episode: i32,
    pub first_aired: i64,
    pub title: *const c_char,
    pub synopsis: *const c_char,
    pub tvdb_id: *const c_char,
    pub thumb: *const c_char,
    pub torrents: *mut TorrentQualityC,
    pub len: i32,
}

impl From<Episode> for EpisodeC {
    fn from(value: Episode) -> Self {
        trace!("Converting Episode to C {}", value);
        let torrents = value.torrents().iter()
            .map(|(k, v)| TorrentQualityC::from(k, v))
            .collect();
        let (torrents, len) = to_c_vec(torrents);

        Self {
            season: value.season().clone() as i32,
            episode: value.episode().clone() as i32,
            first_aired: value.first_aired().clone() as i64,
            title: into_c_string(value.title().clone()),
            synopsis: into_c_string(value.synopsis()),
            tvdb_id: into_c_string(value.tvdb_id().clone()),
            thumb: value.thumb()
                .map(|e| into_c_string(e.clone()))
                .or_else(|| Some(ptr::null()))
                .unwrap(),
            torrents,
            len,
        }
    }
}

impl From<&EpisodeC> for Episode {
    fn from(value: &EpisodeC) -> Self {
        trace!("Converting Episode from C {:?}", value);
        let tvdb_id = match from_c_string(value.tvdb_id).parse::<i32>() {
            Ok(e) => e,
            Err(e) => {
                error!("Episode TVDB ID is invalid, {}", e);
                -1
            }
        };
        let thumb = if !value.thumb.is_null() {
            Some(from_c_string(value.thumb))
        } else {
            None
        };

        Self {
            season: value.season as u32,
            episode: value.episode as u32,
            first_aired: value.first_aired as u64,
            title: from_c_string(value.title),
            overview: from_c_string(value.synopsis),
            tvdb_id,
            tvdb_id_value: tvdb_id.to_string(),
            thumb,
            torrents: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct MediaItemC {
    pub movie_overview: *mut MovieOverviewC,
    pub movie_details: *mut MovieDetailsC,
    pub show_overview: *mut ShowOverviewC,
    pub show_details: *mut ShowDetailsC,
    pub episode: *mut EpisodeC,
}

impl MediaItemC {
    pub fn from_show_details(media: ShowDetails) -> Self {
        Self {
            movie_overview: ptr::null_mut(),
            movie_details: ptr::null_mut(),
            show_overview: ptr::null_mut(),
            show_details: into_c_owned(ShowDetailsC::from(media)),
            episode: ptr::null_mut(),
        }
    }

    pub fn into_identifier(&self) -> Option<Box<dyn MediaIdentifier>> {
        let media: Box<dyn MediaIdentifier>;

        if !self.movie_overview.is_null() {
            let boxed = from_c_into_boxed(self.movie_overview);
            media = Box::new(boxed.to_struct());
            trace!("Created media struct {:?}", media);
            mem::forget(boxed);
        } else if !self.movie_details.is_null() {
            let boxed = from_c_into_boxed(self.movie_details);
            media = Box::new(MovieDetails::from(&*boxed));
            trace!("Created media struct {:?}", media);
            mem::forget(boxed);
        } else if !self.show_overview.is_null() {
            let boxed = from_c_into_boxed(self.show_overview);
            media = Box::new(boxed.to_struct());
            trace!("Created media struct {:?}", media);
            mem::forget(boxed);
        } else if !self.show_details.is_null() {
            let boxed = from_c_into_boxed(self.show_details);
            media = Box::new(boxed.to_struct());
            trace!("Created media struct {:?}", media);
            mem::forget(boxed);
        } else if !self.episode.is_null() {
            let boxed = from_c_into_boxed(self.episode);
            media = Box::new(Episode::from(&*boxed));
            trace!("Created media struct {:?}", media);
            mem::forget(boxed);
        } else {
            return None;
        }

        Some(media)
    }
}

impl From<MovieOverview> for MediaItemC {
    fn from(value: MovieOverview) -> Self {
        Self {
            movie_overview: into_c_owned(MovieOverviewC::from(value)),
            movie_details: ptr::null_mut(),
            show_overview: ptr::null_mut(),
            show_details: ptr::null_mut(),
            episode: ptr::null_mut(),
        }
    }
}

impl From<MovieDetails> for MediaItemC {
    fn from(value: MovieDetails) -> Self {
        Self {
            movie_overview: ptr::null_mut(),
            movie_details: into_c_owned(MovieDetailsC::from(value)),
            show_overview: ptr::null_mut(),
            show_details: ptr::null_mut(),
            episode: ptr::null_mut(),
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
    pub fn from(genre: Genre) -> Self {
        Self {
            key: into_c_string(genre.key().clone()),
            text: into_c_string(genre.text().clone()),
        }
    }

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
    pub fn from(sort_by: SortBy) -> Self {
        Self {
            key: into_c_string(sort_by.key().clone()),
            text: into_c_string(sort_by.text().clone()),
        }
    }

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

/// The C compatible [Images] representation.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ImagesC {
    pub poster: *const c_char,
    pub fanart: *const c_char,
    pub banner: *const c_char,
}

impl ImagesC {
    pub fn from(images: &Images) -> Self {
        trace!("Converting Images to C {{{}}}", images);
        Self {
            poster: into_c_string(images.poster().clone()),
            fanart: into_c_string(images.fanart().clone()),
            banner: into_c_string(images.banner().clone()),
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
            language: into_c_string(language.clone()),
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
            quality: into_c_string(quality.clone()),
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
            url: into_c_string(info.url().clone()),
            provider: into_c_string(info.provider().clone()),
            source: into_c_string(info.source().clone()),
            title: into_c_string(info.title().clone()),
            quality: into_c_string(info.quality().clone()),
            seed: info.seed().clone(),
            peer: info.peer().clone(),
            size: match info.size() {
                None => ptr::null(),
                Some(e) => into_c_string(e.clone())
            },
            filesize: match info.filesize() {
                None => ptr::null(),
                Some(e) => into_c_string(e.clone())
            },
            file: match info.file() {
                None => ptr::null(),
                Some(e) => into_c_string(e.clone())
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
            FavoriteEvent::LikedStateChanged(id, state) => Self::LikedStateChanged(into_c_string(id.clone()), state.clone()),
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub enum WatchedEventC {
    /// Event indicating that the watched state of a media item changed.
    ///
    /// * `*const c_char`   - The imdb id of the media item that changed.
    /// * `bool`            - The new watched state of the media item.
    WatchedStateChanged(*const c_char, bool)
}

impl WatchedEventC {
    pub fn from(event: WatchedEvent) -> Self {
        trace!("Converting WatchedEvent to C {}", &event);
        match event {
            WatchedEvent::WatchedStateChanged(id, state) => Self::WatchedStateChanged(into_c_string(id), state)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_episode() {
        let thumb = "http://localhost/thumb.jpg";
        let episode = Episode {
            season: 1,
            episode: 2,
            first_aired: 160000,
            title: "".to_string(),
            overview: "".to_string(),
            tvdb_id: 0,
            tvdb_id_value: "".to_string(),
            thumb: Some(thumb.to_string()),
            torrents: Default::default(),
        };

        let result = EpisodeC::from(episode);

        assert_eq!(1, result.season);
        assert_eq!(2, result.episode);
        assert_eq!(thumb.to_string(), from_c_string(result.thumb))
    }

    #[test]
    fn tets_from_episode_c() {
        let thumb = "http://localhost/episode_01.png";
        let episode = EpisodeC {
            season: 1,
            episode: 2,
            first_aired: 16000,
            title: into_c_string("lorem".to_string()),
            synopsis: into_c_string("ipsum".to_string()),
            tvdb_id: into_c_string("tt112244".to_string()),
            thumb: into_c_string(thumb.to_string()),
            torrents: ptr::null_mut(),
            len: 0,
        };

        let result = Episode::from(&episode);

        assert_eq!(1, result.season);
        assert_eq!(2, result.episode);
        assert_eq!(Some(thumb.to_string()), result.thumb);
    }

    #[test]
    fn test_from_movie_details_c() {
        let movie_c = MovieDetailsC {
            title: into_c_string("lorem".to_string()),
            imdb_id: into_c_string("tt1122".to_string()),
            year: into_c_string("2021".to_string()),
            rating: ptr::null_mut(),
            images: ImagesC {
                poster: ptr::null_mut(),
                fanart: ptr::null_mut(),
                banner: ptr::null_mut(),
            },
            synopsis: into_c_string("lorem ipsum dolor".to_string()),
            runtime: 20,
            trailer: into_c_string("https://www.youtube.com".to_string()),
            genres: ptr::null_mut(),
            genres_len: 0,
            torrents: ptr::null_mut(),
            torrents_len: 0,
        };
        let expected_result = MovieDetails {
            title: "lorem".to_string(),
            imdb_id: "tt1122".to_string(),
            year: "2021".to_string(),
            runtime: "20".to_string(),
            genres: vec![],
            synopsis: "lorem ipsum dolor".to_string(),
            rating: None,
            images: Default::default(),
            trailer: "https://www.youtube.com".to_string(),
            torrents: Default::default(),
        };

        let result = MovieDetails::from(&movie_c);

        assert_eq!(expected_result, result)
    }
}