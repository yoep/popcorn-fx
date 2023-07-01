use log::trace;

use popcorn_fx_core::core::subtitles::model::SubtitleInfo;
use popcorn_fx_core::core::subtitles::SubtitleCallback;
use popcorn_fx_core::into_c_owned;

use crate::ffi::{SubtitleEventC, SubtitleInfoC, SubtitleInfoSet};
use crate::PopcornFX;

/// The C callback for the subtitle events.
pub type SubtitleCallbackC = extern "C" fn(SubtitleEventC);

/// Retrieve the default options available for the subtitles.
///
/// # Safety
///
/// This function should only be called from C code.
/// The `popcorn_fx` pointer must be valid and properly initialized.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
///
/// # Returns
///
/// A pointer to a `SubtitleInfoSet` instance.
#[no_mangle]
pub extern "C" fn default_subtitle_options(popcorn_fx: &mut PopcornFX) -> *mut SubtitleInfoSet {
    trace!("Retrieving default subtitle options");
    let subtitles = popcorn_fx.subtitle_provider().default_subtitle_options();
    let subtitles: Vec<SubtitleInfoC> = subtitles.into_iter()
        .map(SubtitleInfoC::from)
        .collect();

    into_c_owned(SubtitleInfoSet::from(subtitles))
}

/// Retrieve a special [SubtitleInfo::none] instance of the application.
///
/// # Safety
///
/// This function should only be called from C code.
///
/// # Returns
///
/// A pointer to a `SubtitleInfoC` instance representing "none".
#[no_mangle]
pub extern "C" fn subtitle_none() -> *mut SubtitleInfoC {
    into_c_owned(SubtitleInfoC::from(SubtitleInfo::none()))
}

/// Retrieve a special [SubtitleInfo::custom] instance of the application.
///
/// # Safety
///
/// This function should only be called from C code.
///
/// # Returns
///
/// A pointer to a `SubtitleInfoC` instance representing "custom".
#[no_mangle]
pub extern "C" fn subtitle_custom() -> *mut SubtitleInfoC {
    into_c_owned(SubtitleInfoC::from(SubtitleInfo::custom()))
}

/// Register a new callback for subtitle events.
///
/// # Safety
///
/// This function should only be called from C code.
/// The `popcorn_fx` pointer must be valid and properly initialized.
/// The `callback` function pointer should point to a valid C function that can receive a `SubtitleEventC` parameter and return nothing.
/// The callback function will be invoked whenever a subtitle event occurs in the system.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
/// * `callback` - A function pointer to the C callback function.
#[no_mangle]
pub extern "C" fn register_subtitle_callback(popcorn_fx: &mut PopcornFX, callback: SubtitleCallbackC) {
    trace!("Wrapping C callback for SubtitleCallback");
    let wrapper: SubtitleCallback = Box::new(move |event| {
        let event_c = SubtitleEventC::from(event);
        trace!("Invoking SubtitleEventC {:?}", event_c);
        callback(event_c)
    });

    popcorn_fx.subtitle_manager().register(wrapper);
}

/// Clean the subtitles directory.
///
/// # Safety
///
/// This function should only be called from C code.
/// The `popcorn_fx` pointer must be valid and properly initialized.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
#[no_mangle]
pub extern "C" fn cleanup_subtitles_directory(popcorn_fx: &mut PopcornFX) {
    trace!("Cleaning subtitles directory from C");
    popcorn_fx.subtitle_manager().cleanup()
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use log::info;
    use tempfile::tempdir;

    use popcorn_fx_core::{from_c_owned, from_c_vec};
    use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;
    use popcorn_fx_core::testing::{copy_test_file, init_logger};

    use crate::test::{default_args, new_instance};

    use super::*;

    #[no_mangle]
    pub extern "C" fn subtitle_callback(event: SubtitleEventC) {
        info!("Received subtitle callback event {:?}", event)
    }

    #[test]
    fn test_default_subtitle_options() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = new_instance(temp_path);
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

    #[test]
    fn test_register_subtitle_callback() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = new_instance(temp_path);

        register_subtitle_callback(&mut instance, subtitle_callback);
        instance.subtitle_manager().update_subtitle(SubtitleInfo::none())
    }

    #[test]
    fn test_cleanup_subtitles_directory() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = new_instance(temp_path);
        let filepath = copy_test_file(instance.settings().settings.subtitle_settings.directory.as_str(), "example.srt", None);

        cleanup_subtitles_directory(&mut instance);

        assert_eq!(false, PathBuf::from(filepath).exists(), "expected the subtitle file to have been cleaned");
    }
}