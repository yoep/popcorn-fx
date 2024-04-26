use log::trace;

use popcorn_fx_core::core::screen::DefaultScreenService;

use crate::PopcornFX;

/// Type definition for a callback function that checks if the application is in fullscreen mode.
///
/// The `IsFullscreenCallback` type represents an external callback function that returns a boolean value to indicate
/// whether the application is in fullscreen mode.
pub type IsFullscreenCallback = extern "C" fn() -> bool;

/// Type definition for a callback function that handles fullscreen events.
///
/// The `FullscreenCallback` type represents an external callback function that takes a boolean parameter to indicate
/// the fullscreen state.
pub type FullscreenCallback = extern "C" fn(bool);

/// Register a callback function to check if the application is in fullscreen mode.
///
/// This function registers a new callback function to check the fullscreen state within the PopcornFX instance.
///
/// # Arguments
///
/// * `instance` - A mutable reference to the `PopcornFX` instance.
/// * `callback` - The callback function to be registered for checking the fullscreen state.
#[no_mangle]
pub extern "C" fn register_is_fullscreen_callback(
    instance: &mut PopcornFX,
    callback: IsFullscreenCallback,
) {
    trace!("Registering new is fullscreen callback for C");

    // Check if the screen service is a DefaultScreenService and register the callback
    if let Some(screen) = instance
        .screen_service()
        .downcast_ref::<DefaultScreenService>()
    {
        screen.register_is_fullscreen_callback(Box::new(move || {
            trace!("Calling is fullscreen callback");
            callback()
        }));
    }
}

/// Register a fullscreen callback function.
///
/// This function registers a new fullscreen callback function to handle fullscreen events within the PopcornFX instance.
///
/// # Arguments
///
/// * `instance` - A mutable reference to the `PopcornFX` instance.
/// * `callback` - The fullscreen callback function to be registered.
#[no_mangle]
pub extern "C" fn register_fullscreen_callback(
    instance: &mut PopcornFX,
    callback: FullscreenCallback,
) {
    trace!("Registering new fullscreen callback for C");
    if let Some(screen) = instance
        .screen_service()
        .downcast_ref::<DefaultScreenService>()
    {
        screen.register_fullscreen_callback(Box::new(move |value| {
            trace!("Calling fullscreen callback with {}", value);
            callback(value);
        }));
    }
}

#[cfg(test)]
mod tests {
    use log::info;
    use tempfile::tempdir;

    use popcorn_fx_core::testing::init_logger;

    use crate::test::default_args;

    use super::*;

    extern "C" fn is_fullscreen_callback() -> bool {
        info!("Received is fullscreen callback");
        true
    }

    extern "C" fn fullscreen_callback(value: bool) {
        info!("Received fullscreen callback {}", value);
    }

    #[test]
    fn test_register_is_fullscreen_callback() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));

        register_is_fullscreen_callback(&mut instance, is_fullscreen_callback);

        let result = instance.screen_service().is_fullscreen();
        assert_eq!(true, result);
    }

    #[test]
    fn test_register_fullscreen_callback() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));

        register_fullscreen_callback(&mut instance, fullscreen_callback);
        instance.screen_service().fullscreen(true);
    }
}
