use std::ptr;

use log::{error, trace};

use popcorn_fx_core::into_c_owned;

use crate::ffi::{UpdateCallbackC, UpdateEventC, UpdateStateC, VersionInfoC};
use crate::PopcornFX;

/// Retrieve the latest release version information.
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
#[no_mangle]
pub extern "C" fn update_state(popcorn_fx: &mut PopcornFX) -> UpdateStateC {
    UpdateStateC::from(popcorn_fx.updater().state())
}

/// Start downloading the application update if available.
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
#[no_mangle]
pub extern "C" fn install_update(popcorn_fx: &mut PopcornFX) {
    trace!("Starting installation update from C");
    if let Err(e) = popcorn_fx.updater().install() {
        error!("Failed to start update, {}", e);
    }
}

/// Register a new callback for update events.
#[no_mangle]
pub extern "C" fn register_update_callback(popcorn_fx: &mut PopcornFX, callback: UpdateCallbackC) {
    trace!("Registering new update callback from C");
    popcorn_fx.updater().register(Box::new(move |event| {
        callback(UpdateEventC::from(event))
    }))
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use popcorn_fx_core::testing::init_logger;

    use crate::PopcornFxArgs;

    use super::*;

    #[test]
    fn test_version_info() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_youtube_video_player: false,
            disable_fx_video_player: false,
            disable_vlc_video_player: false,
            tv: false,
            maximized: false,
            insecure: false,
            app_directory: temp_path.to_string(),
        });

        let result = version_info(&mut instance);

        assert!(!result.is_null())
    }

    #[test]
    fn test_update_state() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_youtube_video_player: false,
            disable_fx_video_player: false,
            disable_vlc_video_player: false,
            tv: false,
            maximized: false,
            insecure: false,
            app_directory: temp_path.to_string(),
        });

        let result = update_state(&mut instance);

        match result {
            UpdateStateC::CheckingForNewVersion => {}
            _ => assert!(false, "expected UpdateStateC::CheckingForNewVersion")
        }
    }

    #[test]
    fn test_download_update() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_youtube_video_player: false,
            disable_fx_video_player: false,
            disable_vlc_video_player: false,
            tv: false,
            maximized: false,
            insecure: false,
            app_directory: temp_path.to_string(),
        });

        download_update(&mut instance);
    }
}