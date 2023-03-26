use log::trace;

use crate::ffi::PlaybackControlsCallbackC;
use crate::PopcornFX;

/// Register a new callback listener for the playback controls.
#[no_mangle]
pub extern "C" fn register_playback_controls(popcorn_fx: &mut PopcornFX, callback: PlaybackControlsCallbackC) {
    trace!("Registering new playback controls callback from C");
    popcorn_fx.playback_controls().register(Box::new(move |event| {
        callback(event)
    }))
}

#[cfg(test)]
mod test {
    use log::info;
    use tempfile::tempdir;

    use popcorn_fx_core::core::playback::PlaybackControlEvent;
    use popcorn_fx_core::testing::init_logger;

    use crate::test::default_args;

    use super::*;

    #[no_mangle]
    pub extern "C" fn playback_controls_callback(event: PlaybackControlEvent) {
        info!("Received playback control callback event {:?}", event)
    }

    #[test]
    fn test_register_playback_controls() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));

        register_playback_controls(&mut instance, playback_controls_callback);
    }
}