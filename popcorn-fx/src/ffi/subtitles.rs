use log::trace;

use crate::ffi::{SubtitleC, SubtitleEventC, SubtitleInfoC, SubtitleInfoSet};
use crate::PopcornFX;
use popcorn_fx_core::core::block_in_place_runtime;
use popcorn_fx_core::core::subtitles::model::SubtitleInfo;
use popcorn_fx_core::core::subtitles::{SubtitleCallback, SubtitlePreference};
use popcorn_fx_core::{from_c_vec, into_c_owned};

/// The C callback for the subtitle events.
pub type SubtitleCallbackC = extern "C" fn(SubtitleEventC);

/// Retrieves the current subtitle preference from PopcornFX.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
///
/// # Returns
///
/// A pointer to a `SubtitlePreference` instance.
///
#[no_mangle]
pub extern "C" fn retrieve_subtitle_preference(
    popcorn_fx: &mut PopcornFX,
) -> *mut SubtitlePreference {
    let subtitle_manager = popcorn_fx.subtitle_manager().clone();
    let preference = block_in_place_runtime(subtitle_manager.preference(), popcorn_fx.runtime());
    into_c_owned(preference)
}

/// Updates the subtitle preference for PopcornFX.
///
/// # Arguments
///
/// * `popcorn_fx` - Mutable reference to the PopcornFX instance.
/// * `preference` - The new subtitle preference to set.
#[no_mangle]
pub extern "C" fn update_subtitle_preference(
    popcorn_fx: &mut PopcornFX,
    preference: &SubtitlePreference,
) {
    trace!("Updating subtitle preference from C for {:?}", preference);
    let preference = preference.clone();
    let subtitle_manager = popcorn_fx.subtitle_manager().clone();
    popcorn_fx.runtime().spawn(async move {
        subtitle_manager.update_preference(preference).await;
    });
}

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
    let subtitles: Vec<SubtitleInfoC> = subtitles.into_iter().map(SubtitleInfoC::from).collect();

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

/// Selects the default subtitle from the given list of subtitles provided in C-compatible form.
///
/// This function retrieves the default subtitle selection from the provided list of subtitles,
/// converts the selected subtitle back into a C-compatible format, and returns a pointer to it.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
/// * `subtitles_ptr` - Pointer to the array of subtitles in C-compatible form.
/// * `len` - The length of the subtitles array.
///
/// # Returns
///
/// A pointer to the selected default subtitle in C-compatible form.
#[no_mangle]
pub extern "C" fn select_or_default_subtitle(
    popcorn_fx: &mut PopcornFX,
    set: &mut SubtitleInfoSet,
) -> *mut SubtitleInfoC {
    trace!("Retrieving default subtitle selection from C for {:?}", set);
    let subtitles: Vec<SubtitleInfo> = from_c_vec(set.subtitles, set.len)
        .into_iter()
        .map(|e| SubtitleInfo::from(e))
        .collect();

    let subtitle_manager = popcorn_fx.subtitle_manager().clone();
    let subtitle_info = block_in_place_runtime(
        subtitle_manager.select_or_default(&subtitles[..]),
        popcorn_fx.runtime(),
    );
    trace!("Default subtitle selection resulted in {:?}", subtitle_info);
    into_c_owned(SubtitleInfoC::from(subtitle_info))
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
pub extern "C" fn register_subtitle_callback(
    popcorn_fx: &mut PopcornFX,
    callback: SubtitleCallbackC,
) {
    trace!("Wrapping C callback for SubtitleCallback");
    let wrapper: SubtitleCallback = Box::new(move |event| {
        let event_c = SubtitleEventC::from(event);
        trace!("Invoking SubtitleEventC {:?}", event_c);
        callback(event_c)
    });

    popcorn_fx.subtitle_manager().add_callback(wrapper);
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
    let subtitle_manager = popcorn_fx.subtitle_manager().clone();
    popcorn_fx.runtime().spawn(async move {
        subtitle_manager.cleanup().await;
    });
}

/// Frees the memory allocated for the `SubtitleInfoSet` structure.
///
/// # Safety
///
/// This function is marked as `unsafe` because it's assumed that the `SubtitleInfoSet` structure was allocated using `Box`,
/// and dropping a `Box` pointing to valid memory is safe. However, if the `SubtitleInfoSet` was allocated in a different way
/// or if the memory was already deallocated, calling this function could lead to undefined behavior.
#[no_mangle]
pub extern "C" fn dispose_subtitle_info_set(set: Box<SubtitleInfoSet>) {
    trace!("Disposing subtitle info set C for {:?}", set);
    drop(set);
}

/// Frees the memory allocated for the `SubtitleInfoC` structure.
///
/// # Safety
///
/// This function is marked as `unsafe` because it's assumed that the `SubtitleInfoC` structure was allocated using `Box`,
/// and dropping a `Box` pointing to valid memory is safe. However, if the `SubtitleInfoC` was allocated in a different way
/// or if the memory was already deallocated, calling this function could lead to undefined behavior.
#[no_mangle]
pub extern "C" fn dispose_subtitle_info(info: Box<SubtitleInfoC>) {
    trace!("Disposing subtitle info C {:?}", info);
    drop(info);
}

/// Frees the memory allocated for the `SubtitleC` structure.
///
/// # Safety
///
/// This function is marked as `unsafe` because it's assumed that the `SubtitleC` structure was allocated using `Box`,
/// and dropping a `Box` pointing to valid memory is safe. However, if the `SubtitleC` was allocated in a different way
/// or if the memory was already deallocated, calling this function could lead to undefined behavior.
#[no_mangle]
pub extern "C" fn dispose_subtitle(subtitle: Box<SubtitleC>) {
    trace!("Disposing subtitle C {:?}", subtitle);
    drop(subtitle)
}

/// Frees the memory allocated for the `SubtitlePreference` structure.
///
/// # Safety
///
/// This function is marked as `unsafe` because it's assumed that the `SubtitlePreference` structure was allocated using `Box`,
/// and dropping a `Box` pointing to valid memory is safe. However, if the `SubtitlePreference` was allocated in a different way
/// or if the memory was already deallocated, calling this function could lead to undefined behavior.
///
#[no_mangle]
pub extern "C" fn dispose_subtitle_preference(subtitle_preference: Box<SubtitlePreference>) {
    trace!("Disposing subtitle preference C {:?}", subtitle_preference);
    drop(subtitle_preference)
}

#[cfg(test)]
mod test {
    use log::info;
    use std::path::PathBuf;
    use std::time::Duration;
    use tempfile::tempdir;

    use popcorn_fx_core::core::subtitles::cue::{StyledText, SubtitleCue, SubtitleLine};
    use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;
    use popcorn_fx_core::core::subtitles::model::Subtitle;
    use popcorn_fx_core::core::subtitles::SubtitleFile;
    use popcorn_fx_core::testing::copy_test_file;
    use popcorn_fx_core::{assert_timeout, from_c_owned, from_c_vec, init_logger};

    use crate::test::new_instance;

    use super::*;

    #[no_mangle]
    pub extern "C" fn subtitle_callback(event: SubtitleEventC) {
        info!("Received subtitle callback event {:?}", event)
    }

    #[test]
    fn test_retrieve_subtitle_preference() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let preference = SubtitlePreference::Language(SubtitleLanguage::Danish);
        let mut instance = new_instance(temp_path);

        let subtitle_manager = instance.subtitle_manager().clone();
        block_in_place_runtime(
            subtitle_manager.update_preference(preference.clone()),
            instance.runtime(),
        );

        let result = retrieve_subtitle_preference(&mut instance);

        assert!(!result.is_null(), "expected a preference to be returned");
        assert_eq!(preference, from_c_owned(result));
    }

    #[test]
    fn test_update_subtitle_preference() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let preference = SubtitlePreference::Language(SubtitleLanguage::French);
        let mut instance = new_instance(temp_path);

        update_subtitle_preference(&mut instance, &preference);

        let subtitle_manager = instance.subtitle_manager().clone();

        block_in_place_runtime(
            async {
                assert_timeout!(
                    Duration::from_millis(500),
                    subtitle_manager.preference().await == preference,
                    "expected the subtitle preference to have been updated"
                )
            },
            instance.runtime(),
        );
    }

    #[test]
    fn test_default_subtitle_options() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = new_instance(temp_path);
        let expected_result = vec![SubtitleInfo::none(), SubtitleInfo::custom()];

        let set_ptr = from_c_owned(default_subtitle_options(&mut instance));
        let result: Vec<SubtitleInfo> = from_c_vec(set_ptr.subtitles, set_ptr.len)
            .into_iter()
            .map(SubtitleInfo::from)
            .collect();

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_subtitle_none() {
        init_logger!();

        let result = from_c_owned(subtitle_none());

        assert_eq!(SubtitleLanguage::None, result.language)
    }

    #[test]
    fn test_subtitle_custom() {
        init_logger!();

        let result = from_c_owned(subtitle_custom());

        assert_eq!(SubtitleLanguage::Custom, result.language)
    }

    #[test]
    fn test_register_subtitle_callback() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = new_instance(temp_path);

        register_subtitle_callback(&mut instance, subtitle_callback);

        let subtitle_manager = instance.subtitle_manager().clone();
        block_in_place_runtime(
            subtitle_manager
                .update_preference(SubtitlePreference::Language(SubtitleLanguage::Finnish)),
            instance.runtime(),
        );
    }

    #[test]
    fn test_cleanup_subtitles_directory() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = new_instance(temp_path);
        let settings = instance.settings().clone();
        let subtitle_directory = block_in_place_runtime(
            settings.user_settings_ref(|e| e.subtitle_settings.directory.clone()),
            instance.runtime(),
        );
        let filepath = copy_test_file(subtitle_directory.as_str(), "example.srt", None);

        cleanup_subtitles_directory(&mut instance);

        block_in_place_runtime(
            async {
                assert_timeout!(
                    Duration::from_millis(500),
                    !PathBuf::from(filepath.clone()).exists(),
                    "expected the subtitle file to have been cleaned"
                )
            },
            instance.runtime(),
        );
    }

    #[test]
    fn test_select_or_default_subtitle() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = new_instance(temp_path);
        let info = SubtitleInfo::builder()
            .imdb_id("tt200002")
            .language(SubtitleLanguage::English)
            .files(vec![SubtitleFile::builder()
                .file_id(1)
                .url("SomeUrl")
                .name("MyFilename")
                .score(0.1)
                .downloads(20)
                .build()])
            .build();
        let mut set = SubtitleInfoSet::from(vec![SubtitleInfoC::from(info.clone())]);

        let result = from_c_owned(select_or_default_subtitle(&mut instance, &mut set));

        assert_eq!(info, SubtitleInfo::from(result));
    }

    #[test]
    fn test_dispose_subtitle_info_set() {
        init_logger!();
        let set = SubtitleInfoSet::from(vec![
            SubtitleInfoC::from(SubtitleInfo::none()),
            SubtitleInfoC::from(SubtitleInfo::custom()),
        ]);

        dispose_subtitle_info_set(Box::new(set));
    }

    #[test]
    fn test_dispose_subtitle_info() {
        init_logger!();
        let info = from_c_owned(subtitle_none());

        dispose_subtitle_info(Box::new(info));
    }

    #[test]
    fn test_dispose_subtitle() {
        let subtitle = Subtitle::new(
            vec![SubtitleCue::new(
                "012".to_string(),
                10000,
                20000,
                vec![SubtitleLine::new(vec![StyledText::new(
                    "Lorem ipsum dolor".to_string(),
                    true,
                    false,
                    false,
                )])],
            )],
            Some(
                SubtitleInfo::builder()
                    .imdb_id("tt00001")
                    .language(SubtitleLanguage::English)
                    .build(),
            ),
            "lorem.srt".to_string(),
        );
        let subtitle_c = SubtitleC::from(subtitle);

        dispose_subtitle(Box::new(subtitle_c))
    }

    #[test]
    fn test_dispose_subtitle_preference() {
        init_logger!();
        let preference = SubtitlePreference::Language(SubtitleLanguage::Finnish);

        dispose_subtitle_preference(Box::new(preference));
    }
}
