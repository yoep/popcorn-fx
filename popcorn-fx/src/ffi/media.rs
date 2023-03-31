use std::os::raw::c_char;
use std::ptr;

use log::{debug, error, info};

use popcorn_fx_core::{from_c_string, into_c_owned};
use popcorn_fx_core::core::media::{Category, MovieOverview};

use crate::ffi::{GenreC, MediaErrorC, MediaSetC, MovieOverviewC, SortByC};
use crate::PopcornFX;

/// Retrieve the available movies for the given criteria.
///
/// It returns the [VecMovieC] reference on success, else [ptr::null_mut].
#[no_mangle]
pub extern "C" fn retrieve_available_movies(popcorn_fx: &mut PopcornFX, genre: &GenreC, sort_by: &SortByC, keywords: *const c_char, page: u32) -> Result<MediaSetC, MediaErrorC> {
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
                Ok(MediaSetC::from_movies(movies))
            } else {
                debug!("No movies have been found, returning ptr::null");
                Err(MediaErrorC::NoItemsFound)
            }
        }
        Err(e) => {
            error!("Failed to retrieve movies, {}", e);
            match e {
                _ => Err(MediaErrorC::Failed)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use popcorn_fx_core::{from_c_owned, into_c_string};
    use popcorn_fx_core::core::media::{Genre, SortBy};
    use popcorn_fx_core::testing::init_logger;

    use crate::test::default_args;

    use super::*;

    #[test]
    fn test_retrieve_available_movies() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let genre = GenreC::from(Genre::all());
        let sort_by = SortByC::from(SortBy::new(String::from("trending"), String::new()));
        let mut instance = PopcornFX::new(default_args(temp_path));

        let movies = retrieve_available_movies(&mut instance, &genre, &sort_by, into_c_string("".to_string()), 1);
    }
}