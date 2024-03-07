use std::fmt::{Debug, Formatter};
use std::os::raw::c_char;
use std::ptr;
use std::sync::{Arc, Weak};

use async_trait::async_trait;
use derive_more::Display;
use log::trace;
use tokio::sync::Mutex;

use popcorn_fx_core::{from_c_string, from_c_vec, into_c_owned, into_c_string, into_c_vec};
use popcorn_fx_core::core::{block_in_place, CallbackHandle, Callbacks, CoreCallback, CoreCallbacks};
use popcorn_fx_core::core::players::{Player, PlayerEvent, PlayerManagerEvent, PlayerState, PlayMediaRequest, PlayRequest, PlayUrlRequest};

use crate::ffi::PlayerChangedEventC;

/// A C-compatible callback function type for player manager events.
pub type PlayerManagerEventCallback = extern "C" fn(PlayerManagerEventC);

/// A C-compatible callback function type for player play events.
pub type PlayerPlayCallback = extern "C" fn(PlayRequestC);

/// A C-compatible callback function type for player pause events.
pub type PlayerPauseCallback = extern "C" fn();

/// A C-compatible callback function type for player resume events.
pub type PlayerResumeCallback = extern "C" fn();

/// A C-compatible callback function type for player seek events.
pub type PlayerSeekCallback = extern "C" fn(u64);

/// A C-compatible callback function type for player stop events.
pub type PlayerStopCallback = extern "C" fn();

/// A C-compatible enum representing player events.
#[repr(C)]
#[derive(Debug)]
pub enum PlayerEventC {
    DurationChanged(u64),
    TimeChanged(u64),
    StateChanged(PlayerState),
    VolumeChanged(u32),
}

impl From<PlayerEventC> for PlayerEvent {
    fn from(value: PlayerEventC) -> Self {
        trace!("Converting PlayerEventC into PlayerEvent for {:?}", value);
        match value {
            PlayerEventC::DurationChanged(e) => PlayerEvent::DurationChanged(e.clone()),
            PlayerEventC::TimeChanged(e) => PlayerEvent::TimeChanged(e.clone()),
            PlayerEventC::StateChanged(e) => PlayerEvent::StateChanged(e.clone()),
            PlayerEventC::VolumeChanged(e) => PlayerEvent::VolumeChanged(e.clone()),
        }
    }
}

impl From<PlayerEvent> for PlayerEventC {
    fn from(value: PlayerEvent) -> Self {
        match value {
            PlayerEvent::DurationChanged(e) => PlayerEventC::DurationChanged(e),
            PlayerEvent::TimeChanged(e) => PlayerEventC::TimeChanged(e),
            PlayerEvent::StateChanged(e) => PlayerEventC::StateChanged(e),
            PlayerEvent::VolumeChanged(e) => PlayerEventC::VolumeChanged(e),
        }
    }
}

/// A C-compatible struct representing a player.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct PlayerC {
    /// A pointer to a null-terminated C string representing the player's unique identifier (ID).
    pub id: *mut c_char,
    /// A pointer to a null-terminated C string representing the name of the player.
    pub name: *mut c_char,
    /// A pointer to a null-terminated C string representing the description of the player.
    pub description: *mut c_char,
    pub graphic_resource: *mut u8,
    pub graphic_resource_len: i32,
    /// The state of the player.
    pub state: PlayerState,
    /// Indicates whether embedded playback is supported by the player.
    pub embedded_playback_supported: bool,
}

impl From<Arc<Box<dyn Player>>> for PlayerC {
    fn from(value: Arc<Box<dyn Player>>) -> Self {
        trace!("Converting Player to PlayerC");
        let id = into_c_string(value.id().to_string());
        let name = into_c_string(value.name().to_string());
        let description = into_c_string(value.description().to_string());
        let (graphic_resource, graphic_resource_len) = if !value.graphic_resource().is_empty() {
            into_c_vec(value.graphic_resource())
        } else {
            (ptr::null_mut(), 0)
        };
        let embedded_playback_supported = if let Some(e) = value.downcast_ref::<PlayerWrapper>() {
            e.embedded_playback_supported.clone()
        } else {
            false
        };

        Self {
            id,
            name,
            description,
            graphic_resource,
            graphic_resource_len,
            state: value.state().clone(),
            embedded_playback_supported,
        }
    }
}

/// A C-compatible struct representing player registration information.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct PlayerRegistrationC {
    /// A pointer to a null-terminated C string representing the player's unique identifier (ID).
    pub id: *mut c_char,
    /// A pointer to a null-terminated C string representing the name of the player.
    pub name: *mut c_char,
    /// A pointer to a null-terminated C string representing the description of the player.
    pub description: *mut c_char,
    /// A Pointer to the graphic resource of the player.
    /// Use graphic_resource_len to get the length of the graphic resource byte array.
    pub graphic_resource: *mut u8,
    /// The length of the graphic resource array.
    pub graphic_resource_len: i32,
    /// The state of the player.
    pub state: PlayerState,
    /// Indicates whether embedded playback is supported by the player.
    pub embedded_playback_supported: bool,
    /// A callback function pointer for the "play" action.
    pub play_callback: PlayerPlayCallback,
    /// A callback function pointer for the "pause" action.
    pub pause_callback: PlayerPauseCallback,
    /// A callback function pointer for the "resume" action.
    pub resume_callback: PlayerResumeCallback,
    /// A callback function pointer for the "seek" action.
    pub seek_callback: PlayerSeekCallback,
    /// A callback function pointer for the "stop" action.
    pub stop_callback: PlayerStopCallback,
}

#[repr(C)]
#[derive(Display)]
#[display(fmt = "id: {}, name: {}", id, name)]
pub struct PlayerWrapper {
    id: String,
    name: String,
    description: String,
    graphic_resource: Vec<u8>,
    state: PlayerState,
    embedded_playback_supported: bool,
    play_callback: Mutex<Box<dyn Fn(PlayRequestC) + Send + Sync>>,
    pause_callback: Mutex<Box<dyn Fn() + Send + Sync>>,
    resume_callback: Mutex<Box<dyn Fn() + Send + Sync>>,
    seek_callback: Mutex<Box<dyn Fn(u64) + Send + Sync>>,
    stop_callback: Mutex<Box<dyn Fn() + Send + Sync>>,
    play_request: Mutex<Option<Arc<Box<dyn PlayRequest>>>>,
    callbacks: CoreCallbacks<PlayerEvent>,
}

impl PlayerWrapper {
    pub fn invoke(&self, event: PlayerEvent) {
        self.callbacks.invoke(event);
    }
}

impl Callbacks<PlayerEvent> for PlayerWrapper {
    fn add(&self, callback: CoreCallback<PlayerEvent>) -> CallbackHandle {
        self.callbacks.add(callback)
    }

    fn remove(&self, callback_id: CallbackHandle) {
        self.callbacks.remove(callback_id)
    }
}

#[async_trait]
impl Player for PlayerWrapper {
    fn id(&self) -> &str {
        self.id.as_str()
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn description(&self) -> &str {
        self.description.as_str()
    }

    fn graphic_resource(&self) -> Vec<u8> {
        self.graphic_resource.clone()
    }

    fn state(&self) -> PlayerState {
        self.state.clone()
    }

    fn request(&self) -> Option<Weak<Box<dyn PlayRequest>>> {
        let mutex = block_in_place(self.play_request.lock());
        mutex.as_ref()
            .map(|e| Arc::downgrade(e))
    }

    async fn play(&self, request: Box<dyn PlayRequest>) {
        trace!("Invoking play callback on C player for {:?} with {:?}", self, request);
        {
            let callback = self.play_callback.lock().await;
            callback(PlayRequestC::from(&request));
        }
        {
            let mut play_request = self.play_request.lock().await;
            *play_request = Some(Arc::new(request));
        }
    }

    fn pause(&self) {
        {
            let callback = block_in_place(self.pause_callback.lock());
            callback();
        }
    }

    fn resume(&self) {
        {
            let callback = block_in_place(self.resume_callback.lock());
            callback();
        }
    }

    fn seek(&self, time: u64) {
        {
            let callback = block_in_place(self.seek_callback.lock());
            callback(time);
        }
    }


    fn stop(&self) {
        {
            let callback = block_in_place(self.stop_callback.lock());
            callback();
        }
    }
}

impl Debug for PlayerWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlayerWrapper")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("description", &self.description)
            .field("graphic_resource", &self.graphic_resource.len())
            .field("state", &self.state)
            .field("embedded_playback_supported", &self.embedded_playback_supported)
            .field("callbacks", &self.callbacks)
            .finish()
    }
}

impl From<PlayerRegistrationC> for PlayerWrapper {
    fn from(value: PlayerRegistrationC) -> Self {
        trace!("Converting PlayerC to PlayerWrapperC");
        let id = from_c_string(value.id);
        let name = from_c_string(value.name);
        let description = from_c_string(value.description);
        let graphic_resource : Vec<u8> = if !value.graphic_resource.is_null() {
            from_c_vec(value.graphic_resource, value.graphic_resource_len)
        } else {
            Vec::new()
        };
        let play_callback = value.play_callback;
        let play_callback: Box<dyn Fn(PlayRequestC) + Send + Sync> = Box::new(move |e| play_callback(e));
        let pause_callback = value.pause_callback;
        let resume_callback = value.resume_callback;
        let seek_callback = value.seek_callback;
        let stop_callback = value.stop_callback;
        let pause_callback: Box<dyn Fn() + Send + Sync> = Box::new(move || pause_callback());
        let resume_callback: Box<dyn Fn() + Send + Sync> = Box::new(move || resume_callback());
        let seek_callback: Box<dyn Fn(u64) + Send + Sync> = Box::new(move |time| seek_callback(time));
        let stop_callback: Box<dyn Fn() + Send + Sync> = Box::new(move || stop_callback());

        Self {
            id,
            name,
            description,
            graphic_resource,
            state: value.state.clone(),
            embedded_playback_supported: value.embedded_playback_supported.clone(),
            play_callback: Mutex::new(play_callback),
            pause_callback: Mutex::new(pause_callback),
            resume_callback: Mutex::new(resume_callback),
            seek_callback: Mutex::new(seek_callback),
            stop_callback: Mutex::new(stop_callback),
            play_request: Default::default(),
            callbacks: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct PlayerWrapperC {
    id: *mut c_char,
    wrapper: Weak<Box<dyn Player>>,
}

impl PlayerWrapperC {
    pub fn id(&self) -> String {
        from_c_string(self.id)
    }

    pub fn instance(&self) -> Option<Arc<Box<dyn Player>>> {
        self.wrapper.upgrade()
    }
}

impl From<Weak<Box<dyn Player>>> for PlayerWrapperC {
    fn from(value: Weak<Box<dyn Player>>) -> Self {
        trace!("Converting PlayerWrapperC from Weak<Box<dyn Player>>");
        let id = into_c_string(value.upgrade()
            .map(|e| e.id().to_string())
            .unwrap_or("unknown".to_string()));

        Self {
            id,
            wrapper: value,
        }
    }
}

/// Represents a set of players in C-compatible form.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct PlayerSet {
    /// Pointer to an array of player instances.
    pub players: *mut PlayerC,
    /// Length of the player array.
    pub len: i32,
}

impl From<Vec<PlayerC>> for PlayerSet {
    /// Converts a vector of C players into a `PlayerSet`.
    ///
    /// # Arguments
    ///
    /// * `value` - The vector of C players to convert.
    ///
    /// # Returns
    ///
    /// A `PlayerSet` containing the converted players.
    fn from(value: Vec<PlayerC>) -> Self {
        trace!("Converting C players to PlayerSet");
        let (players, len) = into_c_vec(value);

        Self {
            players,
            len,
        }
    }
}

/// Represents events related to player management in C-compatible form.
#[repr(C)]
#[derive(Debug)]
pub enum PlayerManagerEventC {
    /// Indicates a change in the active player.
    ActivePlayerChanged(PlayerChangedEventC),
    /// Indicates a change in the players set.
    PlayersChanged,
    /// Indicates that the active player's playback has been changed
    PlayerPlaybackChanged(PlayRequestC),
    /// Indicates a change in the duration of a player.
    PlayerDurationChanged(u64),
    /// Indicates a change in the playback time of a player.
    PlayerTimeChanged(u64),
    /// Indicates a change in the state of a player.
    PlayerStateChanged(PlayerState),
}

impl From<PlayerManagerEvent> for PlayerManagerEventC {
    /// Converts a Rust `PlayerManagerEvent` into its C-compatible form.
    ///
    /// # Arguments
    ///
    /// * `value` - The `PlayerManagerEvent` to convert.
    ///
    /// # Returns
    ///
    /// The equivalent `PlayerManagerEventC` enum.
    fn from(value: PlayerManagerEvent) -> Self {
        match value {
            PlayerManagerEvent::ActivePlayerChanged(e) => PlayerManagerEventC::ActivePlayerChanged(PlayerChangedEventC::from(e)),
            PlayerManagerEvent::PlayersChanged => PlayerManagerEventC::PlayersChanged,
            PlayerManagerEvent::PlayerDurationChanged(e) => PlayerManagerEventC::PlayerDurationChanged(e),
            PlayerManagerEvent::PlayerTimeChanged(e) => PlayerManagerEventC::PlayerTimeChanged(e),
            PlayerManagerEvent::PlayerStateChanged(e) => PlayerManagerEventC::PlayerStateChanged(e),
            PlayerManagerEvent::PlayerPlaybackChanged(e) => PlayerManagerEventC::PlayerPlaybackChanged(e.upgrade()
                .map(|e| PlayRequestC::from(e))
                .expect("expected the play request to still be in scope")),
        }
    }
}

/// Represents a play request in C-compatible form.
#[repr(C)]
#[derive(Debug)]
pub struct PlayRequestC {
    /// The URL of the media to be played.
    pub url: *mut c_char,
    /// The title of the media.
    pub title: *mut c_char,
    /// The optional caption of the media, or [ptr::null_mut] if not available.
    pub caption: *mut c_char,
    /// The optional URL of the thumbnail image for the media, or [ptr::null_mut] if not available.
    pub thumb: *mut c_char,
    /// The URL of the background image for the media, or [ptr::null_mut] if not available.
    pub background: *mut c_char,
    /// The quality of the media, or [ptr::null_mut] if not available.
    pub quality: *mut c_char,
    /// Pointer to a mutable u64 value representing the auto-resume timestamp.
    pub auto_resume_timestamp: *mut u64,
    /// The stream handle pointer of the play request.
    /// This handle can be used to retrieve more information about the underlying stream.
    pub stream_handle: *mut i64,
    /// Indicates whether subtitles are enabled for the media.
    pub subtitles_enabled: bool,
}

impl From<&PlayUrlRequest> for PlayRequestC {
    fn from(value: &PlayUrlRequest) -> Self {
        trace!("Converting PlayUrlRequest to PlayRequestC for {:?}", value);
        let caption = if let Some(caption) = value.caption() {
            into_c_string(caption)
        } else {
            ptr::null_mut()
        };
        let thumb = if let Some(thumb) = value.thumbnail() {
            into_c_string(thumb)
        } else {
            ptr::null_mut()
        };
        let background = if let Some(background) = value.background() {
            into_c_string(background)
        } else {
            ptr::null_mut()
        };
        let quality = if let Some(quality) = value.quality() {
            into_c_string(quality)
        } else {
            ptr::null_mut()
        };
        let auto_resume_timestamp = if let Some(e) = value.auto_resume_timestamp() {
            into_c_owned(e)
        } else {
            ptr::null_mut()
        };

        Self {
            url: into_c_string(value.url.clone()),
            title: into_c_string(value.title.clone()),
            thumb,
            caption,
            background,
            quality,
            auto_resume_timestamp,
            subtitles_enabled: value.subtitles_enabled,
            stream_handle: ptr::null_mut(),
        }
    }
}

impl From<&PlayMediaRequest> for PlayRequestC {
    fn from(value: &PlayMediaRequest) -> Self {
        trace!("Converting PlayMediaRequest to PlayRequestC for {:?}", value);
        let caption = if let Some(caption) = value.caption() {
            into_c_string(caption)
        } else {
            ptr::null_mut()
        };
        let thumb = if let Some(thumb) = value.thumbnail() {
            into_c_string(thumb)
        } else {
            ptr::null_mut()
        };
        let background = if let Some(background) = value.background() {
            into_c_string(background)
        } else {
            ptr::null_mut()
        };
        let quality = if let Some(quality) = value.quality() {
            into_c_string(quality)
        } else {
            ptr::null_mut()
        };
        let auto_resume_timestamp = if let Some(e) = value.auto_resume_timestamp() {
            into_c_owned(e)
        } else {
            ptr::null_mut()
        };
        let stream_handle = if let Some(e) = value.torrent_stream.upgrade() {
            into_c_owned(e.stream_handle().value())
        } else {
            ptr::null_mut()
        };

        Self {
            url: into_c_string(value.base.url.clone()),
            title: into_c_string(value.base.title.clone()),
            caption,
            thumb,
            background,
            quality,
            auto_resume_timestamp,
            subtitles_enabled: value.subtitles_enabled(),
            stream_handle,
        }
    }
}

impl From<Box<dyn PlayRequest>> for PlayRequestC {
    fn from(value: Box<dyn PlayRequest>) -> Self {
        Self::from(&value)
    }
}

impl From<&Box<dyn PlayRequest>> for PlayRequestC {
    fn from(value: &Box<dyn PlayRequest>) -> Self {
        if let Some(value) = value.downcast_ref::<PlayMediaRequest>() {
            return PlayRequestC::from(value);
        } else if let Some(value) = value.downcast_ref::<PlayUrlRequest>() {
            return PlayRequestC::from(value);
        }

        panic!("Unexpected play request {:?}", value)
    }
}

impl From<Arc<Box<dyn PlayRequest>>> for PlayRequestC {
    fn from(value: Arc<Box<dyn PlayRequest>>) -> Self {
        if let Some(value) = value.downcast_ref::<PlayMediaRequest>() {
            return PlayRequestC::from(value);
        } else if let Some(value) = value.downcast_ref::<PlayUrlRequest>() {
            return PlayRequestC::from(value);
        }

        panic!("Unexpected play request {:?}", value)
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use log::info;

    use popcorn_fx_core::{from_c_owned, from_c_vec};
    use popcorn_fx_core::core::Handle;
    use popcorn_fx_core::core::media::MovieOverview;
    use popcorn_fx_core::core::players::PlayerChange;
    use popcorn_fx_core::core::torrents::TorrentStream;
    use popcorn_fx_core::testing::{init_logger, MockPlayer, MockTorrentStream};

    use super::*;

    #[no_mangle]
    extern "C" fn play_callback(_: PlayRequestC) {
        info!("Player play C callback invoked");
    }

    #[no_mangle]
    extern "C" fn pause_callback() {
        info!("Player pause C callback invoked");
    }

    #[no_mangle]
    extern "C" fn resume_callback() {
        info!("Player resume C callback invoked");
    }

    #[no_mangle]
    extern "C" fn seek_callback(time: u64) {
        info!("Player seek C callback invoked with {}", time);
    }

    #[no_mangle]
    extern "C" fn stop_callback() {
        info!("Player stop C callback invoked");
    }

    #[test]
    fn test_from_player() {
        init_logger();
        let player_id = "FooBar123";
        let player_name = "foo";
        let player_description = "lorem ipsum dolor";
        let graphic_resource = vec![80, 20];
        let mut mock_player = MockPlayer::new();
        mock_player.expect_id()
            .return_const(player_id.to_string());
        mock_player.expect_name()
            .return_const(player_name.to_string());
        mock_player.expect_description()
            .return_const(player_description.to_string());
        mock_player.expect_graphic_resource()
            .return_const(graphic_resource.clone());
        mock_player.expect_state()
            .return_const(PlayerState::Playing);
        let player = Arc::new(Box::new(mock_player) as Box<dyn Player>);

        let result = PlayerC::from(player);

        let bytes = from_c_vec(result.graphic_resource, result.graphic_resource_len);
        assert_eq!(player_id.to_string(), from_c_string(result.id));
        assert_eq!(player_name.to_string(), from_c_string(result.name));
        assert_eq!(player_description, from_c_string(result.description));
        assert_eq!(graphic_resource, bytes);
        assert_eq!(PlayerState::Playing, result.state);
    }

    #[test]
    fn test_from_player_for_wrapper() {
        init_logger();
        let state = PlayerState::Stopped;
        let player = Arc::new(Box::new(PlayerWrapper {
            id: "".to_string(),
            name: "".to_string(),
            description: "".to_string(),
            graphic_resource: vec![],
            state: state.clone(),
            embedded_playback_supported: true,
            play_callback: Mutex::new(Box::new(|_| {})),
            pause_callback: Mutex::new(Box::new(|| {})),
            resume_callback: Mutex::new(Box::new(|| {})),
            seek_callback: Mutex::new(Box::new(|_| {})),
            stop_callback: Mutex::new(Box::new(|| {})),
            play_request: Default::default(),
            callbacks: Default::default(),
        }) as Box<dyn Player>);

        let result = PlayerC::from(player);

        assert_eq!(state, result.state);
        assert_eq!(true, result.embedded_playback_supported, "expected the embedded playback value to have been set");
    }

    #[test]
    fn from_players() {
        init_logger();
        let player_id = "player123";
        let player = PlayerC {
            id: into_c_string(player_id.to_string()),
            name: into_c_string("my_player".to_string()),
            description: ptr::null_mut(),
            graphic_resource: ptr::null_mut(),
            graphic_resource_len: 0,
            state: PlayerState::Stopped,
            embedded_playback_supported: false,
        };
        let players = vec![player];

        let set = PlayerSet::from(players);
        assert_eq!(1, set.len);

        let vec = from_c_vec(set.players, set.len);
        let result = vec.get(0).unwrap();
        assert_eq!(player_id.to_string(), from_c_string(result.id));
    }

    #[test]
    fn test_from_player_c() {
        init_logger();
        let player_id = "InternalPlayerId";
        let player_name = "InternalPlayerName";
        let description = "Lorem ipsum dolor esta";
        let resource = vec![84, 78, 90];
        let (graphic_resource, graphic_resource_len) = into_c_vec(resource.clone());
        let player = PlayerRegistrationC {
            id: into_c_string(player_id.to_string()),
            name: into_c_string(player_name.to_string()),
            description: into_c_string(description.to_string()),
            graphic_resource,
            graphic_resource_len,
            state: PlayerState::Paused,
            embedded_playback_supported: false,
            play_callback,
            pause_callback,
            resume_callback,
            seek_callback,
            stop_callback,
        };

        let wrapper = PlayerWrapper::from(player);

        assert_eq!(player_id, wrapper.id());
        assert_eq!(player_name, wrapper.name());
        assert_eq!(description, wrapper.description());
        assert_eq!(resource, wrapper.graphic_resource());
    }

    #[test]
    fn test_player_manager_event_c_from() {
        let player_id = "MyId";
        let event = PlayerManagerEvent::ActivePlayerChanged(PlayerChange {
            old_player_id: None,
            new_player_id: player_id.to_string(),
            new_player_name: "".to_string(),
        });

        let result = PlayerManagerEventC::from(event);
        if let PlayerManagerEventC::ActivePlayerChanged(e) = result {
            assert_eq!(player_id.to_string(), from_c_string(e.new_player_id));
        } else {
            assert!(false, "expected PlayerManagerEventC::ActivePlayerChanged, got {:?} instead", result);
        }

        let result = crate::ffi::mappings::players::PlayerManagerEventC::from(PlayerManagerEvent::PlayersChanged);
        if let PlayerManagerEventC::PlayersChanged = result {} else {
            assert!(false, "expected PlayerManagerEventC::PlayersChanged, got {:?} instead", result);
        }
    }

    #[test]
    fn test_play_request_c_from_play_url_request() {
        let url = "https://localhost:8090/foo.mp4";
        let title = "FooBar";
        let thumb = "MyThumb.png";
        let background = "MyBackground.png";
        let request = PlayUrlRequest::builder()
            .url(url)
            .title(title)
            .thumb(thumb)
            .background(background)
            .build();

        let result = PlayRequestC::from(&request);

        assert_eq!(url.to_string(), from_c_string(result.url));
        assert_eq!(title.to_string(), from_c_string(result.title));
        assert_eq!(thumb.to_string(), from_c_string(result.thumb));
        assert_eq!(background.to_string(), from_c_string(result.background));
    }

    #[test]
    fn test_play_request_c_from_play_media_request() {
        let url = "https://localhost:8090/foo.mp4";
        let title = "FooBar";
        let thumb = "MyThumb.png";
        let background = "MyBackground.png";
        let handle = Handle::new();
        let mut torrent_stream = MockTorrentStream::new();
        torrent_stream.expect_stream_handle()
            .times(1)
            .return_const(handle.clone());
        let torrent_stream = Arc::new(Box::new(torrent_stream) as Box<dyn TorrentStream>);
        let movie = MovieOverview {
            title: "".to_string(),
            imdb_id: "".to_string(),
            year: "".to_string(),
            rating: None,
            images: Default::default(),
        };
        let request = PlayMediaRequest::builder()
            .url(url)
            .title(title)
            .thumb(thumb)
            .background(background)
            .media(Box::new(movie))
            .torrent_stream(Arc::downgrade(&torrent_stream))
            .build();

        let result = PlayRequestC::from(&request);

        assert_eq!(url.to_string(), from_c_string(result.url));
        assert_eq!(title.to_string(), from_c_string(result.title));
        assert_eq!(thumb.to_string(), from_c_string(result.thumb));
        assert_eq!(background.to_string(), from_c_string(result.background));
        assert_eq!(handle.value(), from_c_owned(result.stream_handle));
    }
}