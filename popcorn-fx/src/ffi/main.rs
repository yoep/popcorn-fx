use std::os::raw::c_char;
use std::time::Instant;

use clap::{CommandFactory, FromArgMatches};
use log::info;

use popcorn_fx_core::{from_c_string, from_c_vec, into_c_owned};

use crate::{PopcornFX, PopcornFxArgs};

/// Create a new PopcornFX instance.
/// The caller will become responsible for managing the memory of the struct.
/// The instance can be safely deleted by using [dispose_popcorn_fx].
#[no_mangle]
pub extern "C" fn new_popcorn_fx(args: *mut *const c_char, len: i32) -> *mut PopcornFX {
    let start = Instant::now();
    let args = from_c_vec(args, len).into_iter()
        .map(|e| from_c_string(e))
        .collect::<Vec<String>>();
    let matches = PopcornFxArgs::command()
        .allow_external_subcommands(true)
        .ignore_errors(true)
        .get_matches_from(args);
    let args = PopcornFxArgs::from_arg_matches(&matches).expect("expected valid args");
    let instance = PopcornFX::new(args);

    info!("Created new Popcorn FX instance in {} millis", start.elapsed().as_millis());
    into_c_owned(instance)
}

/// Delete the PopcornFX instance, given as a [ptr], in a safe way.
/// All data within the instance will be deleted from memory making the instance unusable.
/// This means that the original pointer will become invalid.
#[no_mangle]
pub extern "C" fn dispose_popcorn_fx(_: Box<PopcornFX>) {
    info!("Disposing Popcorn FX instance");
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use popcorn_fx_core::{into_c_string, to_c_vec};
    use popcorn_fx_core::testing::init_logger;

    use super::*;

    #[test]
    fn test_new_popcorn_fx() {
        init_logger();
        let (args, len) = to_c_vec(vec![
            "popcorn-fx".to_string(),
            "--disable-logger".to_string(),
        ].into_iter()
            .map(|e| into_c_string(e))
            .collect());

        let result = new_popcorn_fx(args, len);

        assert!(!result.is_null(), "expected a valid instance pointer")
    }

    #[test]
    fn test_dispose_popcorn_fx() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_youtube_video_player: false,
            disable_fx_video_player: false,
            disable_vlc_video_player: false,
            app_directory: temp_path.to_string(),
        });

        dispose_popcorn_fx(Box::new(instance))
    }
}