use std::os::raw::c_char;
use std::time::Instant;

use clap::{CommandFactory, FromArgMatches};
use log::{debug, info, trace};

use popcorn_fx_core::{from_c_string, from_c_vec, into_c_owned, into_c_string, VERSION};

use crate::{PopcornFX, PopcornFxArgs};

/// Create a new PopcornFX instance.
/// The caller will become responsible for managing the memory of the struct.
/// The instance can be safely deleted by using [dispose_popcorn_fx].
#[no_mangle]
pub extern "C" fn new_popcorn_fx(len: i32, args: *mut *mut c_char) -> *mut PopcornFX {
    trace!(
        "Creating new popcorn FX instance from C for args: {:?}",
        args
    );
    let start = Instant::now();
    let args = from_c_vec(args, len)
        .into_iter()
        .map(|e| from_c_string(e))
        .collect::<Vec<String>>();
    let matches = PopcornFxArgs::command()
        .allow_external_subcommands(true)
        .ignore_errors(true)
        .get_matches_from(args);
    let args = PopcornFxArgs::from_arg_matches(&matches).expect("expected valid args");
    let instance = PopcornFX::new(args);

    let time_taken = start.elapsed();
    info!(
        "Created new Popcorn FX instance in {}.{:03} seconds",
        time_taken.as_secs(),
        time_taken.subsec_millis()
    );
    into_c_owned(instance)
}

/// Starts the discovery process for external players such as VLC and DLNA servers.
#[no_mangle]
pub extern "C" fn discover_external_players(popcorn_fx: &mut PopcornFX) {
    trace!("Starting external player discovery from C");
    popcorn_fx.start_discovery_external_players();
}

/// Delete the PopcornFX instance, given as a [ptr], in a safe way.
/// All data within the instance will be deleted from memory making the instance unusable.
/// This means that the original pointer will become invalid.
#[no_mangle]
pub extern "C" fn dispose_popcorn_fx(instance: Box<PopcornFX>) {
    debug!("Disposing Popcorn FX instance");
    let start_time = Instant::now();
    drop(instance);
    let time_taken = start_time.elapsed();
    info!(
        "Disposed Popcorn FX instance in {}.{:03} seconds",
        time_taken.as_secs(),
        time_taken.subsec_millis()
    );
}

/// Retrieve the version of Popcorn FX.
#[no_mangle]
pub extern "C" fn version() -> *mut c_char {
    into_c_string(VERSION.to_string())
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use popcorn_fx_core::{from_c_string_owned, init_logger, into_c_vec};

    use crate::test::default_args;

    use super::*;

    #[test]
    fn test_new_popcorn_fx() {
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let (args, len) = into_c_vec(
            vec![
                "popcorn-fx".to_string(),
                format!("--app-directory={}", temp_path),
                "--disable-logger".to_string(),
            ]
            .into_iter()
            .map(|e| into_c_string(e))
            .collect(),
        );

        let result = new_popcorn_fx(len, args);

        assert!(!result.is_null(), "expected a valid instance pointer")
    }

    #[test]
    fn test_discover_external_players() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));

        discover_external_players(&mut instance);
    }

    #[test]
    fn test_dispose_popcorn_fx() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = PopcornFX::new(default_args(temp_path));

        dispose_popcorn_fx(Box::new(instance))
    }

    #[test]
    fn test_version() {
        let result = version();

        assert_eq!(VERSION.to_string(), from_c_string_owned(result))
    }
}
