use std::ptr;

use log::{error, trace};

use popcorn_fx_core::into_c_owned;

use crate::ffi::{UpdateCallbackC, UpdateEventC, UpdateStateC, VersionInfoC};
use crate::PopcornFX;

/// Retrieve the latest release version information from the update channel.
///
/// # Arguments
///
/// * `popcorn_fx` - a mutable reference to a `PopcornFX` instance.
#[no_mangle]
pub extern "C" fn version_info(popcorn_fx: &mut PopcornFX) -> *mut VersionInfoC {
    trace!("Retrieving version info");
    let runtime = popcorn_fx.runtime();
    match runtime.block_on(popcorn_fx.updater().version_info()) {
        Ok(version) => into_c_owned(VersionInfoC::from(&version)),
        Err(e) => {
            error!("Failed to poll version information, {}", e);
            ptr::null_mut()
        }
    }
}

/// Retrieve the current update state of the application.
///
/// # Arguments
///
/// * `popcorn_fx` - a mutable reference to a `PopcornFX` instance.
///
/// # Returns
///
/// The current update state of the application as a [UpdateStateC] value.
#[no_mangle]
pub extern "C" fn update_state(popcorn_fx: &mut PopcornFX) -> UpdateStateC {
    trace!("Retrieving update state from C");
    UpdateStateC::from(popcorn_fx.updater().state())
}

/// Start polling the update channel for new application versions.
///
/// # Arguments
///
/// * `popcorn_fx` - a mutable reference to a `PopcornFX` instance.
#[no_mangle]
pub extern "C" fn check_for_updates(popcorn_fx: &mut PopcornFX) {
    trace!("Checking for new updates from C");
    popcorn_fx.updater().check_for_updates()
}

/// Start downloading the application update if available.
///
/// # Arguments
///
/// * `popcorn_fx` - a mutable reference to a `PopcornFX` instance.
#[no_mangle]
pub extern "C" fn download_update(popcorn_fx: &mut PopcornFX) {
    let updater = popcorn_fx.updater().clone();
    popcorn_fx.runtime().spawn(async move {
        if let Err(e) = updater.download().await {
            error!("Failed to download update, {}", e)
        }
    });
}

/// Install the latest available update.
///
/// # Arguments
///
/// * `popcorn_fx` - a mutable reference to a `PopcornFX` instance.
#[no_mangle]
pub extern "C" fn install_update(popcorn_fx: &mut PopcornFX) {
    trace!("Starting installation update from C");
    if let Err(e) = popcorn_fx.updater().install() {
        error!("Failed to start update, {}", e);
    }
}

/// Register a new callback for update events.
///
/// This function registers a new callback listener for update events in the PopcornFX application.
/// The `callback` argument should be a C-compatible function that will be invoked when an update event occurs.
///
/// The `callback` function should take a single argument of type `UpdateEventC` and return nothing.
/// The `UpdateEventC` type is a C-compatible version of the `UpdateEvent` enum used internally by the PopcornFX updater.
///
/// # Arguments
///
/// * `popcorn_fx` - a mutable reference to a `PopcornFX` instance.
/// * `callback` - a C-compatible function that will be invoked when an update event occurs.
///
/// # Safety
///
/// This function should only be called from C code, and the provided `callback` function should be a valid C function pointer.
#[no_mangle]
pub extern "C" fn register_update_callback(popcorn_fx: &mut PopcornFX, callback: UpdateCallbackC) {
    trace!("Registering new update callback from C");
    popcorn_fx
        .updater()
        .register(Box::new(move |event| callback(UpdateEventC::from(event))))
}

#[cfg(test)]
mod test {
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use tempfile::tempdir;

    use popcorn_fx_core::{from_c_owned, from_c_string};
    use popcorn_fx_core::testing::init_logger;

    use crate::test::default_args;

    use super::*;

    #[test]
    fn test_version_info() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let server = MockServer::start();
        server.mock(|mock, then| {
            mock.method(GET).path("/update/versions.json");
            then.status(200).body(
                r#"{
  "application": {
    "version": "0.2.0",
    "platforms": {
        "debian.x86_64": "http://localhost/update/download/popcorn-time_0.6.5.tar.gz",
        "debian.arm64": "http://localhost/update/download/popcorn-time_0.6.5_arm64.tar.gz",
        "mac.x86_64": "http://localhost/update/download/popcorn-time_0.6.5.tar.gz",
        "windows.x86_64": "http://localhost/update/download/popcorn-time_0.6.5.tar.gz"
    }
  },
  "runtime": {
    "version": "17.0.6",
    "platforms": {
      "debian.x86_64": "http://localhost/update/download/runtime_debian_x86_64.tar.gz",
      "debian.arm64": "http://localhost/update/download/runtime_debian_arm64.tar.gz",
      "windows.x86_64": "http://localhost/update/download/runtime_windows.tar.gz"
    }
  }
}"#,
            );
        });
        let mut popcorn_fx_args = default_args(temp_path);
        popcorn_fx_args.properties.update_channel = server.url("/update/");
        let mut instance = PopcornFX::new(popcorn_fx_args);

        let result = from_c_owned(version_info(&mut instance));

        assert_eq!(
            "0.2.0".to_string(),
            from_c_string(result.application.version)
        );
        assert_eq!("17.0.6".to_string(), from_c_string(result.runtime.version));
    }

    #[test]
    fn test_check_for_updates() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));

        check_for_updates(&mut instance);
    }

    #[test]
    fn test_update_state() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));

        let result = update_state(&mut instance);

        match result {
            UpdateStateC::CheckingForNewVersion => {}
            UpdateStateC::NoUpdateAvailable => {}
            _ => panic!("expected one of [UpdateStateC::CheckingForNewVersion, UpdateStateC::NoUpdateAvailable] but got {:?} instead", result)
        }
    }

    #[test]
    fn test_download_update() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));

        download_update(&mut instance);
    }
}
