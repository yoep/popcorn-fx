use std::os::raw::c_char;
use std::ptr;

use log::{debug, error, info, trace};
use popcorn_fx_core::core::block_in_place_runtime;
use popcorn_fx_core::core::media::Category;
use popcorn_fx_core::{from_c_string, from_c_vec};

use crate::ffi::{favorites_to_c, GenreC, SortByC, VecFavoritesC};
use crate::PopcornFX;

/// Retrieves available favorites from a PopcornFX instance.
///
/// This function retrieves favorites from the provided `popcorn_fx` instance,
/// filtering them based on the specified `genre`, `sort_by`, `keywords`, and `page`.
///
/// # Safety
///
/// This function is marked as unsafe due to potential undefined behavior caused by
/// invalid pointers or memory access when interacting with C code.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a PopcornFX instance.
/// * `genre` - A pointer to a GenreC struct, representing the genre filter.
/// * `sort_by` - A pointer to a SortByC struct, representing the sorting criteria.
/// * `keywords` - A pointer to a C-style string containing search keywords.
/// * `page` - The page number for pagination.
///
/// # Returns
///
/// If successful, returns a pointer to a VecFavoritesC struct containing the retrieved favorites.
/// Returns a null pointer if an error occurs during the retrieval process.
#[no_mangle]
pub extern "C" fn retrieve_available_favorites(
    popcorn_fx: &mut PopcornFX,
    genre: &GenreC,
    sort_by: &SortByC,
    keywords: *mut c_char,
    page: u32,
) -> *mut VecFavoritesC {
    trace!(
        "Retrieving favorites from C for genre: {:?}, sort_by: {:?}, keywords: {:?}, page: {}",
        genre,
        sort_by,
        keywords,
        page
    );
    let genre = genre.to_struct();
    let sort_by = sort_by.to_struct();
    let keywords = from_c_string(keywords);

    trace!(
        "Retrieving favorites for genre: {:?}, sort_by: {:?}, page: {}",
        genre,
        sort_by,
        page
    );
    let providers = popcorn_fx.providers().clone();
    match block_in_place_runtime(
        providers.retrieve(&Category::Favorites, &genre, &sort_by, &keywords, page),
        popcorn_fx.runtime(),
    ) {
        Ok(e) => {
            info!("Retrieved a total of {} favorites", e.len());
            debug!("Favorite items {:?}", e);
            favorites_to_c(e)
        }
        Err(e) => {
            error!("Failed to retrieve favorites, {}", e);
            ptr::null_mut()
        }
    }
}

/// Dispose of a C-compatible favorites collection.
///
/// This function is responsible for cleaning up resources associated with a C-compatible favorites collection.
///
/// # Arguments
///
/// * `favorites` - A C-compatible favorites collection to be disposed of.
#[no_mangle]
pub extern "C" fn dispose_favorites(favorites: Box<VecFavoritesC>) {
    trace!("Disposing favorite C set {:?}", favorites);
    if !favorites.movies.is_null() {
        drop(from_c_vec(favorites.movies, favorites.movies_len));
    }
    if !favorites.shows.is_null() {
        drop(from_c_vec(favorites.shows, favorites.shows_len));
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::ffi::{MovieOverviewC, ShowOverviewC};
    use crate::test::default_args;
    use popcorn_fx_core::core::media::{Genre, MovieOverview, ShowOverview, SortBy};
    use popcorn_fx_core::testing::copy_test_file;
    use popcorn_fx_core::{from_c_owned, init_logger};

    use super::*;

    #[test]
    fn test_retrieve_available_favorites() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        copy_test_file(temp_path, "favorites.json", None);
        let mut instance = PopcornFX::new(default_args(temp_path));

        let result = retrieve_available_favorites(
            &mut instance,
            &GenreC::from(Genre::all()),
            &SortByC::from(SortBy::new("Watched".to_string(), "watched".to_string())),
            ptr::null_mut(),
            0,
        );

        assert!(
            !result.is_null(),
            "expected the favorites set to be non-null"
        );
        let result = from_c_owned(result);
        assert_eq!(result.movies_len, 1);
        assert_eq!(result.shows_len, 2);
    }

    #[test]
    fn test_dispose_favorites() {
        init_logger!();
        let movies = vec![MovieOverviewC::from(MovieOverview {
            title: "Foo".to_string(),
            imdb_id: "tt112233".to_string(),
            year: "2013".to_string(),
            rating: None,
            images: Default::default(),
        })];
        let favorites_set = VecFavoritesC::from(movies, Vec::new());

        dispose_favorites(Box::new(favorites_set));

        let shows = vec![ShowOverviewC::from(ShowOverview {
            title: "Bar".to_string(),
            imdb_id: "tt112233".to_string(),
            tvdb_id: "tt001122".to_string(),
            year: "2010".to_string(),
            num_seasons: 3,
            images: Default::default(),
            rating: None,
        })];
        let favorites_set = VecFavoritesC::from(Vec::new(), shows);

        dispose_favorites(Box::new(favorites_set));
    }
}
