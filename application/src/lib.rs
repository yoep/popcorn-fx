extern crate core;

use std::{mem, ptr, slice};
use std::os::raw::c_char;
use std::path::Path;

use log::{debug, error, info, trace, warn};

use media_mappers::*;
use popcorn_fx_core::{EpisodeC, FavoriteEventC, from_c_into_boxed, from_c_owned, from_c_string, GenreC, into_c_owned, MediaItemC, MediaSetC, MovieDetailsC, ShowDetailsC, SortByC, SubtitleC, SubtitleInfoC, SubtitleMatcherC, to_c_string, to_c_vec, VecFavoritesC, VecSubtitleInfoC, WatchedEventC};
use popcorn_fx_core::core::media::*;
use popcorn_fx_core::core::media::favorites::FavoriteCallback;
use popcorn_fx_core::core::media::watched::WatchedCallback;
use popcorn_fx_core::core::subtitles::model::{SubtitleInfo, SubtitleType};
use popcorn_fx_platform::PlatformInfoC;

use crate::arrays::StringArray;
use crate::popcorn::fx::popcorn_fx::PopcornFX;

pub mod popcorn;
mod arrays;
mod media_mappers;

/// Create a new PopcornFX instance.
/// The caller will become responsible for managing the memory of the struct.
/// The instance can be safely deleted by using [delete_popcorn_fx].
#[no_mangle]
pub extern "C" fn new_popcorn_fx() -> *mut PopcornFX {
    let instance = PopcornFX::new();

    info!("Created new Popcorn FX instance");
    into_c_owned(instance)
}

/// Enable the screensaver on the current platform
#[no_mangle]
pub extern "C" fn enable_screensaver(popcorn_fx: &mut PopcornFX) {
    popcorn_fx.platform_service().enable_screensaver();
}

/// Disable the screensaver on the current platform
#[no_mangle]
pub extern "C" fn disable_screensaver(popcorn_fx: &mut PopcornFX) {
    popcorn_fx.platform_service().disable_screensaver();
}

/// Retrieve the platform information
#[no_mangle]
pub extern "C" fn platform_info(popcorn_fx: &mut PopcornFX) -> *mut PlatformInfoC {
    into_c_owned(PlatformInfoC::from(popcorn_fx.platform_service().platform_info()))
}

/// Retrieve the default options available for the subtitles.
#[no_mangle]
pub extern "C" fn default_subtitle_options(popcorn_fx: &mut PopcornFX) -> *mut VecSubtitleInfoC {
    into_c_owned(VecSubtitleInfoC::from(popcorn_fx.subtitle_service().default_subtitle_options().into_iter()
        .map(|e| SubtitleInfoC::from(e))
        .collect()))
}

/// Retrieve the available subtitles for the given [MovieDetailsC].
///
/// It returns a reference to [VecSubtitleInfoC], else a [ptr::null_mut] on failure.
/// <i>The returned reference should be managed by the caller.</i>
#[no_mangle]
pub extern "C" fn movie_subtitles(popcorn_fx: &mut PopcornFX, movie: &MovieDetailsC) -> *mut VecSubtitleInfoC {
    let runtime = tokio::runtime::Runtime::new().expect("Runtime should have been created");
    let movie_instance = movie.to_struct();

    match runtime.block_on(popcorn_fx.subtitle_service().movie_subtitles(movie_instance)) {
        Ok(e) => {
            debug!("Found movie subtitles {:?}", e);
            let result: Vec<SubtitleInfoC> = e.into_iter()
                .map(|e| SubtitleInfoC::from(e))
                .collect();

            into_c_owned(VecSubtitleInfoC::from(result))
        }
        Err(e) => {
            error!("Movie subtitle search failed, {}", e);
            ptr::null_mut()
        }
    }
}

/// Retrieve the given subtitles for the given episode
#[no_mangle]
pub extern "C" fn episode_subtitles(popcorn_fx: &mut PopcornFX, show: &ShowDetailsC, episode: &EpisodeC) -> *mut VecSubtitleInfoC {
    let runtime = tokio::runtime::Runtime::new().expect("expected a runtime to have been created");
    let show_instance = show.to_struct();
    let episode_instance = episode.to_struct();

    match runtime.block_on(popcorn_fx.subtitle_service().episode_subtitles(show_instance, episode_instance)) {
        Ok(e) => {
            debug!("Found episode subtitles {:?}", e);
            let result: Vec<SubtitleInfoC> = e.into_iter()
                .map(|e| SubtitleInfoC::from(e))
                .collect();

            into_c_owned(VecSubtitleInfoC::from(result))
        }
        Err(e) => {
            error!("Episode subtitle search failed, {}", e);
            into_c_owned(VecSubtitleInfoC::from(vec![]))
        }
    }
}

/// Retrieve the available subtitles for the given filename
#[no_mangle]
pub extern "C" fn filename_subtitles(popcorn_fx: &mut PopcornFX, filename: *mut c_char) -> *mut VecSubtitleInfoC {
    let filename_rust = from_c_string(filename);
    let runtime = tokio::runtime::Runtime::new().expect("expected a runtime to have been created");

    match runtime.block_on(popcorn_fx.subtitle_service().file_subtitles(&filename_rust)) {
        Ok(e) => {
            debug!("Found filename subtitles {:?}", e);
            let result: Vec<SubtitleInfoC> = e.into_iter()
                .map(|e| SubtitleInfoC::from(e))
                .collect();

            into_c_owned(VecSubtitleInfoC::from(result))
        }
        Err(e) => {
            error!("Filename subtitle search failed, {}", e);
            into_c_owned(VecSubtitleInfoC::from(vec![]))
        }
    }
}

/// Select a default subtitle language based on the settings or user interface language.
#[no_mangle]
pub extern "C" fn select_or_default_subtitle(popcorn_fx: &mut PopcornFX, subtitles_ptr: *const SubtitleInfoC, len: usize) -> *mut SubtitleInfoC {
    let c_vec = unsafe { slice::from_raw_parts(subtitles_ptr, len).to_vec() };
    let subtitles: Vec<SubtitleInfo> = c_vec.iter()
        .map(|e| e.to_subtitle())
        .collect();

    let subtitle = into_c_owned(SubtitleInfoC::from(popcorn_fx.subtitle_service().select_or_default(&subtitles)));

    // make sure rust doesn't start cleaning the subtitles as they might be switched later on
    // the pointer can also not be cleaned
    let (vec, _) = to_c_vec(subtitles.into_iter()
        .map(|e| SubtitleInfoC::from(e))
        .collect());
    mem::forget(vec);
    mem::forget(subtitles_ptr);

    subtitle
}

/// Download and parse the given subtitle info.
///
/// It returns the [SubtitleC] reference on success, else [ptr::null_mut].
#[no_mangle]
pub extern "C" fn download_subtitle(popcorn_fx: &mut PopcornFX, subtitle: &SubtitleInfoC, matcher: &SubtitleMatcherC) -> *mut SubtitleC {
    let subtitle_info = subtitle.clone().to_subtitle();
    let matcher = matcher.to_matcher();
    let runtime = tokio::runtime::Runtime::new().expect("expected a runtime to have been created");

    match runtime.block_on(popcorn_fx.subtitle_service().download(&subtitle_info, &matcher)) {
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

/// Parse the given subtitle file.
///
/// It returns the parsed subtitle on success, else null.
#[no_mangle]
pub extern "C" fn parse_subtitle(popcorn_fx: &mut PopcornFX, file_path: *const c_char) -> *mut SubtitleC {
    let string_path = from_c_string(file_path);
    let path = Path::new(&string_path);

    match popcorn_fx.subtitle_service().parse(path) {
        Ok(e) => {
            debug!("Parsed subtitle file, {}", e);
            into_c_owned(SubtitleC::from(e))
        }
        Err(e) => {
            error!("File parsing failed, {}", e);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn subtitle_to_raw(popcorn_fx: &mut PopcornFX, subtitle: &SubtitleC, output_type: usize) -> *const c_char {
    debug!("Converting to raw subtitle type {} for {:?}", output_type, subtitle);
    let subtitle = subtitle.to_subtitle();
    let subtitle_type = SubtitleType::from_ordinal(output_type);

    match popcorn_fx.subtitle_service().convert(subtitle, subtitle_type.clone()) {
        Ok(e) => {
            debug!("Returning subtitle format {} to C", subtitle_type);
            to_c_string(e)
        }
        Err(e) => {
            error!("Failed to convert subtitle to {}, {}", subtitle_type, e);
            ptr::null()
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
    let runtime = tokio::runtime::Runtime::new().expect("expected a runtime to have been created");

    match runtime.block_on(popcorn_fx.providers().retrieve(&Category::MOVIES, &genre, &sort_by, &keywords, page)) {
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
    let runtime = tokio::runtime::Runtime::new().expect("expected a runtime to have been created");

    match runtime.block_on(popcorn_fx.providers().retrieve_details(&Category::MOVIES, &imdb_id)) {
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
    popcorn_fx.providers().reset_api(&Category::MOVIES)
}

/// Retrieve the available [ShowOverviewC] items for the given criteria.
///
/// It returns an array of [ShowOverviewC] items on success, else a [ptr::null_mut].
#[no_mangle]
pub extern "C" fn retrieve_available_shows(popcorn_fx: &mut PopcornFX, genre: &GenreC, sort_by: &SortByC, keywords: *const c_char, page: u32) -> *mut MediaSetC {
    let genre = genre.to_struct();
    let sort_by = sort_by.to_struct();
    let keywords = from_c_string(keywords);
    let runtime = tokio::runtime::Runtime::new().expect("expected a runtime to have been created");

    match runtime.block_on(popcorn_fx.providers().retrieve(&Category::SERIES, &genre, &sort_by, &keywords, page)) {
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
    let runtime = tokio::runtime::Runtime::new().expect("expected a runtime to have been created");

    match runtime.block_on(popcorn_fx.providers().retrieve_details(&Category::SERIES, &imdb_id)) {
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
    popcorn_fx.providers().reset_api(&Category::SERIES)
}

/// Retrieve all liked favorite media items.
///
/// It returns the [VecFavoritesC] holder for the array on success, else [ptr::null_mut].
#[no_mangle]
pub extern "C" fn retrieve_available_favorites(popcorn_fx: &mut PopcornFX, genre: &GenreC, sort_by: &SortByC, keywords: *const c_char, page: u32) -> *mut VecFavoritesC {
    let genre = genre.to_struct();
    let sort_by = sort_by.to_struct();
    let keywords = from_c_string(keywords);
    let runtime = tokio::runtime::Runtime::new().expect("expected a runtime to have been created");

    match runtime.block_on(popcorn_fx.providers().retrieve(&Category::FAVORITES, &genre, &sort_by, &keywords, page)) {
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
    let runtime = tokio::runtime::Runtime::new().expect("expected a runtime to have been created");

    match runtime.block_on(popcorn_fx.providers().retrieve_details(&Category::FAVORITES, &imdb_id)) {
        Ok(e) => {
            trace!("Returning favorite details {:?}", &e);
            match e.media_type() {
                MediaType::Movie => {
                    into_c_owned(MediaItemC::from_movie_details(*e.into_any()
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
    match favorite.to_identifier() {
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
        let details = boxed.to_struct();
        media = Box::new(details.to_overview());
        trace!("Created media struct {:?}", media);
        mem::forget(details);
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
    match favorite.to_identifier() {
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
    let subtitle = subtitle.to_subtitle();
    let subtitle_type = SubtitleType::from_ordinal(output_type);

    match popcorn_fx.subtitle_server().serve(subtitle, subtitle_type) {
        Ok(e) => {
            info!("Serving subtitle at {}", &e);
            to_c_string(e)
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
    match watchable.to_identifier() {
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
    match watchable.to_identifier() {
        Some(e) => {
            let id = e.imdb_id();
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
    match watchable.to_identifier() {
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

/// Dispose the given media item from memory.
#[no_mangle]
pub extern "C" fn dispose_media_item(media: Box<MediaItemC>) {
    if !media.show_overview.is_null() {
        let _ = from_c_owned(media.show_overview).to_struct();
    } else if !media.movie_overview.is_null() {
        let _ = from_c_owned(media.movie_overview).to_struct();
    }
}

/// Dispose all given media items from memory.
#[no_mangle]
pub extern "C" fn dispose_media_items(media: Box<MediaSetC>) {
    trace!("Disposing media items of {:?}", media);
    let _ = media.movies();
    let _ = media.shows();
}

/// Delete the PopcornFX instance in a safe way.
#[no_mangle]
pub extern "C" fn dispose_popcorn_fx(popcorn_fx: Box<PopcornFX>) {
    info!("Disposing Popcorn FX");
    popcorn_fx.dispose();
    drop(popcorn_fx)
}

#[cfg(test)]
mod test {
    use popcorn_fx_core::from_c_owned;

    use super::*;

    #[test]
    fn test_create_and_dispose_popcorn_fx() {
        let instance = from_c_owned(new_popcorn_fx());

        dispose_popcorn_fx(Box::new(instance));
    }

    #[test]
    fn test_dispose_media_item() {
        let movie = MovieOverview::new(
            String::new(),
            String::from("tt54698542"),
            String::new(),
        );
        let media = MediaItemC::from_movie(movie);

        dispose_media_item(Box::new(media));
    }

    #[test]
    fn test_dispose_media_items() {
        let mut instance = from_c_owned(new_popcorn_fx());
        let genre = GenreC::from(Genre::all());
        let sort_by = SortByC::from(SortBy::new("trending".to_string(), String::new()));
        let keywords = to_c_string(String::new());

        let media_items = retrieve_available_movies(&mut instance, &genre, &sort_by, keywords, 1);

        dispose_media_items(Box::new(from_c_owned(media_items)))
    }
}
