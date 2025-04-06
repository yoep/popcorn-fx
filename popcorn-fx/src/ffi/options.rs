use crate::PopcornFX;

/// Verify if the FX embedded video player has been disabled.
#[no_mangle]
pub extern "C" fn is_fx_video_player_enabled(popcorn_fx: &mut PopcornFX) -> bool {
    popcorn_fx.opts().enable_fx_video_player
}

/// Verify if the application mouse should be disabled.
/// The disabling of the mouse should be implemented by the UI implementation and has no behavior on
/// the backend itself.
#[no_mangle]
pub extern "C" fn is_mouse_disabled(popcorn_fx: &mut PopcornFX) -> bool {
    popcorn_fx.opts().disable_mouse
}

/// Verify if the TV mode is activated for the application.
#[no_mangle]
pub extern "C" fn is_tv_mode(popcorn_fx: &mut PopcornFX) -> bool {
    popcorn_fx.opts().tv
}

/// Verify if the application should be maximized on startup.
#[no_mangle]
pub extern "C" fn is_maximized(popcorn_fx: &mut PopcornFX) -> bool {
    popcorn_fx.opts().maximized
}

/// Verify if the application should started in kiosk mode.
/// The behavior of kiosk mode is dependant on the UI implementation and not delegated by the backend.
#[no_mangle]
pub extern "C" fn is_kiosk_mode(popcorn_fx: &mut PopcornFX) -> bool {
    popcorn_fx.opts().kiosk
}

/// Checks if the YouTube video player is enabled in the PopcornFX options.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
///
/// # Returns
///
/// `true` if the YouTube video player is enabled, otherwise `false`.
#[no_mangle]
pub extern "C" fn is_youtube_video_player_enabled(popcorn_fx: &mut PopcornFX) -> bool {
    popcorn_fx.opts().enable_youtube_video_player
}

/// Checks if the VLC video player is enabled in the PopcornFX options.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
///
/// # Returns
///
/// `true` if the VLC video player is enabled, otherwise `false`.
#[no_mangle]
pub extern "C" fn is_vlc_video_player_enabled(popcorn_fx: &mut PopcornFX) -> bool {
    popcorn_fx.opts().enable_vlc_video_player
}

#[cfg(test)]
mod test {
    use popcorn_fx_core::init_logger;
    use tempfile::tempdir;

    use crate::PopcornFxArgs;

    use super::*;

    #[test]
    fn test_is_youtube_video_player_enabled() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_mouse: false,
            enable_youtube_video_player: true,
            enable_fx_video_player: false,
            enable_vlc_video_player: false,
            tv: false,
            maximized: false,
            kiosk: false,
            insecure: false,
            app_directory: temp_path.to_string(),
            data_directory: temp_dir.path().join("data").to_str().unwrap().to_string(),
            properties: Default::default(),
        });

        let result = is_youtube_video_player_enabled(&mut instance);

        assert_eq!(true, result)
    }

    #[test]
    fn test_is_fx_video_player_enabled() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_mouse: false,
            enable_youtube_video_player: false,
            enable_fx_video_player: true,
            enable_vlc_video_player: false,
            tv: false,
            maximized: false,
            kiosk: false,
            insecure: false,
            app_directory: temp_path.to_string(),
            data_directory: temp_dir.path().join("data").to_str().unwrap().to_string(),
            properties: Default::default(),
        });

        let result = is_fx_video_player_enabled(&mut instance);

        assert_eq!(true, result, "expected FX video player to be enabled");
    }

    #[test]
    fn test_is_vlc_video_player_enabled() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_mouse: false,
            enable_youtube_video_player: false,
            enable_fx_video_player: false,
            enable_vlc_video_player: true,
            tv: false,
            maximized: false,
            kiosk: false,
            insecure: false,
            app_directory: temp_path.to_string(),
            data_directory: temp_dir.path().join("data").to_str().unwrap().to_string(),
            properties: Default::default(),
        });

        let result = is_vlc_video_player_enabled(&mut instance);

        assert_eq!(true, result)
    }

    #[test]
    fn test_is_mouse_disabled() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_mouse: true,
            enable_youtube_video_player: false,
            enable_fx_video_player: false,
            enable_vlc_video_player: false,
            tv: false,
            maximized: false,
            kiosk: false,
            insecure: false,
            app_directory: temp_path.to_string(),
            data_directory: temp_dir.path().join("data").to_str().unwrap().to_string(),
            properties: Default::default(),
        });

        let result = is_mouse_disabled(&mut instance);

        assert_eq!(true, result)
    }

    #[test]
    fn test_is_tv_mode() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_mouse: false,
            enable_youtube_video_player: false,
            enable_fx_video_player: false,
            enable_vlc_video_player: false,
            tv: true,
            maximized: false,
            kiosk: false,
            insecure: false,
            app_directory: temp_path.to_string(),
            data_directory: temp_dir.path().join("data").to_str().unwrap().to_string(),
            properties: Default::default(),
        });

        let result = is_tv_mode(&mut instance);

        assert_eq!(true, result)
    }

    #[test]
    fn test_is_maximized() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_mouse: false,
            enable_youtube_video_player: false,
            enable_fx_video_player: false,
            enable_vlc_video_player: false,
            tv: false,
            maximized: true,
            kiosk: false,
            insecure: false,
            app_directory: temp_path.to_string(),
            data_directory: temp_dir.path().join("data").to_str().unwrap().to_string(),
            properties: Default::default(),
        });

        let result = is_maximized(&mut instance);

        assert_eq!(true, result)
    }

    #[test]
    fn test_is_kiosk_mode() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_mouse: false,
            enable_youtube_video_player: false,
            enable_fx_video_player: false,
            enable_vlc_video_player: false,
            tv: false,
            maximized: true,
            kiosk: true,
            insecure: false,
            app_directory: temp_path.to_string(),
            data_directory: temp_dir.path().join("data").to_str().unwrap().to_string(),
            properties: Default::default(),
        });

        let result = is_kiosk_mode(&mut instance);

        assert_eq!(true, result)
    }
}
