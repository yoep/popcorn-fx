extern crate core;

use fx_callback::Callback;
use log::{debug, error, info, trace, warn};
use std::os::raw::c_char;
use std::{mem, ptr};

#[cfg(feature = "ffi")]
use crate::ffi::*;
pub use fx::*;
use popcorn_fx_core::core::block_in_place_runtime;
use popcorn_fx_core::core::config::{
    PlaybackSettings, ServerSettings, SubtitleSettings, TorrentSettings, UiSettings,
};
use popcorn_fx_core::core::media::watched::WatchedCallback;
use popcorn_fx_core::core::media::*;
use popcorn_fx_core::core::subtitles::matcher::SubtitleMatcher;
use popcorn_fx_core::core::subtitles::model::SubtitleInfo;
use popcorn_fx_core::{
    from_c_into_boxed, from_c_owned, from_c_string, into_c_owned, into_c_string,
};

#[cfg(feature = "ffi")]
pub mod ffi;
mod fx;

/// Retrieve the available subtitles for the given [MovieDetailsC].
///
/// This function takes a reference to the `PopcornFX` instance and a reference to a `MovieDetailsC`.
/// It returns a reference to `SubtitleInfoSet` containing the available subtitles for the movie,
/// or a null pointer (`ptr::null_mut()`) on failure.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the `PopcornFX` instance.
/// * `movie` - A reference to the `MovieDetailsC` for which subtitles are to be retrieved.
///
/// # Returns
///
/// A pointer to the `SubtitleInfoSet` containing the available subtitles, or a null pointer on failure.
/// <i>The returned reference should be managed by the caller.</i>
#[no_mangle]
pub extern "C" fn movie_subtitles(
    popcorn_fx: &mut PopcornFX,
    movie: &MovieDetailsC,
) -> *mut SubtitleInfoSet {
    let movie_instance = MovieDetails::from(movie);

    match popcorn_fx.runtime().block_on(
        popcorn_fx
            .subtitle_provider()
            .movie_subtitles(&movie_instance),
    ) {
        Ok(e) => {
            debug!("Found movie subtitles {:?}", e);
            let result: Vec<SubtitleInfoC> =
                e.into_iter().map(|e| SubtitleInfoC::from(e)).collect();

            into_c_owned(SubtitleInfoSet::from(result))
        }
        Err(e) => {
            error!("Movie subtitle search failed, {}", e);
            ptr::null_mut()
        }
    }
}

/// Retrieve the given subtitles for the given episode.
///
/// This function takes a reference to the `PopcornFX` instance, a reference to a `ShowDetailsC`, and a reference
/// to an `EpisodeC` for which subtitles are to be retrieved.
/// It returns a reference to `SubtitleInfoSet` containing the available subtitles for the episode.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the `PopcornFX` instance.
/// * `show` - A reference to the `ShowDetailsC` containing information about the show.
/// * `episode` - A reference to the `EpisodeC` for which subtitles are to be retrieved.
///
/// # Returns
///
/// A pointer to the `SubtitleInfoSet` containing the available subtitles for the episode.
/// <i>The returned reference should be managed by the caller.</i>
#[no_mangle]
pub extern "C" fn episode_subtitles(
    popcorn_fx: &mut PopcornFX,
    show: &ShowDetailsC,
    episode: &EpisodeC,
) -> *mut SubtitleInfoSet {
    let show_instance = show.to_struct();
    let episode_instance = Episode::from(episode);

    match popcorn_fx.runtime().block_on(
        popcorn_fx
            .subtitle_provider()
            .episode_subtitles(&show_instance, &episode_instance),
    ) {
        Ok(e) => {
            debug!("Found episode subtitles {:?}", e);
            let result: Vec<SubtitleInfoC> =
                e.into_iter().map(|e| SubtitleInfoC::from(e)).collect();

            into_c_owned(SubtitleInfoSet::from(result))
        }
        Err(e) => {
            error!("Episode subtitle search failed, {}", e);
            into_c_owned(SubtitleInfoSet::from(vec![]))
        }
    }
}

/// Retrieve the available subtitles for the given filename
#[no_mangle]
pub extern "C" fn filename_subtitles(
    popcorn_fx: &mut PopcornFX,
    filename: *mut c_char,
) -> *mut SubtitleInfoSet {
    let filename_rust = from_c_string(filename);

    match popcorn_fx.runtime().block_on(
        popcorn_fx
            .subtitle_provider()
            .file_subtitles(&filename_rust),
    ) {
        Ok(e) => {
            debug!("Found filename subtitles {:?}", e);
            let result: Vec<SubtitleInfoC> =
                e.into_iter().map(|e| SubtitleInfoC::from(e)).collect();

            into_c_owned(SubtitleInfoSet::from(result))
        }
        Err(e) => {
            error!("Filename subtitle search failed, {}", e);
            into_c_owned(SubtitleInfoSet::from(vec![]))
        }
    }
}

/// Reset the current preferred subtitle configuration.
/// This will remove any selected [SubtitleInfo] or custom subtitle file.
#[no_mangle]
pub extern "C" fn reset_subtitle(popcorn_fx: &mut PopcornFX) {
    let subtitle_manager = popcorn_fx.subtitle_manager().clone();
    popcorn_fx.runtime().spawn(async move {
        subtitle_manager.reset().await;
    });
}

/// Download the given [SubtitleInfo] based on the best match according to the [SubtitleMatcher].
///
/// It returns the filepath to the subtitle on success, else [ptr::null_mut].
#[no_mangle]
pub extern "C" fn download(
    popcorn_fx: &mut PopcornFX,
    subtitle: &SubtitleInfoC,
    matcher: &SubtitleMatcherC,
) -> *mut c_char {
    trace!(
        "Starting subtitle download from C for info: {:?}, matcher: {:?}",
        subtitle,
        matcher
    );
    let subtitle_info = SubtitleInfo::from(subtitle);
    let matcher = SubtitleMatcher::from(matcher);

    match popcorn_fx.runtime().block_on(
        popcorn_fx
            .subtitle_provider()
            .download(&subtitle_info, &matcher),
    ) {
        Ok(e) => {
            debug!("Returning subtitle filepath {:?}", &e);
            into_c_string(e)
        }
        Err(e) => {
            error!("Failed to download subtitle, {}", e);
            ptr::null_mut()
        }
    }
}

/// Download and parse the given subtitle info.
///
/// It returns the [SubtitleC] reference on success, else [ptr::null_mut].
#[no_mangle]
pub extern "C" fn download_and_parse_subtitle(
    popcorn_fx: &mut PopcornFX,
    subtitle: &SubtitleInfoC,
    matcher: &SubtitleMatcherC,
) -> *mut SubtitleC {
    trace!(
        "Downloading and parsing subtitle from C for info: {:?}, matcher: {:?}",
        subtitle,
        matcher
    );
    let subtitle_info = SubtitleInfo::from(subtitle);
    let matcher = SubtitleMatcher::from(matcher);

    match popcorn_fx.runtime().block_on(
        popcorn_fx
            .subtitle_provider()
            .download_and_parse(&subtitle_info, &matcher),
    ) {
        Ok(e) => {
            let result = SubtitleC::from(e);
            debug!("Returning parsed subtitle {:?}", result);
            into_c_owned(result)
        }
        Err(e) => {
            error!("Failed to download subtitle, {}", e);
            ptr::null_mut()
        }
    }
}

/// Reset all available api stats for the movie api.
/// This will make all disabled api's available again.
#[no_mangle]
pub extern "C" fn reset_show_apis(popcorn_fx: &mut PopcornFX) {
    let providers = popcorn_fx.providers().clone();
    popcorn_fx
        .runtime()
        .spawn(async move { providers.reset_api(&Category::Series).await });
}

/// Verify if the given media item is liked/favorite of the user.
/// It will use the first non [ptr::null_mut] field from the [MediaItemC] struct.
///
/// It will return false if all fields in the [MediaItemC] are [ptr::null_mut].
#[no_mangle]
pub extern "C" fn is_media_liked(popcorn_fx: &mut PopcornFX, favorite: &mut MediaItemC) -> bool {
    trace!("Verifying if media is liked for {:?}", favorite);
    match favorite.as_identifier() {
        None => {
            warn!("Unable to verify if media is liked, all FavoriteC fields are null");
            false
        }
        Some(media) => {
            let favorite_service = popcorn_fx.favorite_service().clone();
            let liked =
                block_in_place_runtime(favorite_service.is_liked_dyn(&media), popcorn_fx.runtime());
            trace!(
                "Liked state is {} for {} {}",
                &liked,
                media.media_type(),
                media.imdb_id()
            );
            mem::forget(media);
            liked
        }
    }
}

/// Retrieve all favorites of the user.
///
/// It will return an array of favorites on success, else [ptr::null_mut].
#[no_mangle]
pub extern "C" fn retrieve_all_favorites(popcorn_fx: &mut PopcornFX) -> *mut VecFavoritesC {
    let favorite_service = popcorn_fx.favorite_service().clone();
    match block_in_place_runtime(favorite_service.all(), popcorn_fx.runtime()) {
        Ok(e) => favorites_to_c(e),
        Err(e) => {
            error!("Failed to retrieve favorites, {}", e);
            ptr::null_mut()
        }
    }
}

/// Add the media item to the favorites.
/// Duplicate favorite media items are ignored.
#[no_mangle]
pub extern "C" fn add_to_favorites(popcorn_fx: &mut PopcornFX, favorite: &MediaItemC) {
    let media: Box<dyn MediaIdentifier>;

    if !favorite.movie_overview.is_null() {
        let boxed = from_c_into_boxed(favorite.movie_overview);
        media = Box::new(boxed.to_struct());
        trace!("Created media struct {:?}", media);
        mem::forget(boxed);
    } else if !favorite.movie_details.is_null() {
        let boxed = from_c_into_boxed(favorite.movie_details);
        let details = MovieDetails::from(&*boxed);
        media = Box::new(details.to_overview());
        trace!("Created media struct {:?}", media);
        mem::forget(boxed);
    } else if !favorite.show_overview.is_null() {
        let boxed = from_c_into_boxed(favorite.show_overview);
        media = Box::new(boxed.to_struct());
        trace!("Created media struct {:?}", media);
        mem::forget(boxed);
    } else if !favorite.show_details.is_null() {
        let boxed = from_c_into_boxed(favorite.show_details);
        let details = Box::new(boxed.to_struct());
        media = Box::new(details.to_overview());
        trace!("Created media struct {:?}", media);
        mem::forget(boxed);
    } else {
        error!("Unable to add favorite, all FavoriteC fields are null");
        return;
    }

    let favorite_service = popcorn_fx.favorite_service().clone();
    if let Err(e) = block_in_place_runtime(favorite_service.add(media), popcorn_fx.runtime()) {
        error!("{}", e);
    }
}

/// Remove the media item from favorites.
#[no_mangle]
pub extern "C" fn remove_from_favorites(popcorn_fx: &mut PopcornFX, favorite: &MediaItemC) {
    match favorite.as_identifier() {
        None => error!("Unable to remove favorite, all FavoriteC fields are null"),
        Some(e) => {
            let favorite_service = popcorn_fx.favorite_service().clone();
            block_in_place_runtime(favorite_service.remove(e), popcorn_fx.runtime());
        }
    }
}

/// Register a new callback listener for favorite events.
#[no_mangle]
pub extern "C" fn register_favorites_event_callback<'a>(
    popcorn_fx: &mut PopcornFX,
    callback: extern "C" fn(FavoriteEventC),
) {
    trace!("Wrapping C callback for FavoriteCallback");
    let mut receiver = popcorn_fx.favorite_service().subscribe();
    popcorn_fx.runtime().spawn(async move {
        while let Some(event) = receiver.recv().await {
            callback(FavoriteEventC::from((*event).clone()));
        }
    });
}

/// Verify if the given media item is watched by the user.
///
/// It returns true when the item is watched, else false.
#[no_mangle]
pub extern "C" fn is_media_watched(popcorn_fx: &mut PopcornFX, watchable: &MediaItemC) -> bool {
    match watchable.as_identifier() {
        Some(media) => {
            let media_id = media.to_string();
            trace!("Verifying if media item is watched for {}", media_id);
            let watched = popcorn_fx.watched_service().is_watched_dyn(&media);
            mem::forget(media);
            trace!("Retrieved watched state {} for {}", &watched, media_id);
            watched
        }
        None => {
            error!("Failed to verify the watched state, no watchable item given");
            false
        }
    }
}

/// Retrieve all watched media item id's.
///
/// It returns an array of watched id's.
#[no_mangle]
pub extern "C" fn retrieve_all_watched(popcorn_fx: &mut PopcornFX) -> StringArray {
    trace!("Retrieving all watched media id's");
    match popcorn_fx.watched_service().all() {
        Ok(e) => {
            debug!("Retrieved watched items {:?}", &e);
            StringArray::from(e)
        }
        Err(e) => {
            error!("Failed to retrieve watched items, {}", e);
            StringArray::from(vec![])
        }
    }
}

/// Retrieve all watched movie id's.
///
/// It returns an array of watched movie id's.
#[no_mangle]
pub extern "C" fn retrieve_watched_movies(popcorn_fx: &mut PopcornFX) -> *mut StringArray {
    match popcorn_fx.watched_service().watched_movies() {
        Ok(e) => {
            debug!("Retrieved watched items {:?}", &e);
            into_c_owned(StringArray::from(e))
        }
        Err(e) => {
            error!("Failed to retrieve watched items, {}", e);
            into_c_owned(StringArray::from(vec![]))
        }
    }
}

/// Retrieve all watched show media id's.
///
/// It returns  an array of watched show id's.
#[no_mangle]
pub extern "C" fn retrieve_watched_shows(popcorn_fx: &mut PopcornFX) -> *mut StringArray {
    match popcorn_fx.watched_service().watched_shows() {
        Ok(e) => {
            debug!("Retrieved watched items {:?}", &e);
            into_c_owned(StringArray::from(e))
        }
        Err(e) => {
            error!("Failed to retrieve watched items, {}", e);
            into_c_owned(StringArray::from(vec![]))
        }
    }
}

/// Add the given media item to the watched list.
#[no_mangle]
pub extern "C" fn add_to_watched(popcorn_fx: &mut PopcornFX, watchable: &MediaItemC) {
    match watchable.as_identifier() {
        Some(e) => {
            let id = e.imdb_id().to_string();
            match popcorn_fx.watched_service().add(e) {
                Ok(_) => info!("Media item {} as been added as seen", id),
                Err(e) => error!("Failed to add media item {} as watched, {}", id, e),
            };
        }
        None => {
            error!("Unable to add watchable, no media item given")
        }
    }
}

/// Remove the given media item from the watched list.
#[no_mangle]
pub extern "C" fn remove_from_watched(popcorn_fx: &mut PopcornFX, watchable: &MediaItemC) {
    match watchable.as_identifier() {
        Some(e) => popcorn_fx.watched_service().remove(e),
        None => {
            error!("Unable to add watchable, no media item given")
        }
    }
}

/// Register a new callback listener for watched events.
#[no_mangle]
pub extern "C" fn register_watched_event_callback<'a>(
    popcorn_fx: &mut PopcornFX,
    callback: extern "C" fn(WatchedEventC),
) {
    trace!("Wrapping C callback for WatchedCallback");
    let wrapper: WatchedCallback = Box::new(move |event| {
        callback(WatchedEventC::from(event));
    });

    popcorn_fx.watched_service().register(wrapper)
}

/// Verify if the given magnet uri has already been stored.
#[no_mangle]
pub extern "C" fn torrent_collection_is_stored(
    popcorn_fx: &mut PopcornFX,
    magnet_uri: *mut c_char,
) -> bool {
    let magnet_uri = from_c_string(magnet_uri);
    trace!(
        "Checking if magnet uri is stored for {}",
        magnet_uri.as_str()
    );
    let torrent_collection = popcorn_fx.torrent_collection().clone();
    block_in_place_runtime(
        torrent_collection.is_stored(magnet_uri.as_str()),
        popcorn_fx.runtime(),
    )
}

/// Retrieve all stored magnets from the torrent collection.
/// It returns the set on success, else [ptr::null_mut].
#[no_mangle]
pub extern "C" fn torrent_collection_all(popcorn_fx: &mut PopcornFX) -> *mut TorrentCollectionSet {
    trace!("Retrieving torrent collection magnets");
    let torrent_collection = popcorn_fx.torrent_collection().clone();
    let runtime = popcorn_fx.runtime();
    match block_in_place_runtime(torrent_collection.all(), runtime) {
        Ok(e) => {
            let set = TorrentCollectionSet::from(e);
            into_c_owned(set)
        }
        Err(e) => {
            error!("Failed to retrieve magnets, {}", e);
            ptr::null_mut()
        }
    }
}

/// Add the given magnet info to the torrent collection.
#[no_mangle]
pub extern "C" fn torrent_collection_add(
    popcorn_fx: &mut PopcornFX,
    name: *mut c_char,
    magnet_uri: *mut c_char,
) {
    let name = from_c_string(name);
    let magnet_uri = from_c_string(magnet_uri);
    trace!("Adding magnet {} to torrent collection", magnet_uri);

    let torrent_collection = popcorn_fx.torrent_collection().clone();
    block_in_place_runtime(
        torrent_collection.insert(name.as_str(), magnet_uri.as_str()),
        popcorn_fx.runtime(),
    );
}

/// Remove the given magnet uri from the torrent collection.
#[no_mangle]
pub extern "C" fn torrent_collection_remove(popcorn_fx: &mut PopcornFX, magnet_uri: *mut c_char) {
    let magnet_uri = from_c_string(magnet_uri);
    trace!("Removing magnet {} from torrent collection", magnet_uri);

    let torrent_collection = popcorn_fx.torrent_collection().clone();
    block_in_place_runtime(
        torrent_collection.remove(magnet_uri.as_str()),
        popcorn_fx.runtime(),
    );
}

/// Retrieve the application settings.
/// These are the setting preferences of the users for the popcorn FX instance.
#[no_mangle]
pub extern "C" fn application_settings(popcorn_fx: &mut PopcornFX) -> *mut PopcornSettingsC {
    trace!("Retrieving application settings");
    let application_config = popcorn_fx.settings().clone();
    let settings = block_in_place_runtime(application_config.user_settings(), popcorn_fx.runtime());
    into_c_owned(PopcornSettingsC::from(settings))
}

/// Reload the settings of the application.
#[no_mangle]
pub extern "C" fn reload_settings(popcorn_fx: &mut PopcornFX) {
    trace!("Reloading the popcorn fx settings");
    popcorn_fx.reload_settings()
}

/// Register a new callback for all setting events.
#[no_mangle]
pub extern "C" fn register_settings_callback(
    popcorn_fx: &mut PopcornFX,
    callback: ApplicationConfigCallbackC,
) {
    trace!("Registering application settings callback from C");
    let wrapper = Box::new(move |event| {
        let event_c = ApplicationConfigEventC::from(event);
        trace!("Invoking ApplicationConfigEventC {:?}", event_c);
        callback(event_c)
    });

    let mut receiver = popcorn_fx.settings().subscribe();
    popcorn_fx.runtime().spawn(async move {
        while let Some(event) = receiver.recv().await {
            wrapper((*event).clone());
        }
    });
}

/// Update the subtitle settings with the new value.
#[no_mangle]
pub extern "C" fn update_subtitle_settings(
    popcorn_fx: &mut PopcornFX,
    subtitle_settings: SubtitleSettingsC,
) {
    trace!(
        "Updating the subtitle settings from {:?}",
        subtitle_settings
    );
    let subtitle = SubtitleSettings::from(subtitle_settings);
    block_in_place_runtime(
        popcorn_fx.settings().update_subtitle(subtitle),
        popcorn_fx.runtime(),
    );
}

/// Update the torrent settings with the new value.
#[no_mangle]
pub extern "C" fn update_torrent_settings(
    popcorn_fx: &mut PopcornFX,
    torrent_settings: TorrentSettingsC,
) {
    trace!("Updating the torrent settings from {:?}", torrent_settings);
    let settings = TorrentSettings::from(torrent_settings);
    block_in_place_runtime(
        popcorn_fx.settings().update_torrent(settings),
        popcorn_fx.runtime(),
    );
}

/// Update the ui settings with the new value.
#[no_mangle]
pub extern "C" fn update_ui_settings(popcorn_fx: &mut PopcornFX, settings: UiSettingsC) {
    trace!("Updating the ui settings from {:?}", settings);
    let settings = UiSettings::from(settings);
    block_in_place_runtime(
        popcorn_fx.settings().update_ui(settings),
        popcorn_fx.runtime(),
    );
}

/// Update the server settings with the new value.
#[no_mangle]
pub extern "C" fn update_server_settings(popcorn_fx: &mut PopcornFX, settings: ServerSettingsC) {
    trace!("Updating the server settings from {:?}", settings);
    let settings = ServerSettings::from(settings);
    block_in_place_runtime(
        popcorn_fx.settings().update_server(settings),
        popcorn_fx.runtime(),
    );
}

/// Update the playback settings with the new value.
#[no_mangle]
pub extern "C" fn update_playback_settings(
    popcorn_fx: &mut PopcornFX,
    settings: PlaybackSettingsC,
) {
    trace!("Updating the playback settings from {:?}", settings);
    let settings = PlaybackSettings::from(settings);
    block_in_place_runtime(
        popcorn_fx.settings().update_playback(settings),
        popcorn_fx.runtime(),
    );
}

/// Dispose of a C-compatible MediaItemC value wrapped in a Box.
///
/// This function is responsible for cleaning up resources associated with a C-compatible MediaItemC value
/// wrapped in a Box.
///
/// # Arguments
///
/// * `media` - A Box containing a C-compatible MediaItemC value to be disposed of.
#[no_mangle]
pub extern "C" fn dispose_media_item(media: Box<MediaItemC>) {
    trace!("Disposing MediaItemC reference {:?}", media);
    dispose_media_item_value(*media)
}

/// Dispose of a C-compatible MediaItemC value.
///
/// This function is responsible for cleaning up resources associated with a C-compatible MediaItemC value.
///
/// # Arguments
///
/// * `media` - A C-compatible MediaItemC value to be disposed of.
#[no_mangle]
pub extern "C" fn dispose_media_item_value(media: MediaItemC) {
    trace!("Disposing MediaItemC {:?}", media);
    if !media.show_overview.is_null() {
        let _ = from_c_owned(media.show_overview);
    } else if !media.show_details.is_null() {
        let _ = from_c_owned(media.show_details);
    } else if !media.movie_overview.is_null() {
        let _ = from_c_owned(media.movie_overview);
    } else if !media.movie_details.is_null() {
        let _ = from_c_owned(media.movie_details);
    }
}

/// Dispose the [TorrentCollectionSet] from memory.
#[no_mangle]
pub extern "C" fn dispose_torrent_collection(collection_set: Box<TorrentCollectionSet>) {
    trace!("Disposing collection set {:?}", collection_set)
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use tempfile::tempdir;

    use popcorn_fx_core::core::config::{DecorationType, SubtitleFamily};
    use popcorn_fx_core::core::subtitles::language::SubtitleLanguage;
    use popcorn_fx_core::testing::copy_test_file;
    use popcorn_fx_core::{from_c_owned, init_logger};

    use crate::fx::PopcornFxArgs;

    use super::*;

    /// The default set of [PopcornFxArgs] for testing purposes.
    /// This makes it easier to reuse and adopt the args struct when needed without the need to
    /// modify it in each test.
    pub fn default_args(temp_path: &str) -> PopcornFxArgs {
        PopcornFxArgs {
            disable_logger: true,
            disable_mouse: false,
            enable_youtube_video_player: false,
            enable_fx_video_player: false,
            enable_vlc_video_player: false,
            tv: false,
            maximized: false,
            kiosk: false,
            insecure: false,
            app_directory: temp_path.to_string(),
            data_directory: PathBuf::from(temp_path)
                .join("data")
                .to_str()
                .unwrap()
                .to_string(),
            properties: Default::default(),
        }
    }

    pub fn new_instance(temp_path: &str) -> PopcornFX {
        let instance = PopcornFX::new(default_args(temp_path));
        let config = instance.settings();
        let settings = block_in_place_runtime(config.user_settings(), instance.runtime());
        let mut subtitle_settings = settings.subtitle_settings;
        subtitle_settings.directory = PathBuf::from(temp_path)
            .join("subtitles")
            .to_str()
            .unwrap()
            .to_string();
        block_in_place_runtime(
            config.update_subtitle(subtitle_settings),
            instance.runtime(),
        );
        let mut torrent_settings = settings.torrent_settings;
        torrent_settings.directory = PathBuf::from(temp_path).join("torrents");
        block_in_place_runtime(config.update_torrent(torrent_settings), instance.runtime());
        instance
    }

    #[no_mangle]
    pub extern "C" fn settings_callback(_: ApplicationConfigEventC) {}

    #[test]
    fn test_dispose_popcorn_fx() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let instance = PopcornFX::new(default_args(temp_path));

        dispose_popcorn_fx(Box::new(instance));
    }

    #[test]
    fn test_is_liked_movie_overview() {
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));
        let movie = MovieOverview::new(
            "".to_string(),
            "tt0000000122".to_string(),
            "2020".to_string(),
        );
        let mut media = MediaItemC::from(movie);

        let result = is_media_liked(&mut instance, &mut media);

        assert_eq!(false, result)
    }

    #[test]
    fn test_is_liked_movie_details() {
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));
        let movie = MovieDetails::new(
            "".to_string(),
            "tt0000000111".to_string(),
            "2020".to_string(),
        );
        let mut media = MediaItemC::from(movie);

        let result = is_media_liked(&mut instance, &mut media);

        assert_eq!(false, result)
    }

    #[test]
    fn test_torrent_collection_is_stored() {
        let magnet_uri = "magnet:?MagnetA";
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));
        copy_test_file(temp_path, "torrent-collection.json", None);

        let result =
            torrent_collection_is_stored(&mut instance, into_c_string(magnet_uri.to_string()));

        assert_eq!(true, result)
    }

    #[test]
    fn test_torrent_collection_all() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));
        copy_test_file(temp_path, "torrent-collection.json", None);

        let result = from_c_owned(torrent_collection_all(&mut instance));

        assert_eq!(1, result.len)
    }

    #[test]
    fn test_register_settings_callback() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let subtitle_c = SubtitleSettingsC::from(&SubtitleSettings::new(
            Some(temp_path.to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        ));
        let mut instance = PopcornFX::new(default_args(temp_path));

        register_settings_callback(&mut instance, settings_callback);
        update_subtitle_settings(&mut instance, subtitle_c);
    }

    #[test]
    fn test_update_subtitle_settings() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = PopcornFX::new(default_args(temp_path));
        let settings = SubtitleSettings {
            directory: format!("{}/subtitles", temp_path),
            auto_cleaning_enabled: false,
            default_subtitle: SubtitleLanguage::German,
            font_family: SubtitleFamily::Arial,
            font_size: 32,
            decoration: DecorationType::SeeThroughBackground,
            bold: true,
        };

        update_subtitle_settings(&mut instance, SubtitleSettingsC::from(&settings));
        let config = instance.settings().clone();

        let result = block_in_place_runtime(
            config.user_settings_ref(|e| e.subtitle().clone()),
            instance.runtime(),
        );

        assert_eq!(settings, result)
    }

    #[test]
    fn test_dispose_media_item() {
        let movie = MovieOverview::new(String::new(), String::from("tt54698542"), String::new());
        let media = MediaItemC::from(movie);

        dispose_media_item(Box::new(media));
    }
}
