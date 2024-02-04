use log::trace;

use popcorn_fx_core::from_c_vec;

use crate::ffi::{ByteArray, StringArray};

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

/// Dispose of a C-compatible string array.
///
/// This function takes ownership of a boxed `StringArray` object, releasing its resources.
///
/// # Arguments
///
/// * `array` - A boxed `StringArray` object to be disposed of.
#[no_mangle]
pub extern "C" fn dispose_string_array(array: Box<StringArray>) {
    trace!("Disposing C string array {:?}", array);
    drop(from_c_vec(array.values, array.len));
}

#[cfg(test)]
mod tests {
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use tempfile::tempdir;

    use popcorn_fx_core::core::media::{Images, ShowOverview};
    use popcorn_fx_core::from_c_owned;
    use popcorn_fx_core::testing::{init_logger, read_test_file_to_bytes};

    use crate::ffi::{load_fanart, MediaItemC};
    use crate::PopcornFX;
    use crate::test::default_args;

    use super::*;

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

    #[test]
    fn test_dispose_string_array() {
        init_logger();
        let values = vec![
            "Foo".to_string(),
            "Bar".to_string()
        ];
        let array = StringArray::from(values);

        dispose_string_array(Box::new(array));
    }
}