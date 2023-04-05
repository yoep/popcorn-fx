use std::os::raw::c_char;

use log::{debug, error, info, trace};

use popcorn_fx_core::core::media::{Category, MovieOverview, ShowOverview};
use popcorn_fx_core::from_c_string;

use crate::ffi::{GenreC, MediaErrorC, MediaSetC, MediaSetResult, SortByC};
use crate::PopcornFX;

/// Retrieve the available movies for the given criteria.
///
/// It returns the [VecMovieC] reference on success, else [ptr::null_mut].
#[no_mangle]
pub extern "C" fn retrieve_available_movies(popcorn_fx: &mut PopcornFX, genre: &GenreC, sort_by: &SortByC, keywords: *const c_char, page: u32) -> MediaSetResult {
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
                MediaSetResult::Ok(MediaSetC::from_movies(movies))
            } else {
                debug!("No movies have been found, returning ptr::null");
                MediaSetResult::Err(MediaErrorC::NoItemsFound)
            }
        }
        Err(e) => {
            error!("Failed to retrieve movies, {}", e);
            MediaSetResult::from(e)
        }
    }
}

/// Retrieve the available [ShowOverviewC] items for the given criteria.
///
/// It returns an array of [ShowOverviewC] items on success, else a [ptr::null_mut].
#[no_mangle]
pub extern "C" fn retrieve_available_shows(popcorn_fx: &mut PopcornFX, genre: &GenreC, sort_by: &SortByC, keywords: *const c_char, page: u32) -> MediaSetResult {
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
                MediaSetResult::Ok(MediaSetC::from_shows(shows))
            } else {
                debug!("No shows have been found, returning ptr::null");
                MediaSetResult::Err(MediaErrorC::NoItemsFound)
            }
        }
        Err(e) => {
            error!("Failed to retrieve movies, {}", e);
            MediaSetResult::from(e)
        }
    }
}

/// Reset all available api stats for the movie api.
/// This will make all disabled api's available again.
#[no_mangle]
pub extern "C" fn reset_movie_apis(popcorn_fx: &mut PopcornFX) {
    trace!("Resetting the movie api providers from C");
    popcorn_fx.providers().reset_api(&Category::Movies)
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use futures::future::err;
    use tempfile::tempdir;

    use popcorn_fx_core::{from_c_owned, into_c_string};
    use popcorn_fx_core::core::media::{Genre, SortBy};
    use popcorn_fx_core::testing::init_logger;

    use crate::test::default_args;

    use super::*;

    #[test]
    fn test_retrieve_available_movies() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let genre = GenreC::from(Genre::all());
        let sort_by = SortByC::from(SortBy::new(String::from("trending"), String::new()));
        let mut instance = PopcornFX::new(default_args(temp_path));

        let result = retrieve_available_movies(&mut instance, &genre, &sort_by, into_c_string("".to_string()), 1);

        match result {
            MediaSetResult::Ok(set) => {}
            _ => panic!("Expected MediaSetResult::Ok")
        }
    }

    #[test]
    fn test_retrieve_available_movies_error() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let genre = GenreC::from(Genre::all());
        let sort_by = SortByC::from(SortBy::new(String::from("trending"), String::new()));
        let mut popcorn_fx_args = default_args(temp_path);
        popcorn_fx_args.properties.providers = HashMap::new();
        let mut instance = PopcornFX::new(popcorn_fx_args);

        let result = retrieve_available_movies(&mut instance, &genre, &sort_by, into_c_string("".to_string()), 1);

        match result {
            MediaSetResult::Err(error) => assert_eq!(MediaErrorC::NoAvailableProviders, error),
            _ => panic!("Expected MediaSetResult::Err")
        }
    }

    #[test]
    fn test_reset_movie_apis() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));

        reset_movie_apis(&mut instance);
    }

    #[test]
    fn test_retrieve_available_shows() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let genre = GenreC::from(Genre::all());
        let sort_by = SortByC::from(SortBy::new(String::from("trending"), String::new()));
        let mut instance = PopcornFX::new(default_args(temp_path));

        let result = retrieve_available_shows(&mut instance, &genre, &sort_by, into_c_string("".to_string()), 1);

        match result {
            MediaSetResult::Ok(set) => {}
            _ => panic!("Expected MediaSetResult::Ok")
        }
    }

    #[test]
    fn test_retrieve_available_shows_error() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let genre = GenreC::from(Genre::all());
        let sort_by = SortByC::from(SortBy::new(String::from("trending"), String::new()));
        let mut popcorn_fx_args = default_args(temp_path);
        popcorn_fx_args.properties.providers = HashMap::new();
        let mut instance = PopcornFX::new(popcorn_fx_args);

        let result = retrieve_available_shows(&mut instance, &genre, &sort_by, into_c_string("".to_string()), 1);

        match result {
            MediaSetResult::Err(error) => assert_eq!(MediaErrorC::NoAvailableProviders, error),
            _ => panic!("Expected MediaSetResult::Err")
        }
    }
}