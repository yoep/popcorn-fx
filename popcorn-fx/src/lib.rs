extern crate core;

use std::{mem, ptr, slice};
use std::os::raw::c_char;

use log::{debug, error, info, trace, warn};
use tokio::runtime::Runtime;

pub use fx::*;
use popcorn_fx_core::{from_c_into_boxed, from_c_owned, from_c_string, into_c_owned, into_c_string, TorrentCollectionSet};
use popcorn_fx_core::core::config::{PlaybackSettings, ServerSettings, SubtitleSettings, TorrentSettings, UiSettings};
use popcorn_fx_core::core::events::PlayerStoppedEvent;
use popcorn_fx_core::core::media::*;
use popcorn_fx_core::core::media::favorites::FavoriteCallback;
use popcorn_fx_core::core::media::watched::WatchedCallback;
use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;
use popcorn_fx_core::core::subtitles::model::{Subtitle, SubtitleInfo, SubtitleType};
use popcorn_fx_core::core::torrent::{Torrent, TorrentState, TorrentStreamState};
use popcorn_fx_torrent_stream::{TorrentC, TorrentStreamC, TorrentStreamEventC, TorrentWrapperC};

#[cfg(feature = "ffi")]
use crate::ffi::*;

mod fx;
#[cfg(feature = "ffi")]
pub mod ffi;


/// Retrieve the available subtitles for the given [MovieDetailsC].
///
/// It returns a reference to [SubtitleInfoSet], else a [ptr::null_mut] on failure.
/// <i>The returned reference should be managed by the caller.</i>
#[no_mangle]
pub extern "C" fn movie_subtitles(popcorn_fx: &mut PopcornFX, movie: &MovieDetailsC) -> *mut SubtitleInfoSet {
    let movie_instance = MovieDetails::from(movie);

    match popcorn_fx.runtime().block_on(popcorn_fx.subtitle_provider().movie_subtitles(movie_instance)) {
        Ok(e) => {
            debug!("Found movie subtitles {:?}", e);
            let result: Vec<SubtitleInfoC> = e.into_iter()
                .map(|e| SubtitleInfoC::from(e))
                .collect();

            into_c_owned(SubtitleInfoSet::from(result))
        }
        Err(e) => {
            error!("Movie subtitle search failed, {}", e);
            ptr::null_mut()
        }
    }
}

/// Retrieve the given subtitles for the given episode
#[no_mangle]
pub extern "C" fn episode_subtitles(popcorn_fx: &mut PopcornFX, show: &ShowDetailsC, episode: &EpisodeC) -> *mut SubtitleInfoSet {
    let show_instance = show.to_struct();
    let episode_instance = Episode::from(episode);

    match popcorn_fx.runtime().block_on(popcorn_fx.subtitle_provider().episode_subtitles(show_instance, episode_instance)) {
        Ok(e) => {
            debug!("Found episode subtitles {:?}", e);
            let result: Vec<SubtitleInfoC> = e.into_iter()
                .map(|e| SubtitleInfoC::from(e))
                .collect();

            into_c_owned(SubtitleInfoSet::from(result))
        }
        Err(e) => {
            error!("Episode subtitle search failed, {}", e);
            into_c_owned(SubtitleInfoSet::from(vec![]))
        }
    }
}

/// Retrieve the available subtitles for the given filename
#[no_mangle]
pub extern "C" fn filename_subtitles(popcorn_fx: &mut PopcornFX, filename: *mut c_char) -> *mut SubtitleInfoSet {
    let filename_rust = from_c_string(filename);

    match popcorn_fx.runtime().block_on(popcorn_fx.subtitle_provider().file_subtitles(&filename_rust)) {
        Ok(e) => {
            debug!("Found filename subtitles {:?}", e);
            let result: Vec<SubtitleInfoC> = e.into_iter()
                .map(|e| SubtitleInfoC::from(e))
                .collect();

            into_c_owned(SubtitleInfoSet::from(result))
        }
        Err(e) => {
            error!("Filename subtitle search failed, {}", e);
            into_c_owned(SubtitleInfoSet::from(vec![]))
        }
    }
}

/// Select a default subtitle language based on the settings or user interface language.
#[no_mangle]
pub extern "C" fn select_or_default_subtitle(popcorn_fx: &mut PopcornFX, subtitles_ptr: *const SubtitleInfoC, len: usize) -> *mut SubtitleInfoC {
    let c_vec = unsafe { slice::from_raw_parts(subtitles_ptr, len).to_vec() };
    let subtitles: Vec<SubtitleInfo> = c_vec.iter()
        .map(|e| SubtitleInfo::from(e))
        .collect();

    let subtitle = into_c_owned(SubtitleInfoC::from(popcorn_fx.subtitle_provider().select_or_default(&subtitles[..])));

    mem::forget(c_vec);
    mem::forget(subtitles);
    mem::forget(subtitles_ptr);

    subtitle
}

/// Retrieve the preferred subtitle instance for the next [Media] item playback.
///
/// It returns the [SubtitleInfoC] when present, else [ptr::null_mut].
#[no_mangle]
pub extern "C" fn retrieve_preferred_subtitle(popcorn_fx: &mut PopcornFX) -> *mut SubtitleInfoC {
    match popcorn_fx.subtitle_manager().preferred_subtitle() {
        None => ptr::null_mut(),
        Some(e) => into_c_owned(SubtitleInfoC::from(e))
    }
}

/// Retrieve the preferred subtitle language for the next [Media] item playback.
///
/// It returns the preferred subtitle language.
#[no_mangle]
pub extern "C" fn retrieve_preferred_subtitle_language(popcorn_fx: &mut PopcornFX) -> SubtitleLanguage {
    popcorn_fx.subtitle_manager().preferred_language()
}

/// Verify if the subtitle has been disabled by the user.
///
/// It returns true when the subtitle track should be disabled, else false.
#[no_mangle]
pub extern "C" fn is_subtitle_disabled(popcorn_fx: &mut PopcornFX) -> bool {
    popcorn_fx.subtitle_manager().is_disabled()
}

/// Update the preferred subtitle for the [Media] item playback.
/// This action will reset any custom configured subtitle files.
#[no_mangle]
pub extern "C" fn update_subtitle(popcorn_fx: &mut PopcornFX, subtitle: &SubtitleInfoC) {
    popcorn_fx.subtitle_manager().update_subtitle(SubtitleInfo::from(subtitle))
}

/// Update the preferred subtitle to a custom subtitle filepath.
/// This action will reset any preferred subtitle.
#[no_mangle]
pub extern "C" fn update_subtitle_custom_file(popcorn_fx: &mut PopcornFX, custom_filepath: *const c_char) {
    let custom_filepath = from_c_string(custom_filepath);
    trace!("Updating custom subtitle filepath to {}", &custom_filepath);

    popcorn_fx.subtitle_manager().update_custom_subtitle(custom_filepath.as_str())
}

/// Disable the subtitle track on request of the user.
/// This will make the [is_subtitle_disabled] return `true`.
#[no_mangle]
pub extern "C" fn disable_subtitle(popcorn_fx: &mut PopcornFX) {
    trace!("Disabling the subtitle track");
    popcorn_fx.subtitle_manager().disable_subtitle()
}

/// Reset the current preferred subtitle configuration.
/// This will remove any selected [SubtitleInfo] or custom subtitle file.
#[no_mangle]
pub extern "C" fn reset_subtitle(popcorn_fx: &mut PopcornFX) {
    popcorn_fx.subtitle_manager().reset()
}

/// Download the given [SubtitleInfo] based on the best match according to the [SubtitleMatcher].
///
/// It returns the filepath to the subtitle on success, else [ptr::null_mut].
#[no_mangle]
pub extern "C" fn download(popcorn_fx: &mut PopcornFX, subtitle: &SubtitleInfoC, matcher: &SubtitleMatcherC) -> *const c_char {
    trace!("Starting subtitle download for info: {:?}, matcher: {:?}", subtitle, matcher);
    let subtitle_info = SubtitleInfo::from(subtitle);
    let matcher = matcher.to_matcher();

    match popcorn_fx.runtime().block_on(popcorn_fx.subtitle_provider().download(&subtitle_info, &matcher)) {
        Ok(e) => {
            debug!("Returning subtitle filepath {:?}", &e);
            into_c_string(e)
        }
        Err(e) => {
            error!("Failed to download subtitle, {}", e);
            ptr::null_mut()
        }
    }
}

/// Download and parse the given subtitle info.
///
/// It returns the [SubtitleC] reference on success, else [ptr::null_mut].
#[no_mangle]
pub extern "C" fn download_and_parse_subtitle(popcorn_fx: &mut PopcornFX, subtitle: &SubtitleInfoC, matcher: &SubtitleMatcherC) -> *mut SubtitleC {
    let subtitle_info = SubtitleInfo::from(subtitle);
    let matcher = matcher.to_matcher();

    match popcorn_fx.runtime().block_on(popcorn_fx.subtitle_provider().download_and_parse(&subtitle_info, &matcher)) {
        Ok(e) => {
            let result = SubtitleC::from(e);
            debug!("Returning parsed subtitle {:?}", result);
            into_c_owned(result)
        }
        Err(e) => {
            error!("Failed to download subtitle, {}", e);
            ptr::null_mut()
        }
    }
}

/// Retrieve the available movies for the given criteria.
///
/// It returns the [VecMovieC] reference on success, else [ptr::null_mut].
#[no_mangle]
pub extern "C" fn retrieve_available_movies(popcorn_fx: &mut PopcornFX, genre: &GenreC, sort_by: &SortByC, keywords: *const c_char, page: u32) -> *mut MediaSetC {
    let genre = genre.to_struct();
    let sort_by = sort_by.to_struct();
    let keywords = from_c_string(keywords);

    match popcorn_fx.runtime().block_on(popcorn_fx.providers().retrieve(&Category::Movies, &genre, &sort_by, &keywords, page)) {
        Ok(e) => {
            info!("Retrieved a total of {} movies, {:?}", e.len(), &e);
            let movies: Vec<MovieOverview> = e.into_iter()
                .map(|e| *e
                    .into_any()
                    .downcast::<MovieOverview>()
                    .expect("expected media to be a movie overview"))
                .collect();

            if movies.len() > 0 {
                into_c_owned(MediaSetC::from_movies(movies))
            } else {
                debug!("No movies have been found, returning ptr::null");
                ptr::null_mut()
            }
        }
        Err(e) => {
            error!("Failed to retrieve movies, {}", e);
            ptr::null_mut()
        }
    }
}

/// Retrieve the details of a given movie.
/// It will query the api for the given IMDB ID.
///
/// It returns the [MovieDetailsC] on success, else [ptr::null_mut].
#[no_mangle]
pub extern "C" fn retrieve_movie_details(popcorn_fx: &mut PopcornFX, imdb_id: *const c_char) -> *mut MovieDetailsC {
    let imdb_id = from_c_string(imdb_id);

    match popcorn_fx.runtime().block_on(popcorn_fx.providers().retrieve_details(&Category::Movies, &imdb_id)) {
        Ok(e) => {
            trace!("Returning movie details {:?}", &e);
            into_c_owned(MovieDetailsC::from(*e
                .into_any()
                .downcast::<MovieDetails>()
                .expect("expected media to be movie details")))
        }
        Err(e) => {
            error!("Failed to retrieve movie details, {}", e);
            ptr::null_mut()
        }
    }
}

/// Reset all available api stats for the movie api.
/// This will make all disabled api's available again.
#[no_mangle]
pub extern "C" fn reset_movie_apis(popcorn_fx: &mut PopcornFX) {
    popcorn_fx.providers().reset_api(&Category::Movies)
}

/// Retrieve the available [ShowOverviewC] items for the given criteria.
///
/// It returns an array of [ShowOverviewC] items on success, else a [ptr::null_mut].
#[no_mangle]
pub extern "C" fn retrieve_available_shows(popcorn_fx: &mut PopcornFX, genre: &GenreC, sort_by: &SortByC, keywords: *const c_char, page: u32) -> *mut MediaSetC {
    let genre = genre.to_struct();
    let sort_by = sort_by.to_struct();
    let keywords = from_c_string(keywords);

    match popcorn_fx.runtime().block_on(popcorn_fx.providers().retrieve(&Category::Series, &genre, &sort_by, &keywords, page)) {
        Ok(e) => {
            info!("Retrieved a total of {} shows, {:?}", e.len(), &e);
            let shows: Vec<ShowOverview> = e.into_iter()
                .map(|e| *e
                    .into_any()
                    .downcast::<ShowOverview>()
                    .expect("expected media to be a show"))
                .collect();

            if shows.len() > 0 {
                into_c_owned(MediaSetC::from_shows(shows))
            } else {
                debug!("No shows have been found, returning ptr::null");
                ptr::null_mut()
            }
        }
        Err(e) => {
            error!("Failed to retrieve movies, {}", e);
            ptr::null_mut()
        }
    }
}

/// Retrieve the details of a show based on the given IMDB ID.
/// The details contain all information about the show such as episodes and descriptions.
///
/// It returns the [ShowDetailsC] on success, else a [ptr::null_mut].
#[no_mangle]
pub extern "C" fn retrieve_show_details(popcorn_fx: &mut PopcornFX, imdb_id: *const c_char) -> *mut ShowDetailsC {
    let imdb_id = from_c_string(imdb_id);

    match popcorn_fx.runtime().block_on(popcorn_fx.providers().retrieve_details(&Category::Series, &imdb_id)) {
        Ok(e) => {
            trace!("Returning show details {:?}", &e);
            into_c_owned(ShowDetailsC::from(*e
                .into_any()
                .downcast::<ShowDetails>()
                .expect("expected media to be a show")))
        }
        Err(e) => {
            error!("Failed to retrieve show details, {}", e);
            ptr::null_mut()
        }
    }
}

/// Reset all available api stats for the movie api.
/// This will make all disabled api's available again.
#[no_mangle]
pub extern "C" fn reset_show_apis(popcorn_fx: &mut PopcornFX) {
    popcorn_fx.providers().reset_api(&Category::Series)
}

/// Retrieve all liked favorite media items.
///
/// It returns the [VecFavoritesC] holder for the array on success, else [ptr::null_mut].
#[no_mangle]
pub extern "C" fn retrieve_available_favorites(popcorn_fx: &mut PopcornFX, genre: &GenreC, sort_by: &SortByC, keywords: *const c_char, page: u32) -> *mut VecFavoritesC {
    trace!("Retrieving favorites from C for genre: {:?}, sort_by: {:?}, keywords: {:?}, page: {}", genre, sort_by, keywords, page);
    let genre = genre.to_struct();
    let sort_by = sort_by.to_struct();
    let keywords = from_c_string(keywords);

    trace!("Retrieving favorites for genre: {:?}, sort_by: {:?}, page: {}", genre, sort_by, page);
    match popcorn_fx.runtime().block_on(popcorn_fx.providers().retrieve(&Category::Favorites, &genre, &sort_by, &keywords, page)) {
        Ok(e) => {
            info!("Retrieved a total of {} favorites, {:?}", e.len(), &e);
            favorites_to_c(e)
        }
        Err(e) => {
            error!("Failed to retrieve favorites, {}", e);
            ptr::null_mut()
        }
    }
}

/// Retrieve the details of a favorite item on the given IMDB ID.
/// The details contain all information about the media item.
///
/// It returns the [MediaItemC] on success, else a [ptr::null_mut].
#[no_mangle]
pub extern "C" fn retrieve_favorite_details(popcorn_fx: &mut PopcornFX, imdb_id: *const c_char) -> *mut MediaItemC {
    let imdb_id = from_c_string(imdb_id);

    match popcorn_fx.runtime().block_on(popcorn_fx.providers().retrieve_details(&Category::Favorites, imdb_id.as_str())) {
        Ok(e) => {
            trace!("Returning favorite details {:?}", &e);
            match e.media_type() {
                MediaType::Movie => {
                    into_c_owned(MediaItemC::from(*e.into_any()
                        .downcast::<MovieDetails>()
                        .expect("expected the favorite item to be a movie")))
                }
                MediaType::Show => {
                    into_c_owned(MediaItemC::from_show_details(*e.into_any()
                        .downcast::<ShowDetails>()
                        .expect("expected the favorite item to be a show")))
                }
                _ => {
                    error!("Media type {} is not supported to retrieve favorite details", e.media_type());
                    ptr::null_mut()
                }
            }
        }
        Err(e) => {
            error!("Failed to retrieve favorite details, {}", e);
            ptr::null_mut()
        }
    }
}

/// Verify if the given media item is liked/favorite of the user.
/// It will use the first non [ptr::null_mut] field from the [MediaItemC] struct.
///
/// It will return false if all fields in the [MediaItemC] are [ptr::null_mut].
#[no_mangle]
pub extern "C" fn is_media_liked(popcorn_fx: &mut PopcornFX, favorite: &MediaItemC) -> bool {
    trace!("Verifying if media is liked for {:?}", favorite);
    match favorite.into_identifier() {
        None => {
            warn!("Unable to verify if media is liked, all FavoriteC fields are null");
            false
        }
        Some(media) => {
            let liked = popcorn_fx.favorite_service().is_liked_dyn(&media);
            trace!("Liked state is {} for {} {}", &liked, media.media_type(), media.imdb_id());
            mem::forget(media);
            liked
        }
    }
}

/// Retrieve all favorites of the user.
///
/// It will return an array of favorites on success, else [ptr::null_mut].
#[no_mangle]
pub extern "C" fn retrieve_all_favorites(popcorn_fx: &mut PopcornFX) -> *mut VecFavoritesC {
    match popcorn_fx.favorite_service().all() {
        Ok(e) => {
            favorites_to_c(e)
        }
        Err(e) => {
            error!("Failed to retrieve favorites, {}", e);
            ptr::null_mut()
        }
    }
}

/// Add the media item to the favorites.
/// Duplicate favorite media items are ignored.
#[no_mangle]
pub extern "C" fn add_to_favorites(popcorn_fx: &mut PopcornFX, favorite: &MediaItemC) {
    let media: Box<dyn MediaIdentifier>;

    if !favorite.movie_overview.is_null() {
        let boxed = from_c_into_boxed(favorite.movie_overview);
        media = Box::new(boxed.to_struct());
        trace!("Created media struct {:?}", media);
        mem::forget(boxed);
    } else if !favorite.movie_details.is_null() {
        let boxed = from_c_into_boxed(favorite.movie_details);
        let details = MovieDetails::from(&*boxed);
        media = Box::new(details.to_overview());
        trace!("Created media struct {:?}", media);
        mem::forget(boxed);
    } else if !favorite.show_overview.is_null() {
        let boxed = from_c_into_boxed(favorite.show_overview);
        media = Box::new(boxed.to_struct());
        trace!("Created media struct {:?}", media);
        mem::forget(boxed);
    } else if !favorite.show_details.is_null() {
        let boxed = from_c_into_boxed(favorite.show_details);
        let details = Box::new(boxed.to_struct());
        media = Box::new(details.to_overview());
        trace!("Created media struct {:?}", media);
        mem::forget(boxed);
    } else {
        error!("Unable to add favorite, all FavoriteC fields are null");
        return;
    }

    match popcorn_fx.favorite_service().add(media) {
        Ok(_) => {}
        Err(e) => error!("{}", e)
    }
}

/// Remove the media item from favorites.
#[no_mangle]
pub extern "C" fn remove_from_favorites(popcorn_fx: &mut PopcornFX, favorite: &MediaItemC) {
    match favorite.into_identifier() {
        None => error!("Unable to remove favorite, all FavoriteC fields are null"),
        Some(e) => popcorn_fx.favorite_service().remove(e)
    }
}

/// Register a new callback listener for favorite events.
#[no_mangle]
pub extern "C" fn register_favorites_event_callback<'a>(popcorn_fx: &mut PopcornFX, callback: extern "C" fn(FavoriteEventC)) {
    trace!("Wrapping C callback for FavoriteCallback");
    let wrapper: FavoriteCallback = Box::new(move |event| {
        callback(FavoriteEventC::from(event));
    });

    popcorn_fx.favorite_service().register(wrapper)
}

/// Serve the given subtitle as [SubtitleType] format.
///
/// It returns the url which hosts the [Subtitle].
#[no_mangle]
pub extern "C" fn serve_subtitle(popcorn_fx: &mut PopcornFX, subtitle: SubtitleC, output_type: usize) -> *const c_char {
    let subtitle = Subtitle::from(subtitle);
    let subtitle_type = SubtitleType::from_ordinal(output_type);

    match popcorn_fx.subtitle_server().serve(subtitle, subtitle_type) {
        Ok(e) => {
            info!("Serving subtitle at {}", &e);
            into_c_string(e)
        }
        Err(e) => {
            error!("Failed to serve subtitle, {}", e);
            ptr::null()
        }
    }
}

/// Verify if the given media item is watched by the user.
///
/// It returns true when the item is watched, else false.
#[no_mangle]
pub extern "C" fn is_media_watched(popcorn_fx: &mut PopcornFX, watchable: &MediaItemC) -> bool {
    match watchable.into_identifier() {
        Some(media) => {
            trace!("Verifying if media item is watched for {}", &media);
            let watched = popcorn_fx.watched_service().is_watched_dyn(&media);
            mem::forget(media);
            watched
        }
        None => {
            error!("Failed to verify the watched state, no watchable item given");
            false
        }
    }
}

/// Retrieve all watched media item id's.
///
/// It returns an array of watched id's.
#[no_mangle]
pub extern "C" fn retrieve_all_watched(popcorn_fx: &mut PopcornFX) -> StringArray {
    trace!("Retrieving all watched media id's");
    match popcorn_fx.watched_service().all() {
        Ok(e) => {
            debug!("Retrieved watched items {:?}", &e);
            StringArray::from(e)
        }
        Err(e) => {
            error!("Failed to retrieve watched items, {}", e);
            StringArray::from(vec![])
        }
    }
}

/// Retrieve all watched movie id's.
///
/// It returns an array of watched movie id's.
#[no_mangle]
pub extern "C" fn retrieve_watched_movies(popcorn_fx: &mut PopcornFX) -> StringArray {
    match popcorn_fx.watched_service().watched_movies() {
        Ok(e) => {
            debug!("Retrieved watched items {:?}", &e);
            StringArray::from(e)
        }
        Err(e) => {
            error!("Failed to retrieve watched items, {}", e);
            StringArray::from(vec![])
        }
    }
}

/// Retrieve all watched show media id's.
///
/// It returns  an array of watched show id's.
#[no_mangle]
pub extern "C" fn retrieve_watched_shows(popcorn_fx: &mut PopcornFX) -> StringArray {
    match popcorn_fx.watched_service().watched_shows() {
        Ok(e) => {
            debug!("Retrieved watched items {:?}", &e);
            StringArray::from(e)
        }
        Err(e) => {
            error!("Failed to retrieve watched items, {}", e);
            StringArray::from(vec![])
        }
    }
}

/// Add the given media item to the watched list.
#[no_mangle]
pub extern "C" fn add_to_watched(popcorn_fx: &mut PopcornFX, watchable: &MediaItemC) {
    match watchable.into_identifier() {
        Some(e) => {
            let id = e.imdb_id().to_string();
            match popcorn_fx.watched_service().add(e) {
                Ok(_) => info!("Media item {} as been added as seen", id),
                Err(e) => error!("Failed to add media item {} as watched, {}", id, e)
            };
        }
        None => {
            error!("Unable to add watchable, no media item given")
        }
    }
}

/// Remove the given media item from the watched list.
#[no_mangle]
pub extern "C" fn remove_from_watched(popcorn_fx: &mut PopcornFX, watchable: &MediaItemC) {
    match watchable.into_identifier() {
        Some(e) => popcorn_fx.watched_service().remove(e),
        None => {
            error!("Unable to add watchable, no media item given")
        }
    }
}

/// Register a new callback listener for watched events.
#[no_mangle]
pub extern "C" fn register_watched_event_callback<'a>(popcorn_fx: &mut PopcornFX, callback: extern "C" fn(WatchedEventC)) {
    trace!("Wrapping C callback for WatchedCallback");
    let wrapper: WatchedCallback = Box::new(move |event| {
        callback(WatchedEventC::from(event));
    });

    popcorn_fx.watched_service().register(wrapper)
}

/// The torrent wrapper for moving data between rust and java.
/// This is a temp wrapper till the torrent component is replaced.
#[no_mangle]
pub extern "C" fn torrent_wrapper(torrent: TorrentC) -> *mut TorrentWrapperC {
    trace!("Wrapping TorrentC into TorrentWrapperC");
    into_c_owned(TorrentWrapperC::from(torrent))
}

/// Inform the FX core that the state of the torrent has changed.
#[no_mangle]
pub extern "C" fn torrent_state_changed(torrent: &TorrentWrapperC, state: TorrentState) {
    torrent.state_changed(state)
}

/// Inform the FX core that a piece for the torrent has finished downloading.
#[no_mangle]
pub extern "C" fn torrent_piece_finished(torrent: &TorrentWrapperC, piece: u32) {
    torrent.piece_finished(piece)
}

/// Start a torrent stream for the given torrent.
#[no_mangle]
pub extern "C" fn start_stream(popcorn_fx: &mut PopcornFX, torrent: &'static TorrentWrapperC) -> *mut TorrentStreamC {
    trace!("Starting a new stream from C for {:?}", torrent);
    match popcorn_fx.torrent_stream_server().start_stream(Box::new(torrent) as Box<dyn Torrent>) {
        Ok(e) => {
            info!("Started new stream {}", e);
            into_c_owned(TorrentStreamC::from(e))
        }
        Err(e) => {
            error!("Failed to start stream, {}", e);
            ptr::null_mut()
        }
    }
}

/// Stop the given torrent stream.
#[no_mangle]
pub extern "C" fn stop_stream(popcorn_fx: &mut PopcornFX, stream: &mut TorrentStreamC) {
    trace!("Stopping torrent stream of {:?}", stream);
    let stream = stream.stream();
    match stream {
        None => error!("Unable to stop stream, pointer is invalid"),
        Some(stream) => {
            trace!("Stream {:?} has been read, trying to stop server", stream);
            popcorn_fx.torrent_stream_server().stop_stream(&stream);
        }
    }
}

/// Register a new callback for the torrent stream.
#[no_mangle]
pub extern "C" fn register_torrent_stream_callback(stream: &mut TorrentStreamC, callback: extern "C" fn(TorrentStreamEventC)) {
    trace!("Wrapping TorrentStreamEventC callback");
    let stream = stream.stream();
    match stream {
        None => error!("Unable to register callback, pointer is invalid"),
        Some(stream) => {
            stream.register_stream(Box::new(move |e| {
                callback(TorrentStreamEventC::from(e))
            }));
        }
    }
}

/// Retrieve the current state of the stream.
/// Use [register_torrent_stream_callback] instead if the latest up-to-date information is required.
///
/// It returns the known [TorrentStreamState] at the time of invocation.
#[no_mangle]
pub extern "C" fn torrent_stream_state(stream: &mut TorrentStreamC) -> TorrentStreamState {
    let stream = stream.stream();
    match stream {
        None => {
            error!("Unable to get stream state, pointer is invalid");
            TorrentStreamState::Stopped
        }
        Some(stream) => {
            stream.stream_state()
        }
    }
}

/// Retrieve the auto-resume timestamp for the given media id and/or filename.
#[no_mangle]
pub extern "C" fn auto_resume_timestamp(popcorn_fx: &mut PopcornFX, id: *const c_char, filename: *const c_char) -> *mut u64 {
    trace!("Retrieving auto-resume timestamp of id: {:?}, filename: {:?}", id, filename);
    let id_value: String;
    let filename_value: String;
    let id = if !id.is_null() {
        id_value = from_c_string(id);
        Some(id_value.as_str())
    } else {
        None
    };
    let filename = if !filename.is_null() {
        filename_value = from_c_string(filename);
        Some(filename_value.as_str())
    } else {
        None
    };

    match popcorn_fx.auto_resume_service().resume_timestamp(id, filename) {
        None => {
            info!("Auto-resume timestamp not found for id: {:?}, filename: {:?}", id, filename);
            ptr::null_mut()
        }
        Some(e) => {
            into_c_owned(e)
        }
    }
}

/// Handle the player stopped event.
/// The event data will be cleaned by this fn, reuse of the data is thereby not possible.
///
/// * `event`   - The C event instance of the player stopped data.
#[no_mangle]
pub extern "C" fn handle_player_stopped_event(popcorn_fx: &mut PopcornFX, event: PlayerStoppedEventC) {
    trace!("Handling the player stopped event {:?}", event);
    let event = PlayerStoppedEvent::from(&event);
    popcorn_fx.auto_resume_service().player_stopped(&event);
}

/// Resolve the given torrent url into meta information of the torrent.
/// The url can be a magnet, http or file url to the torrent file.
#[no_mangle]
pub extern "C" fn torrent_info(popcorn_fx: &mut PopcornFX, url: *const c_char) {
    let url = from_c_string(url);
    trace!("Retrieving torrent information for {}", url);
    let runtime = tokio::runtime::Runtime::new().expect("Runtime should have been created");

    runtime.block_on(async {
        match popcorn_fx.torrent_manager().info(url.as_str()).await {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to resolve torrent information, {}", e)
            }
        };
    })
}

/// Verify if the given magnet uri has already been stored.
#[no_mangle]
pub extern "C" fn torrent_collection_is_stored(popcorn_fx: &mut PopcornFX, magnet_uri: *const c_char) -> bool {
    let magnet_uri = from_c_string(magnet_uri);
    trace!("Checking if magnet uri is stored for {}", magnet_uri.as_str());
    popcorn_fx.torrent_collection().is_stored(magnet_uri.as_str())
}

/// Retrieve all stored magnets from the torrent collection.
/// It returns the set on success, else [ptr::null_mut].
#[no_mangle]
pub extern "C" fn torrent_collection_all(popcorn_fx: &mut PopcornFX) -> *mut TorrentCollectionSet {
    trace!("Retrieving torrent collection magnets");
    match popcorn_fx.torrent_collection().all() {
        Ok(e) => {
            let set = TorrentCollectionSet::from(e);
            into_c_owned(set)
        }
        Err(e) => {
            error!("Failed to retrieve magnets, {}", e);
            ptr::null_mut()
        }
    }
}

/// Add the given magnet info to the torrent collection.
#[no_mangle]
pub extern "C" fn torrent_collection_add(popcorn_fx: &mut PopcornFX, name: *const c_char, magnet_uri: *const c_char) {
    let name = from_c_string(name);
    let magnet_uri = from_c_string(magnet_uri);
    trace!("Adding magnet {} to torrent collection", magnet_uri);

    popcorn_fx.torrent_collection().insert(name.as_str(), magnet_uri.as_str());
}

/// Remove the given magnet uri from the torrent collection.
#[no_mangle]
pub extern "C" fn torrent_collection_remove(popcorn_fx: &mut PopcornFX, magnet_uri: *const c_char) {
    let magnet_uri = from_c_string(magnet_uri);
    trace!("Removing magnet {} from torrent collection", magnet_uri);

    popcorn_fx.torrent_collection().remove(magnet_uri.as_str());
}

/// Retrieve the application settings.
/// These are the setting preferences of the users for the popcorn FX instance.
#[no_mangle]
pub extern "C" fn application_settings(popcorn_fx: &mut PopcornFX) -> *mut PopcornSettingsC {
    trace!("Retrieving application settings");
    let mutex = popcorn_fx.settings();
    into_c_owned(PopcornSettingsC::from(mutex.user_settings()))
}

/// Reload the settings of the application.
#[no_mangle]
pub extern "C" fn reload_settings(popcorn_fx: &mut PopcornFX) {
    trace!("Reloading the popcorn fx settings");
    popcorn_fx.reload_settings()
}

/// Register a new callback for all setting events.
#[no_mangle]
pub extern "C" fn register_settings_callback(popcorn_fx: &mut PopcornFX, callback: ApplicationConfigCallbackC) {
    trace!("Registering application settings callback");
    let wrapper = Box::new(move |event| {
        let event_c = ApplicationConfigEventC::from(event);
        trace!("Invoking ApplicationConfigEventC {:?}", event_c);
        callback(event_c)
    });

    popcorn_fx.settings().register(wrapper);
}

/// Update the subtitle settings with the new value.
#[no_mangle]
pub extern "C" fn update_subtitle_settings(popcorn_fx: &mut PopcornFX, subtitle_settings: SubtitleSettingsC) {
    trace!("Updating the subtitle settings from {:?}", subtitle_settings);
    let subtitle = SubtitleSettings::from(subtitle_settings);
    popcorn_fx.settings().update_subtitle(subtitle);
}

/// Update the torrent settings with the new value.
#[no_mangle]
pub extern "C" fn update_torrent_settings(popcorn_fx: &mut PopcornFX, torrent_settings: TorrentSettingsC) {
    trace!("Updating the torrent settings from {:?}", torrent_settings);
    let settings = TorrentSettings::from(torrent_settings);
    popcorn_fx.settings().update_torrent(settings);
}

/// Update the ui settings with the new value.
#[no_mangle]
pub extern "C" fn update_ui_settings(popcorn_fx: &mut PopcornFX, settings: UiSettingsC) {
    trace!("Updating the ui settings from {:?}", settings);
    let settings = UiSettings::from(settings);
    popcorn_fx.settings().update_ui(settings);
}

/// Update the server settings with the new value.
#[no_mangle]
pub extern "C" fn update_server_settings(popcorn_fx: &mut PopcornFX, settings: ServerSettingsC) {
    trace!("Updating the server settings from {:?}", settings);
    let settings = ServerSettings::from(settings);
    popcorn_fx.settings().update_server(settings);
}

/// Update the playback settings with the new value.
#[no_mangle]
pub extern "C" fn update_playback_settings(popcorn_fx: &mut PopcornFX, settings: PlaybackSettingsC) {
    trace!("Updating the playback settings from {:?}", settings);
    let settings = PlaybackSettings::from(settings);
    popcorn_fx.settings().update_playback(settings);
}

/// Dispose the given media item from memory.
#[no_mangle]
pub extern "C" fn dispose_media_item(media: Box<MediaItemC>) {
    if !media.show_overview.is_null() {
        let _ = from_c_owned(media.show_overview);
    } else if !media.show_details.is_null() {
        let _ = from_c_owned(media.show_details);
    } else if !media.movie_overview.is_null() {
        let _ = from_c_owned(media.movie_overview);
    } else if !media.movie_details.is_null() {
        let _ = from_c_owned(media.movie_details);
    }
}

/// Dispose all given media items from memory.
#[no_mangle]
pub extern "C" fn dispose_media_items(media: Box<MediaSetC>) {
    trace!("Disposing media items of {:?}", media);
    let _ = media.movies();
    let _ = media.shows();
}

/// Dispose the torrent stream.
/// Make sure [stop_stream] has been called before dropping the instance.
#[no_mangle]
pub extern "C" fn dispose_torrent_stream(stream: Box<TorrentStreamC>) {
    trace!("Disposing stream {:?}", stream)
}

/// Dispose the given subtitle.
#[no_mangle]
pub extern "C" fn dispose_subtitle(subtitle: Box<SubtitleC>) {
    trace!("Disposing subtitle {:?}", subtitle);
    let _ = Subtitle::from(*subtitle);
}

/// Dispose the [TorrentCollectionSet] from memory.
#[no_mangle]
pub extern "C" fn dispose_torrent_collection(collection_set: Box<TorrentCollectionSet>) {
    trace!("Disposing collection set {:?}", collection_set)
}

#[cfg(test)]
mod test {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use tempfile::tempdir;

    use popcorn_fx_core::core::config::{DecorationType, SubtitleFamily};
    use popcorn_fx_core::core::subtitles::cue::{StyledText, SubtitleCue, SubtitleLine};
    use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;
    use popcorn_fx_core::core::torrent::{TorrentEvent, TorrentState};
    use popcorn_fx_core::from_c_owned;
    use popcorn_fx_core::testing::{copy_test_file, init_logger};

    use crate::fx::PopcornFxArgs;

    use super::*;

    #[no_mangle]
    pub extern "C" fn has_bytes_callback(_: i32, _: *mut u64) -> bool {
        true
    }

    #[no_mangle]
    pub extern "C" fn has_piece_callback(_: u32) -> bool {
        true
    }

    #[no_mangle]
    pub extern "C" fn total_pieces_callback() -> i32 {
        10
    }

    #[no_mangle]
    pub extern "C" fn prioritize_bytes_callback(_: i32, _: *mut u64) {}

    #[no_mangle]
    pub extern "C" fn prioritize_pieces_callback(_: i32, _: *mut u32) {}

    #[no_mangle]
    pub extern "C" fn sequential_mode_callback() {}

    #[no_mangle]
    pub extern "C" fn torrent_state_callback() -> TorrentState {
        TorrentState::Downloading
    }

    #[no_mangle]
    pub extern "C" fn settings_callback(_: ApplicationConfigEventC) {}

    #[test]
    fn test_dispose_popcorn_fx() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_youtube_video_player: false,
            disable_fx_video_player: false,
            disable_vlc_video_player: false,
            tv: false,
            maximized: false,
            insecure: false,
            app_directory: temp_path.to_string(),
        });

        dispose_popcorn_fx(Box::new(instance));
    }

    #[test]
    fn test_is_liked_movie_overview() {
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_youtube_video_player: false,
            disable_fx_video_player: false,
            disable_vlc_video_player: false,
            tv: false,
            maximized: false,
            insecure: false,
            app_directory: temp_path.to_string(),
        });
        let movie = MovieOverview::new(
            "".to_string(),
            "tt0000000122".to_string(),
            "2020".to_string(),
        );
        let media = MediaItemC::from(movie);

        let result = is_media_liked(&mut instance, &media);

        assert_eq!(false, result)
    }

    #[test]
    fn test_is_liked_movie_details() {
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_youtube_video_player: false,
            disable_fx_video_player: false,
            disable_vlc_video_player: false,
            tv: false,
            maximized: false,
            insecure: false,
            app_directory: temp_path.to_string(),
        });
        let movie = MovieDetails::new(
            "".to_string(),
            "tt0000000111".to_string(),
            "2020".to_string(),
        );
        let media = MediaItemC::from(movie);

        let result = is_media_liked(&mut instance, &media);

        assert_eq!(false, result)
    }

    #[test]
    fn test_auto_resume_timestamp() {
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_youtube_video_player: false,
            disable_fx_video_player: false,
            disable_vlc_video_player: false,
            tv: false,
            maximized: false,
            insecure: false,
            app_directory: temp_path.to_string(),
        });
        let id = "tt0000001111".to_string();
        let filename = "lorem-ipsum-dolor-estla.mkv".to_string();

        let result = auto_resume_timestamp(&mut instance, into_c_string(id), into_c_string(filename));

        assert_eq!(ptr::null_mut(), result)
    }

    #[test]
    fn test_update_subtitle() {
        let language1 = SubtitleLanguage::Finnish;
        let subtitle1 = SubtitleInfo::new(
            "tt212121".to_string(),
            language1.clone(),
        );
        let info_c1 = SubtitleInfoC::from(subtitle1.clone());
        let language2 = SubtitleLanguage::English;
        let subtitle2 = SubtitleInfo::new(
            "tt212333".to_string(),
            language2.clone(),
        );
        let info_c2 = SubtitleInfoC::from(subtitle2.clone());
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_youtube_video_player: false,
            disable_fx_video_player: false,
            disable_vlc_video_player: false,
            tv: false,
            maximized: false,
            insecure: false,
            app_directory: temp_path.to_string(),
        });

        update_subtitle(&mut instance, &info_c1);
        let info_result = SubtitleInfo::from(&from_c_owned(retrieve_preferred_subtitle(&mut instance)));
        let language_result = retrieve_preferred_subtitle_language(&mut instance);
        assert_eq!(subtitle1, info_result);
        assert_eq!(language1, language_result);

        update_subtitle(&mut instance, &info_c2);
        let info_result = SubtitleInfo::from(&from_c_owned(retrieve_preferred_subtitle(&mut instance)));
        let language_result = retrieve_preferred_subtitle_language(&mut instance);
        assert_eq!(subtitle2, info_result);
        assert_eq!(language2, language_result);

        reset_subtitle(&mut instance);
        let preferred_result = retrieve_preferred_subtitle_language(&mut instance);
        assert_eq!(SubtitleLanguage::None, preferred_result);
    }

    #[test]
    fn test_reset_movie_apis() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_youtube_video_player: false,
            disable_fx_video_player: false,
            disable_vlc_video_player: false,
            tv: false,
            maximized: false,
            insecure: false,
            app_directory: temp_path.to_string(),
        });

        reset_movie_apis(&mut instance);
    }

    #[test]
    fn test_torrent_state_changed() {
        let torrent = TorrentC {
            filepath: into_c_string("lorem.txt".to_string()),
            has_byte_callback: has_bytes_callback,
            has_piece_callback: has_piece_callback,
            total_pieces: total_pieces_callback,
            prioritize_bytes: prioritize_bytes_callback,
            prioritize_pieces: prioritize_pieces_callback,
            sequential_mode: sequential_mode_callback,
            torrent_state: torrent_state_callback,
        };
        let (tx, rx) = channel();

        let wrapper = from_c_owned(torrent_wrapper(torrent));
        wrapper.wrapper().register(Box::new(move |e| {
            tx.send(e).unwrap()
        }));
        torrent_state_changed(&wrapper, TorrentState::Starting);

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();

        match result {
            TorrentEvent::StateChanged(state) => assert_eq!(TorrentState::Starting, state),
            _ => {}
        }
    }

    #[test]
    fn test_disable_subtitle() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_youtube_video_player: false,
            disable_fx_video_player: false,
            disable_vlc_video_player: false,
            tv: false,
            maximized: false,
            insecure: false,
            app_directory: temp_path.to_string(),
        });

        let disabled = is_subtitle_disabled(&mut instance);
        assert!(!disabled, "expected the subtitle track to be enabled by default");

        disable_subtitle(&mut instance);
        let result = is_subtitle_disabled(&mut instance);

        assert!(result, "expected the subtitle track to be disabled")
    }

    #[test]
    fn test_torrent_collection_is_stored() {
        let magnet_uri = "magnet:?MagnetA";
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_youtube_video_player: false,
            disable_fx_video_player: false,
            disable_vlc_video_player: false,
            tv: false,
            maximized: false,
            insecure: false,
            app_directory: temp_path.to_string(),
        });
        copy_test_file(temp_path, "torrent-collection.json", None);

        let result = torrent_collection_is_stored(&mut instance, into_c_string(magnet_uri.to_string()));

        assert_eq!(true, result)
    }

    #[test]
    fn test_torrent_collection_all() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_youtube_video_player: false,
            disable_fx_video_player: false,
            disable_vlc_video_player: false,
            tv: false,
            maximized: false,
            insecure: false,
            app_directory: temp_path.to_string(),
        });
        copy_test_file(temp_path, "torrent-collection.json", None);

        let result = from_c_owned(torrent_collection_all(&mut instance));

        assert_eq!(1, result.len)
    }

    #[test]
    fn test_register_settings_callback() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let subtitle_c = SubtitleSettingsC::from(&SubtitleSettings::new(
            Some(temp_path.to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        ));
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_youtube_video_player: false,
            disable_fx_video_player: false,
            disable_vlc_video_player: false,
            tv: false,
            maximized: false,
            insecure: false,
            app_directory: temp_path.to_string(),
        });

        register_settings_callback(&mut instance, settings_callback);
        update_subtitle_settings(&mut instance, subtitle_c);
    }

    #[test]
    fn test_update_subtitle_settings() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_youtube_video_player: false,
            disable_fx_video_player: false,
            disable_vlc_video_player: false,
            tv: false,
            maximized: false,
            insecure: false,
            app_directory: temp_path.to_string(),
        });
        let settings = SubtitleSettings {
            directory: format!("{}/subtitles", temp_path),
            auto_cleaning_enabled: false,
            default_subtitle: SubtitleLanguage::German,
            font_family: SubtitleFamily::Arial,
            font_size: 32,
            decoration: DecorationType::SeeThroughBackground,
            bold: true,
        };

        update_subtitle_settings(&mut instance, SubtitleSettingsC::from(&settings));
        let mutex = instance.settings();
        let result = mutex.user_settings().subtitle();

        assert_eq!(&settings, result)
    }

    #[test]
    fn test_dispose_media_item() {
        let movie = MovieOverview::new(
            String::new(),
            String::from("tt54698542"),
            String::new(),
        );
        let media = MediaItemC::from(movie);

        dispose_media_item(Box::new(media));
    }

    #[test]
    fn test_dispose_media_items() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_youtube_video_player: false,
            disable_fx_video_player: false,
            disable_vlc_video_player: false,
            tv: false,
            maximized: false,
            insecure: false,
            app_directory: temp_path.to_string(),
        });
        let genre = GenreC::from(Genre::all());
        let sort_by = SortByC::from(SortBy::new("trending".to_string(), String::new()));
        let keywords = into_c_string(String::new());

        let media_items = retrieve_available_movies(&mut instance, &genre, &sort_by, keywords, 1);

        dispose_media_items(Box::new(from_c_owned(media_items)))
    }

    #[test]
    fn test_dispose_subtitle() {
        let subtitle = Subtitle::new(
            vec![SubtitleCue::new(
                "012".to_string(),
                10000,
                20000,
                vec![SubtitleLine::new(
                    vec![StyledText::new(
                        "Lorem ipsum dolor".to_string(),
                        true,
                        false,
                        false,
                    )]
                )],
            )],
            Some(SubtitleInfo::new("tt00001".to_string(), SubtitleLanguage::English)),
            "lorem.srt".to_string(),
        );
        let subtitle_c = SubtitleC::from(subtitle);

        dispose_subtitle(Box::new(subtitle_c))
    }
}
