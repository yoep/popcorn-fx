use std::os::raw::c_char;
use std::ptr;

use log::{error, info, trace};

use popcorn_fx_core::core::media::Category;
use popcorn_fx_core::from_c_string;

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
    match popcorn_fx
        .runtime()
        .block_on(popcorn_fx.providers().retrieve(
            &Category::Favorites,
            &genre,
            &sort_by,
            &keywords,
            page,
        )) {
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

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use popcorn_fx_core::core::media::{Genre, SortBy};
    use popcorn_fx_core::testing::init_logger;

    use crate::test::default_args;

    use super::*;

    #[test]
    fn test_retrieve_available_favorites() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
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
    }
}
