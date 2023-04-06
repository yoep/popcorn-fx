use log::trace;

use popcorn_fx_core::into_c_owned;

use crate::ffi::{ByteArray, MediaItemC};
use crate::PopcornFX;

/// Load the fanart image data for the given media item.
///
/// It will return a byte array with the image data when available,
/// else the placeholder data to use instead on failure.
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

#[cfg(test)]
mod test {
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use tempfile::tempdir;

    use popcorn_fx_core::core::media::{Images, MovieDetails};
    use popcorn_fx_core::from_c_vec;
    use popcorn_fx_core::testing::{init_logger, read_test_file_to_bytes};

    use crate::test::default_args;

    use super::*;

    #[test]
    fn test_load_fanart() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let expected_result = read_test_file_to_bytes("image.jpg");
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET)
                .path("fanart.png");
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
                fanart: server.url("fanart.png"),
                banner: "".to_string(),
            },
            trailer: "".to_string(),
            torrents: Default::default(),
        };
        let mut instance = PopcornFX::new(default_args(temp_path));

        let array = load_fanart(&mut instance, &MediaItemC::from(media));
        let result = from_c_vec(array.values, array.len);

        assert_eq!(expected_result, result)
    }
}