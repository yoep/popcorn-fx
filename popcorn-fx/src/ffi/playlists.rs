use crate::ffi::{CArray, PlaylistItemC, PlaylistManagerCallbackC, PlaylistManagerEventC};
use crate::PopcornFX;
use log::{trace, warn};
use popcorn_fx_core::core::block_in_place_runtime;
use popcorn_fx_core::core::playlist::{Playlist, PlaylistItem};
use popcorn_fx_core::{from_c_vec, into_c_owned};
use std::ptr;

/// Play a playlist from C by converting it to the Rust data structure and starting playback asynchronously.
///
/// This function takes a mutable reference to a `PopcornFX` instance and a C-compatible array of `PlaylistItemC` items.
/// It converts the C array into a Rust `Playlist` and starts playback asynchronously using the playlist manager.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
/// * `playlist` - A C-compatible array of `PlaylistItemC` items representing the playlist to play.
///
/// # Returns
///
/// If the playlist playback is successfully started, a pointer to the internal playlist handle is returned.
/// Otherwise, if an error occurs or the playlist is empty, a null pointer is returned.
#[no_mangle]
pub extern "C" fn play_playlist(
    popcorn_fx: &mut PopcornFX,
    playlist_c: &CArray<PlaylistItemC>,
) -> *const i64 {
    trace!("Converting playlist from C for {:?}", playlist_c);
    let playlist: Playlist = Vec::<PlaylistItemC>::from(playlist_c)
        .into_iter()
        .map(|e| PlaylistItem::from(e))
        .collect();

    trace!("Starting playlist from C for {:?}", playlist);
    block_in_place_runtime(
        async {
            popcorn_fx
                .playlist_manager()
                .play(playlist)
                .await
                .map(|e| e.value() as *const i64)
                .unwrap_or_else(|| {
                    warn!("Failed to start playlist from C");
                    ptr::null()
                })
        },
        popcorn_fx.runtime(),
    )
}

/// Play the next item in the playlist from C.
///
/// This function is exposed as a C-compatible function and is intended to be called from C or other languages.
/// It takes a mutable reference to a `PopcornFX` instance and attempts to start playback of the next item in the playlist managed by the `PlaylistManager`.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
///
/// # Returns
///
/// A raw pointer to an `i64` representing the handle of the playlist item if playback was successfully started;
/// otherwise, a null pointer if there are no more items to play or if an error occurred during playback initiation.
#[no_mangle]
pub extern "C" fn play_next_playlist_item(popcorn_fx: &mut PopcornFX) -> *const i64 {
    trace!("Playing next item in playlist from C");
    block_in_place_runtime(
        async {
            popcorn_fx
                .playlist_manager()
                .play_next()
                .await
                .map(|e| e.value() as *const i64)
                .unwrap_or(ptr::null())
        },
        popcorn_fx.runtime(),
    )
}

/// Stop the playback of the current playlist from C.
///
/// This function is exposed as a C-compatible function and is intended to be called from C or other languages.
/// It takes a mutable reference to a `PopcornFX` instance and stops the playback of the currently playing item in the playlist.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the `PopcornFX` instance.
#[no_mangle]
pub extern "C" fn stop_playlist(popcorn_fx: &mut PopcornFX) {
    trace!("Stopping current playlist from C");
    let runtime = popcorn_fx.runtime();
    block_in_place_runtime(popcorn_fx.playlist_manager().stop(), runtime)
}

/// Registers a C-compatible callback function to receive playlist manager events.
///
/// This function is exposed as a C-compatible function and is intended to be called from C or other languages.
/// It takes a mutable reference to a `PopcornFX` instance and a C-compatible callback function as arguments.
///
/// The function registers the provided callback function with the `PlaylistManager` from the `PopcornFX` instance.
/// When a playlist manager event occurs, the callback function is invoked with the corresponding C-compatible event data.
///
/// # Safety
///
/// This function is marked as `unsafe` because it interacts with C-compatible code and dereferences raw pointers.
/// Users of this function should ensure that they provide a valid `PopcornFX` instance and a valid `PlaylistManagerCallbackC`.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the `PopcornFX` instance.
/// * `callback` - The C-compatible callback function to be registered.
#[no_mangle]
pub extern "C" fn register_playlist_manager_callback(
    popcorn_fx: &mut PopcornFX,
    callback: PlaylistManagerCallbackC,
) {
    trace!("Registering new C callback for playlist manager events");
    popcorn_fx
        .playlist_manager()
        .subscribe(Box::new(move |event| {
            trace!("Invoking playlist manager C event for {:?}", event);
            let event = PlaylistManagerEventC::from(event);
            callback(event);
        }));
}

/// Retrieves the playlist from PopcornFX.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to the PopcornFX instance.
///
/// # Returns
///
/// A CArray of PlaylistItemC representing the playlist.
#[no_mangle]
pub extern "C" fn playlist(popcorn_fx: &mut PopcornFX) -> *mut CArray<PlaylistItemC> {
    trace!("Retrieving playlist from C");
    block_in_place_runtime(
        async {
            let vec: Vec<PlaylistItemC> = popcorn_fx
                .playlist_manager()
                .playlist()
                .await
                .items
                .into_iter()
                .map(|e| PlaylistItemC::from(e))
                .collect();
            into_c_owned(CArray::from(vec))
        },
        popcorn_fx.runtime(),
    )
}

/// Dispose of a playlist item.
///
/// # Arguments
///
/// * `item` - A boxed `PlaylistItemC` representing the item to be disposed of.
#[no_mangle]
pub extern "C" fn dispose_playlist_item(item: Box<PlaylistItemC>) {
    trace!("Disposing playlist item {:?}", item);
    drop(item);
}

/// Dispose of a C-style array of playlist items.
///
/// This function takes ownership of a C-style array of `PlaylistItemC` and drops it to free the associated memory.
///
/// # Arguments
///
/// * `set` - A boxed C-style array of `PlaylistItemC` to be disposed of.
#[no_mangle]
pub extern "C" fn dispose_playlist_set(set: Box<CArray<PlaylistItemC>>) {
    trace!("Disposing playlist set {:?}", set);
    drop(from_c_vec(set.items, set.len));
}

/// Dispose of a C-compatible PlaylistManagerEventC value.
///
/// This function is responsible for cleaning up resources associated with a C-compatible PlaylistManagerEventC value.
///
/// # Arguments
///
/// * `event` - A C-compatible PlaylistManagerEventC value to be disposed of.
#[no_mangle]
pub extern "C" fn dispose_playlist_manager_event_value(event: PlaylistManagerEventC) {
    trace!("Disposing PlaylistManagerEventC {:?}", event);
    drop(event);
}

#[cfg(test)]
mod test {
    use std::ptr;
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use tempfile::tempdir;

    use crate::test::default_args;
    use popcorn_fx_core::core::loader::MediaLoaderEvent;
    use popcorn_fx_core::core::playlist::{PlaylistManagerEvent, PlaylistState};
    use popcorn_fx_core::{init_logger, into_c_owned, into_c_string};

    use super::*;

    #[test]
    fn test_play_playlist() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let item = PlaylistItemC::from(PlaylistItem {
            url: Some("http://localhost:9870/my-video.mkv".to_string()),
            title: "MyPlaylistItem".to_string(),
            caption: Some("MyCaption".to_string()),
            thumb: Some("http://localhost:9870/my-thumb.png".to_string()),
            media: Default::default(),
            quality: None,
            auto_resume_timestamp: None,
            subtitle: Default::default(),
            torrent: Default::default(),
        });
        let mut playlist = CArray::from(vec![item]);
        let (tx, rx) = channel();
        let (tx_state, rx_state) = channel();
        let mut instance = PopcornFX::new(default_args(temp_path));

        instance
            .playlist_manager()
            .subscribe(Box::new(move |e| match e {
                PlaylistManagerEvent::PlaylistChanged => tx.send(e).unwrap(),
                PlaylistManagerEvent::StateChanged(state) => tx_state.send(state).unwrap(),
                _ => {}
            }));
        play_playlist(&mut instance, &mut playlist);

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(
            PlaylistManagerEvent::PlaylistChanged,
            result,
            "expected the PlaylistChanged event to have been published"
        );

        let result = rx_state.recv_timeout(Duration::from_millis(200)).unwrap();
        assert_eq!(
            PlaylistState::Playing,
            result,
            "expected the state to have changed"
        );
    }

    #[test]
    fn test_play_next_playlist_item() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut playlist = CArray::from(vec![
            PlaylistItemC::from(PlaylistItem {
                url: None,
                title: "Item1".to_string(),
                caption: None,
                thumb: None,
                media: Default::default(),
                quality: None,
                auto_resume_timestamp: None,
                subtitle: Default::default(),
                torrent: Default::default(),
            }),
            PlaylistItemC::from(PlaylistItem {
                url: None,
                title: "Item2".to_string(),
                caption: None,
                thumb: None,
                media: Default::default(),
                quality: None,
                auto_resume_timestamp: None,
                subtitle: Default::default(),
                torrent: Default::default(),
            }),
        ]);
        let mut instance = PopcornFX::new(default_args(temp_path));

        play_playlist(&mut instance, &mut playlist);
        let handle = play_next_playlist_item(&mut instance);
        assert!(
            !handle.is_null(),
            "expected a valid loader handle to have been returned"
        );
    }

    // TODO: fix timeout when a certain struct is being dropped at the end of the test
    // #[test]
    fn test_stop_playlist() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let (tx, rx) = channel();
        let mut instance = PopcornFX::new(default_args(temp_path));
        let runtime = instance.runtime();

        let mut receiver = instance.media_loader().subscribe();
        runtime.spawn(async move {
            loop {
                if let Some(event) = receiver.recv().await {
                    if let MediaLoaderEvent::LoadingStarted(handle, _) = &*event {
                        tx.send(*handle).unwrap();
                        break;
                    }
                } else {
                    break;
                }
            }
        });

        // start the playback
        let result = runtime.block_on(instance.playlist_manager().play(Playlist::from_iter(vec![
            PlaylistItem {
                url: None,
                title: "Item1".to_string(),
                caption: None,
                thumb: None,
                media: Default::default(),
                quality: None,
                auto_resume_timestamp: None,
                subtitle: Default::default(),
                torrent: Default::default(),
            },
            PlaylistItem {
                url: None,
                title: "Item2".to_string(),
                caption: None,
                thumb: None,
                media: Default::default(),
                quality: None,
                auto_resume_timestamp: None,
                subtitle: Default::default(),
                torrent: Default::default(),
            },
        ])));
        // check the loading handle
        match result {
            Some(handle) => {
                let loading_handle = rx
                    .recv_timeout(Duration::from_millis(500))
                    .expect("expected the playback to have been started");
                assert_eq!(
                    handle, loading_handle,
                    "expected the currently loading handle to have been te same"
                );
                // cancel the loading task
                instance.media_loader().cancel(handle);
            }
            None => assert!(false, "expected the playback to have been started"),
        }

        stop_playlist(&mut instance);

        let result = instance
            .runtime()
            .block_on(instance.playlist_manager().has_next());
        assert_eq!(false, result, "expected the playlist to be empty");
    }

    #[test]
    fn test_dispose_playlist_item() {
        init_logger!();
        let item = Box::new(PlaylistItemC {
            url: into_c_string("http://my_url".to_string()),
            title: into_c_string("Foo Bar".to_string()),
            caption: ptr::null_mut(),
            thumb: into_c_string("MyThumb".to_string()),
            parent_media: ptr::null_mut(),
            media: ptr::null_mut(),
            quality: ptr::null_mut(),
            auto_resume_timestamp: ptr::null_mut(),
            subtitles_enabled: false,
            subtitle_info: ptr::null_mut(),
            torrent_filename: ptr::null_mut(),
        });

        dispose_playlist_item(item);
    }

    #[test]
    fn test_dispose_playlist_set() {
        init_logger!();
        let item = PlaylistItemC {
            url: into_c_string("http://my_url".to_string()),
            title: into_c_string("Foo Bar".to_string()),
            caption: ptr::null_mut(),
            thumb: into_c_string("MyThumb".to_string()),
            parent_media: ptr::null_mut(),
            media: ptr::null_mut(),
            quality: ptr::null_mut(),
            auto_resume_timestamp: into_c_owned(500u64),
            subtitles_enabled: false,
            subtitle_info: ptr::null_mut(),
            torrent_filename: ptr::null_mut(),
        };
        let playlist = CArray::<PlaylistItemC>::from(vec![item]);

        dispose_playlist_set(Box::new(playlist));
    }
}
