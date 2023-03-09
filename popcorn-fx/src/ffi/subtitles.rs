use log::trace;

use popcorn_fx_core::{into_c_owned, SubtitleInfoC, SubtitleInfoSet};
use popcorn_fx_core::core::subtitles::model::SubtitleInfo;

use crate::PopcornFX;

/// Retrieve the default options available for the subtitles.
#[no_mangle]
pub extern "C" fn default_subtitle_options(popcorn_fx: &mut PopcornFX) -> *mut SubtitleInfoSet {
    trace!("Retrieving default subtitle options");
    let subtitles = popcorn_fx.subtitle_provider().default_subtitle_options();
    let subtitles: Vec<SubtitleInfoC> = subtitles.into_iter()
        .map(SubtitleInfoC::from)
        .collect();

    into_c_owned(SubtitleInfoSet::from(subtitles))
}

/// Retrieve a special [SubtitleInfo::none] type instance of the application.
#[no_mangle]
pub extern "C" fn subtitle_none() -> *mut SubtitleInfoC {
    into_c_owned(SubtitleInfoC::from(SubtitleInfo::none()))
}

/// Retrieve a special [SubtitleInfo::custom] type instance of the application.
#[no_mangle]
pub extern "C" fn subtitle_custom() -> *mut SubtitleInfoC {
    into_c_owned(SubtitleInfoC::from(SubtitleInfo::custom()))
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use popcorn_fx_core::{from_c_owned, from_c_vec};
    use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;
    use popcorn_fx_core::testing::init_logger;

    use crate::PopcornFxArgs;

    use super::*;

    #[test]
    fn test_default_subtitle_options() {
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
            app_directory: temp_path.to_string(),
        });
        let expected_result = vec![SubtitleInfo::none(), SubtitleInfo::custom()];

        let set_ptr = from_c_owned(default_subtitle_options(&mut instance));
        let result: Vec<SubtitleInfo> = from_c_vec(set_ptr.subtitles, set_ptr.len).into_iter()
            .map(SubtitleInfo::from)
            .collect();

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_subtitle_none() {
        init_logger();

        let result = from_c_owned(subtitle_none());

        assert_eq!(SubtitleLanguage::None, result.language)
    }

    #[test]
    fn test_subtitle_custom() {
        init_logger();

        let result = from_c_owned(subtitle_custom());

        assert_eq!(SubtitleLanguage::Custom, result.language)
    }
}