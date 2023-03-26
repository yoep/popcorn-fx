use std::os::raw::c_char;
use std::ptr;

use log::{error, trace};

use popcorn_fx_core::{from_c_string, into_c_owned};

use crate::ffi::StringArray;
use crate::PopcornFX;

/// Retrieve the array of available genres for the given provider.
///
/// It returns an empty list when the provider name doesn't exist.
#[no_mangle]
pub extern "C" fn retrieve_provider_genres(popcorn_fx: &mut PopcornFX, name: *const c_char) -> *mut StringArray {
    let name = from_c_string(name);
    trace!("Retrieving genres from C for {}", name);
    match popcorn_fx.settings().properties().provider(name.as_str()) {
        Ok(e) => {
            into_c_owned(StringArray::from(e.genres()))
        }
        Err(e) => {
            error!("Provider name {} doesn't exist", e);
            ptr::null_mut()
        }
    }
}

/// Retrieve the array of available sorts for the given provider.
///
/// It returns an empty list when the provider name doesn't exist.
#[no_mangle]
pub extern "C" fn retrieve_provider_sort_by(popcorn_fx: &mut PopcornFX, name: *const c_char) -> *mut StringArray {
    let name = from_c_string(name);
    trace!("Retrieving sort_by from C for {}", name);
    match popcorn_fx.settings().properties().provider(name.as_str()) {
        Ok(e) => {
            into_c_owned(StringArray::from(e.sort_by()))
        }
        Err(e) => {
            error!("Provider name {} doesn't exist", e);
            ptr::null_mut()
        }
    }
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use popcorn_fx_core::{from_c_owned, from_c_vec, into_c_string};
    use popcorn_fx_core::testing::init_logger;

    use crate::PopcornFxArgs;

    use super::*;

    #[test]
    fn test_retrieve_provider_genres() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_youtube_video_player: true,
            disable_fx_video_player: false,
            disable_vlc_video_player: false,
            tv: false,
            maximized: false,
            insecure: false,
            app_directory: temp_path.to_string(),
            properties: Default::default(),
        });

        let array = from_c_owned(retrieve_provider_genres(&mut instance, into_c_string("series".to_string())));
        let result: Vec<String> = from_c_vec(array.values, array.len).into_iter()
            .map(|e| from_c_string(e))
            .collect();

        assert!(result.contains(&"adventure".to_string()), "expected the correct genres array")
    }

    #[test]
    fn test_retrieve_provider_genres_unknown_name() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_youtube_video_player: true,
            disable_fx_video_player: false,
            disable_vlc_video_player: false,
            tv: false,
            maximized: false,
            insecure: false,
            app_directory: temp_path.to_string(),
            properties: Default::default(),
        });

        let result = retrieve_provider_genres(&mut instance, into_c_string("lorem ipsum dolor estla".to_string()));

        assert!(result.is_null())
    }

    #[test]
    fn test_retrieve_provider_sort_by() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_youtube_video_player: true,
            disable_fx_video_player: false,
            disable_vlc_video_player: false,
            tv: false,
            maximized: false,
            insecure: false,
            app_directory: temp_path.to_string(),
            properties: Default::default(),
        });

        let array = from_c_owned(retrieve_provider_sort_by(&mut instance, into_c_string("favorites".to_string())));
        let result: Vec<String> = from_c_vec(array.values, array.len).into_iter()
            .map(|e| from_c_string(e))
            .collect();

        assert!(result.contains(&"watched".to_string()), "expected the correct sort_by array")
    }
}