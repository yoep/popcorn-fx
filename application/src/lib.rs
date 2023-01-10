extern crate core;

use std::{ptr, slice};
use std::os::raw::c_char;
use std::path::Path;

use log::{debug, error, info, trace};

use popcorn_fx_core::{EpisodeC, from_c_string, GenreC, into_c_owned, MovieC, ShowC, SortByC, SubtitleC, SubtitleInfoC, SubtitleMatcherC, to_c_string, VecMovieC, VecShowC, VecSubtitleInfoC};
use popcorn_fx_core::core::media::{Category, MovieDetails, ShowDetails};
use popcorn_fx_core::core::subtitles::model::{SubtitleInfo, SubtitleType};
use popcorn_fx_platform::PlatformInfoC;

use crate::popcorn::fx::popcorn_fx::PopcornFX;

pub mod popcorn;

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

/// Retrieve the available subtitles for the given [MovieC].
///
/// It returns a reference to [VecSubtitleInfoC], else a [ptr::null_mut] on failure.
/// <i>The returned reference should be managed by the caller.</i>
#[no_mangle]
pub extern "C" fn movie_subtitles(popcorn_fx: &mut PopcornFX, movie: &MovieC) -> *mut VecSubtitleInfoC {
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
pub extern "C" fn episode_subtitles(popcorn_fx: &mut PopcornFX, show: &ShowC, episode: &EpisodeC) -> *mut VecSubtitleInfoC {
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
    let subtitles: Vec<SubtitleInfo> = c_vec.into_iter()
        .map(|e| e.to_subtitle())
        .collect();

    into_c_owned(SubtitleInfoC::from(popcorn_fx.subtitle_service().select_or_default(&subtitles)))
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
pub extern "C" fn retrieve_available_movies(popcorn_fx: &mut PopcornFX, genre: &GenreC, sort_by: &SortByC, keywords: *const c_char, page: u32) -> *mut VecMovieC {
    let genre = genre.to_struct();
    let sort_by = sort_by.to_struct();
    let keywords = from_c_string(keywords);
    let runtime = tokio::runtime::Runtime::new().expect("expected a runtime to have been created");

    return match popcorn_fx.providers().get(Category::MOVIES) {
        None => {
            error!("No provider could be found for {}", Category::MOVIES);
            ptr::null_mut()
        }
        Some(e) => {
            match runtime.block_on(e.retrieve(&genre, &sort_by, &keywords, page)) {
                Ok(e) => {
                    info!("Retrieved a total of {} movies, {}", e.size(), &e);
                    let movies: Vec<MovieC> = e.into_content().into_iter()
                        .map(|e| e
                            .into_any()
                            .downcast::<MovieDetails>()
                            .expect("expected media to be a movie"))
                        .map(|e| MovieC::from(*e))
                        .collect();

                    if movies.len() > 0 {
                        into_c_owned(VecMovieC::from(movies))
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
    };
}

/// Retrieve the details of a given movie.
/// It will query the api for the given IMDB ID.
///
/// It returns the [MovieC] on success, else [ptr::null_mut].
#[no_mangle]
pub extern "C" fn retrieve_movie_details(popcorn_fx: &mut PopcornFX, imdb_id: *const c_char) -> *mut MovieC {
    let imdb_id = from_c_string(imdb_id);
    let runtime = tokio::runtime::Runtime::new().expect("expected a runtime to have been created");

    return match popcorn_fx.providers().get(Category::MOVIES) {
        None => {
            error!("No provider could be found for {}", Category::MOVIES);
            ptr::null_mut()
        }
        Some(provider) => {
            match runtime.block_on(provider.retrieve_details(&imdb_id)) {
                Ok(e) => {
                    trace!("Returning movie details {:?}", &e);
                    into_c_owned(MovieC::from(*e
                        .into_any()
                        .downcast::<MovieDetails>()
                        .expect("expected media to be a movie")))
                }
                Err(e) => {
                    error!("Failed to retrieve movie details, {}", e);
                    ptr::null_mut()
                }
            }
        }
    };
}

/// Reset all available api stats for the movie api.
/// This will make all disabled api's available again.
#[no_mangle]
pub extern "C" fn reset_movie_apis(popcorn_fx: &mut PopcornFX) {
    return match popcorn_fx.providers().get(Category::MOVIES) {
        None => error!("No provider could be found for {}", Category::MOVIES),
        Some(provider) => provider.reset_api()
    };
}

/// Retrieve the available [ShowC] items for the given criteria.
///
/// It returns an array of [ShowC] items on success, else a [ptr::null_mut].
#[no_mangle]
pub extern "C" fn retrieve_available_shows(popcorn_fx: &mut PopcornFX, genre: &GenreC, sort_by: &SortByC, keywords: *const c_char, page: u32) -> *mut VecShowC {
    let genre = genre.to_struct();
    let sort_by = sort_by.to_struct();
    let keywords = from_c_string(keywords);
    let runtime = tokio::runtime::Runtime::new().expect("expected a runtime to have been created");

    return match popcorn_fx.providers().get(Category::SERIES) {
        None => {
            error!("No provider could be found for {}", Category::SERIES);
            ptr::null_mut()
        }
        Some(e) => {
            match runtime.block_on(e.retrieve(&genre, &sort_by, &keywords, page)) {
                Ok(e) => {
                    info!("Retrieved a total of {} shows, {}", e.size(), &e);
                    let shows: Vec<ShowC> = e.into_content().into_iter()
                        .map(|e| e
                            .into_any()
                            .downcast::<ShowDetails>()
                            .expect("expected media to be a show"))
                        .map(|e| ShowC::from(*e))
                        .collect();

                    if shows.len() > 0 {
                        into_c_owned(VecShowC::from(shows))
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
    };
}

#[no_mangle]
pub extern "C" fn retrieve_show_details(popcorn_fx: &mut PopcornFX, imdb_id: *const c_char) -> *mut ShowC {
    let imdb_id = from_c_string(imdb_id);
    let runtime = tokio::runtime::Runtime::new().expect("expected a runtime to have been created");

    return match popcorn_fx.providers().get(Category::SERIES) {
        None => {
            error!("No provider could be found for {}", Category::SERIES);
            ptr::null_mut()
        }
        Some(provider) => {
            match runtime.block_on(provider.retrieve_details(&imdb_id)) {
                Ok(e) => {
                    trace!("Returning show details {:?}", &e);
                    into_c_owned(ShowC::from(*e
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
    };
}

/// Reset all available api stats for the movie api.
/// This will make all disabled api's available again.
#[no_mangle]
pub extern "C" fn reset_show_apis(popcorn_fx: &mut PopcornFX) {
    return match popcorn_fx.providers().get(Category::SERIES) {
        None => error!("No provider could be found for {}", Category::SERIES),
        Some(provider) => provider.reset_api()
    };
}

/// Delete the PopcornFX instance in a safe way.
#[no_mangle]
pub extern "C" fn dispose_popcorn_fx(popcorn_fx: Box<PopcornFX>) {
    info!("Disposing Popcorn FX");
    popcorn_fx.dispose();
    drop(popcorn_fx)
}
