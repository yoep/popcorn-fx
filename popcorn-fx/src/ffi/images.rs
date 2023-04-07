use std::os::raw::c_char;
use std::ptr;

use log::{trace, warn};

use popcorn_fx_core::{from_c_string, from_c_vec, into_c_owned};

use crate::ffi::{ByteArray, MediaItemC};
use crate::PopcornFX;

/// Retrieve the default poster (placeholder) image data as a C compatible byte array.
///
/// This function returns a pointer to a `ByteArray` struct that contains the data for the default poster placeholder image.
/// The default poster placeholder image is typically used as a fallback when a poster image is not available for a media item.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the `PopcornFX` instance.
///
/// # Returns
///
/// A pointer to a `ByteArray` struct containing the default poster holder image data.
///
/// # Safety
///
/// This function should only be called from C code, and the returned byte array should be disposed of using the `dispose_byte_array` function.
#[no_mangle]
pub extern "C" fn poster_placeholder(popcorn_fx: &mut PopcornFX) -> *mut ByteArray {
    trace!("Retrieving the default poster image from C");
    into_c_owned(ByteArray::from(popcorn_fx.image_loader().default_poster()))
}

/// Retrieve the default artwork (placeholder) image data as a C-compatible byte array.
///
/// This function returns a C-compatible byte array containing the data for the default artwork (placeholder) image.
/// The default artwork image is typically used as a fallback when an artwork image is not available for a media item or is still being loaded.
///
/// # Arguments
///
/// * `popcorn_fx` - a mutable reference to a `PopcornFX` instance.
///
/// # Safety
///
/// This function should only be called from C code, and the returned byte array should be disposed of using the `dispose_byte_array` function.
#[no_mangle]
pub extern "C" fn artwork_placeholder(popcorn_fx: &mut PopcornFX) -> *mut ByteArray {
    trace!("Retrieving the default artwork image from C");
    into_c_owned(ByteArray::from(popcorn_fx.image_loader().default_artwork()))
}

/// Loads the fanart image data for the given media item.
///
/// This function should be called from C code in order to load fanart image data for a media item.
///
/// # Arguments
///
/// * `popcorn_fx` - a mutable reference to the `PopcornFX` instance that will load the image data.
/// * `media` - a C-compatible media item holder that contains information about the media item to load.
///
/// # Returns
///
/// If fanart image data is available for the media item, a C-compatible byte array containing the image data is returned.
/// Otherwise, a placeholder byte array is returned.
///
/// # Safety
///
/// This function should only be called from C code, and the returned byte array should be disposed of using the `dispose_byte_array` function.
#[no_mangle]
pub extern "C" fn load_fanart(popcorn_fx: &mut PopcornFX, media: &MediaItemC) -> *mut ByteArray {
    trace!("Loading fanart from C for {:?}", media);
    let image_loader = popcorn_fx.image_loader().clone();
    popcorn_fx.runtime().block_on(async move {
        match media.as_overview() {
            None => into_c_owned(ByteArray::from(vec![])),
            Some(media_overview) => {
                into_c_owned(ByteArray::from(image_loader.load_fanart(&media_overview).await))
            }
        }
    })
}

/// Load the poster image data for the given media item.
///
/// If poster image data is available for the media item, it is returned as a `ByteArray`.
/// Otherwise, a placeholder `ByteArray` containing the default poster holder image data is returned.
///
/// # Arguments
///
/// * `popcorn_fx` - a mutable reference to a `PopcornFX` instance.
/// * `media` - a reference to a `MediaItemC` object that represents the media item to load.
///
/// # Safety
///
/// This function should only be called from C code, and the returned byte array should be disposed of using the `dispose_byte_array` function.
#[no_mangle]
pub extern "C" fn load_poster(popcorn_fx: &mut PopcornFX, media: &MediaItemC) -> *mut ByteArray {
    trace!("Loading poster from C for {:?}", media);
    let image_loader = popcorn_fx.image_loader().clone();
    popcorn_fx.runtime().block_on(async move {
        match media.as_overview() {
            None => into_c_owned(ByteArray::from(vec![])),
            Some(media_overview) => {
                into_c_owned(ByteArray::from(image_loader.load_poster(&media_overview).await))
            }
        }
    })
}

/// Load the image data from the given URL.
///
/// If image data is available for the provided URL, it is returned as a ByteArray.
/// Otherwise, a null pointer is returned when the data couldn't be loaded.
///
/// # Arguments
///
/// * popcorn_fx - a mutable reference to a PopcornFX instance.
/// * url - a pointer to a null-terminated C string that contains the URL from which to load the image data.
///
/// # Safety
///
/// This function should only be called from C code, and the returned byte array should be disposed of using the dispose_byte_array function.
#[no_mangle]
pub extern "C" fn load_image(popcorn_fx: &mut PopcornFX, url: *const c_char) -> *mut ByteArray {
    trace!("Loading image url from C for {:?}", url);
    let url = from_c_string(url);
    let image_loader = popcorn_fx.image_loader().clone();
    popcorn_fx.runtime().block_on(async move {
        match image_loader.load(url.as_str()).await {
            None => {
                warn!("Failed to load the image data from url {}", url);
                ptr::null_mut()
            }
            Some(data) => {
                into_c_owned(ByteArray::from(data))
            }
        }
    })
}

/// Frees the memory allocated for the given C-compatible byte array.
///
/// This function should be called from C code in order to free memory that has been allocated by Rust.
///
/// # Safety
///
/// This function should only be called on C-compatible byte arrays that have been allocated by Rust.
#[no_mangle]
pub extern "C" fn dispose_byte_array(array: Box<ByteArray>) {
    trace!("Disposing C byte array {:?}", array);
    drop(from_c_vec(array.values, array.len));
}

#[cfg(test)]
mod test {
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use tempfile::tempdir;

    use popcorn_fx_core::{from_c_owned, from_c_vec, into_c_string};
    use popcorn_fx_core::core::media::{Images, MovieDetails, ShowDetails, ShowOverview};
    use popcorn_fx_core::testing::{init_logger, read_test_file_to_bytes};

    use crate::test::default_args;

    use super::*;

    #[test]
    fn test_default_poster() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));

        let array = from_c_owned(poster_placeholder(&mut instance));
        let result = from_c_vec(array.values, array.len);

        assert!(result.len() > 0)
    }

    #[test]
    fn test_load_fanart() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let expected_result = read_test_file_to_bytes("image.jpg");
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET)
                .path("/fanart.png");
            then.status(200)
                .body(expected_result.as_slice());
        });
        let media = MovieDetails {
            title: "lorem ipsum".to_string(),
            imdb_id: "tt55555".to_string(),
            year: "2006".to_string(),
            runtime: "96".to_string(),
            genres: vec![],
            synopsis: "".to_string(),
            rating: None,
            images: Images {
                poster: "".to_string(),
                fanart: server.url("/fanart.png"),
                banner: "".to_string(),
            },
            trailer: "".to_string(),
            torrents: Default::default(),
        };
        let mut instance = PopcornFX::new(default_args(temp_path));

        let array = from_c_owned(load_fanart(&mut instance, &MediaItemC::from(media)));
        let result = from_c_vec(array.values, array.len);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_load_poster() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let expected_result = read_test_file_to_bytes("image.jpg");
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET)
                .path("/poster.png");
            then.status(200)
                .body(expected_result.as_slice());
        });
        let media = ShowDetails {
            imdb_id: "".to_string(),
            tvdb_id: "".to_string(),
            title: "".to_string(),
            year: "".to_string(),
            num_seasons: 0,
            images: Images {
                poster: server.url("/poster.png"),
                fanart: "".to_string(),
                banner: "".to_string(),
            },
            rating: None,
            context_locale: "".to_string(),
            synopsis: "".to_string(),
            runtime: "".to_string(),
            status: "".to_string(),
            genres: vec![],
            episodes: vec![],
            liked: None,
        };
        let mut instance = PopcornFX::new(default_args(temp_path));

        let array = from_c_owned(load_poster(&mut instance, &MediaItemC::from(media)));
        let result = from_c_vec(array.values, array.len);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_load_image() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let expected_result = read_test_file_to_bytes("image.jpg");
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET)
                .path("/image.png");
            then.status(200)
                .body(expected_result.as_slice());
        });
        let mut instance = PopcornFX::new(default_args(temp_path));

        let array = from_c_owned(load_image(&mut instance, into_c_string(server.url("/image.png"))));
        let result = from_c_vec(array.values, array.len);

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_dispose_byte_array() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let expected_result = read_test_file_to_bytes("image.jpg");
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET)
                .path("/fanart.png");
            then.status(200)
                .body(expected_result.as_slice());
        });
        let media = ShowOverview {
            imdb_id: "tt212121".to_string(),
            tvdb_id: "212121".to_string(),
            title: "Ipsum dolor".to_string(),
            year: "2004".to_string(),
            num_seasons: 0,
            images: Images {
                poster: "".to_string(),
                fanart: server.url("/fanart.png"),
                banner: "".to_string(),
            },
            rating: None,
        };
        let mut instance = PopcornFX::new(default_args(temp_path));

        let array = from_c_owned(load_fanart(&mut instance, &MediaItemC::from(media)));
        dispose_byte_array(Box::new(array))
    }
}