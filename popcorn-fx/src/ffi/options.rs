use crate::PopcornFX;

/// Verify if the youtube video player has been disabled.
#[no_mangle]
pub extern "C" fn is_youtube_video_player_disabled(popcorn_fx: &mut PopcornFX) -> bool {
    popcorn_fx.opts().disable_youtube_video_player
}

/// Verify if the FX embedded video player has been disabled.
#[no_mangle]
pub extern "C" fn is_fx_video_player_disabled(popcorn_fx: &mut PopcornFX) -> bool {
    popcorn_fx.opts().disable_fx_video_player
}

/// Verify if the vlc video player has been disabled.
#[no_mangle]
pub extern "C" fn is_vlc_video_player_disabled(popcorn_fx: &mut PopcornFX) -> bool {
    popcorn_fx.opts().disable_vlc_video_player
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

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use popcorn_fx_core::testing::init_logger;

    use crate::PopcornFxArgs;

    use super::*;

    #[test]
    fn test_is_youtube_video_player_disabled() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_youtube_video_player: true,
            disable_fx_video_player: false,
            disable_vlc_video_player: false,
            tv: false,
            maximized: false,
            app_directory: temp_path.to_string(),
        });

        let result = is_youtube_video_player_disabled(&mut instance);

        assert_eq!(true, result)
    }

    #[test]
    fn test_is_fx_video_player_disabled() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_youtube_video_player: false,
            disable_fx_video_player: true,
            disable_vlc_video_player: false,
            tv: false,
            maximized: false,
            app_directory: temp_path.to_string(),
        });

        let result = is_fx_video_player_disabled(&mut instance);

        assert_eq!(true, result)
    }

    #[test]
    fn test_is_vlc_video_player_disabled() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_youtube_video_player: false,
            disable_fx_video_player: false,
            disable_vlc_video_player: true,
            tv: false,
            maximized: false,
            app_directory: temp_path.to_string(),
        });

        let result = is_vlc_video_player_disabled(&mut instance);

        assert_eq!(true, result)
    }

    #[test]
    fn test_is_tv_mode() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_youtube_video_player: false,
            disable_fx_video_player: false,
            disable_vlc_video_player: false,
            tv: true,
            maximized: false,
            app_directory: temp_path.to_string(),
        });

        let result = is_tv_mode(&mut instance);

        assert_eq!(true, result)
    }

    #[test]
    fn test_is_maximized() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(PopcornFxArgs {
            disable_logger: true,
            disable_youtube_video_player: false,
            disable_fx_video_player: false,
            disable_vlc_video_player: false,
            tv: false,
            maximized: true,
            app_directory: temp_path.to_string(),
        });

        let result = is_maximized(&mut instance);

        assert_eq!(true, result)
    }
}