use std::env;
use std::ffi::{c_char, CString};
use std::path::PathBuf;
use std::string::ToString;

use async_trait::async_trait;
use libloading::Library;
use log::{debug, error, trace, warn};
use tokio::sync::Mutex;

use popcorn_fx_core::core::block_in_place;
use popcorn_fx_core::core::utils::network::available_socket;

use crate::chromecast::transcode;
use crate::chromecast::transcode::{TranscodeError, TranscodeOutput, Transcoder, TranscodeState, TranscodeType};
use crate::chromecast::transcode::lib_vlc::{LibraryHandle, libvlc_instance_t, libvlc_media_add_option, libvlc_media_new_location, libvlc_media_player_new, libvlc_media_player_play, libvlc_media_player_release, libvlc_media_player_set_media, libvlc_media_player_stop, libvlc_media_player_t, libvlc_media_release, libvlc_media_t, LibvlcInstanceT};

#[cfg(target_family = "unix")]
const PATH_SEPARATOR: &str = ":";
#[cfg(target_family = "windows")]
const PATH_SEPARATOR: &str = ";";
#[cfg(target_os = "macos")]
const LIBVLC_FILENAMES: [&str; 1] = ["libvlc.dylib"];
#[cfg(target_os = "macos")]
const LIBVLC_WELL_KNOWN_DIRECTORIES: [&str; 2] = [
    "/Applications/VLC.app/Contents/Frameworks",
    "/Applications/VLC.app/Contents/MacOS/lib",
];
#[cfg(target_os = "macos")]
const LIBVLC_PLUGIN_PATHS: [&str; 1] = [
    "../plugins",
];
#[cfg(target_os = "linux")]
const LIBVLC_FILENAMES: [&str; 1] = [
    "libvlc.so"
];
#[cfg(target_os = "linux")]
const LIBVLC_WELL_KNOWN_DIRECTORIES: [&str; 6] = [
    "/usr/lib/x86_64-linux-gnu",
    "/usr/lib64",
    "/usr/local/lib64",
    "/usr/lib/i386-linux-gnu",
    "/usr/lib",
    "/usr/local/lib",
];
#[cfg(target_os = "linux")]
const LIBVLC_PLUGIN_PATHS: [&str; 2] = [
    "plugins",
    "vlc/plugins",
];
#[cfg(target_os = "windows")]
const LIBVLC_FILENAMES: [&str; 1] = [
    "libvlc.dll"
];
#[cfg(target_os = "windows")]
const LIBVLC_WELL_KNOWN_DIRECTORIES: [&str; 0] = [];
#[cfg(target_os = "windows")]
const LIBVLC_PLUGIN_PATHS: [&str; 2] = [
    "plugins",
    "vlc\\plugins",
];

/// VLC transcoder used for media transcoding with libvlc.
/// The VLC transcoder accepts any http media stream as its input and will provide a new output 
/// http stream with the transcoded media.
#[derive(Debug)]
pub struct VlcTranscoder {
    library: LibraryHandle,
    instance: LibvlcInstanceT<libvlc_instance_t>,
    media_player: Mutex<Option<LibvlcInstanceT<libvlc_media_player_t>>>,
    media: Mutex<Option<LibvlcInstanceT<libvlc_media_t>>>,
    state: Mutex<TranscodeState>,
}

impl VlcTranscoder {
    /// Creates a new `VlcTranscoder` instance for the given instance and library handle.
    /// The [libvlc_instance_t] should be a valid pointer and not NULL.
    ///
    /// # Example
    ///
    /// Use [VlcTranscoderDiscovery] to discover and create an instance of `VlcTranscoder`.
    ///
    /// ```rust,no_run
    /// use popcorn_fx_players::chromecast::transcode::VlcTranscoderDiscovery;
    ///
    /// let transcoder = VlcTranscoderDiscovery::discover().expect("expected a VLC transcoder");
    /// ```
    ///
    /// Or construct it with a manipulated library or custom path.
    ///
    /// ```
    /// use popcorn_fx_players::chromecast::transcode::{VlcTranscoder, VlcTranscoderDiscovery};
    ///
    /// let directories = ["/tmp/my-dir".to_string(), "/some/other/path".to_string()];
    /// let (instance, library) = VlcTranscoderDiscovery::do_libvlc_discovery(directories.to_vec()).expect("expected a VLC instance");
    /// let transcoder = VlcTranscoder::new(instance, library);
    /// ```
    pub fn new(instance: libvlc_instance_t, library: LibraryHandle) -> Self {
        Self {
            library,
            instance: LibvlcInstanceT::new(instance),
            media_player: Default::default(),
            media: Default::default(),
            state: Mutex::new(TranscodeState::Unknown),
        }
    }

    async fn update_state_async(&self, state: TranscodeState) {
        let mut mutex = self.state.lock().await;
        trace!("Updating transcoder state to {:?}", state);
        *mutex = state.clone();
        debug!("Transcoder state changed to {:?}", state);
    }

    async fn create_media_player(&self) -> transcode::Result<LibvlcInstanceT<libvlc_media_player_t>> {
        trace!("Creating new VLC media player instance");
        let native_fn = self.library.get::<libvlc_media_player_new>(b"libvlc_media_player_new")
            .map_err(|e| TranscodeError::Initialization(e.to_string()))?;
        let media_player = native_fn(self.instance.0);

        let media_player = LibvlcInstanceT::new(media_player);
        trace!("Created new VLC media player instance {:?}", media_player);
        {
            let mut mutex = self.media_player.lock().await;
            *mutex = Some(media_player.clone());
            debug!("Stored new VLC media player instance {:?}", media_player);
        }
        Ok(media_player)
    }

    async fn create_media(&self, url: &str, options: &[&str]) -> transcode::Result<LibvlcInstanceT<libvlc_media_t>> {
        let native_fn = self.library.get::<libvlc_media_new_location>(b"libvlc_media_new_location\0")
            .map_err(|e| TranscodeError::Initialization(e.to_string()))?;
        let murl = CString::new(url)
            .map_err(|e| TranscodeError::Initialization(e.to_string()))?;

        // release the current media item if one is present
        self.release_media().await;

        let media = LibvlcInstanceT::new(native_fn(self.instance.0, murl.into_raw()));
        debug!("Created new media item {:?}", media);

        // initialize the media options
        let option_fn = self.library.get::<libvlc_media_add_option>(b"libvlc_media_add_option\0")
            .map_err(|e| TranscodeError::Initialization(e.to_string()))?;
        trace!("Adding media item options {:?}", options);
        for option in options {
            option_fn(media.0, Self::parse_string(*option));
        }

        {
            let mut mutex = self.media.lock().await;
            *mutex = Some(media.clone());
        }
        Ok(media)
    }

    async fn release_media(&self) {
        if let Some(media) = self.media.lock().await.take() {
            match self.library.get::<libvlc_media_release>(b"libvlc_media_release\0") {
                Ok(native_fn) => {
                    debug!("Releasing media item {:?}", media);
                    native_fn(media.0);
                }
                Err(e) => error!("Unable to release media, failed to get libvlc_media_release: {}", e),
            }
        }
    }

    async fn release_media_player(&self) {
        if let Some(media_player) = self.media_player.lock().await.take() {
            match self.library.get::<libvlc_media_player_release>(b"libvlc_media_player_release\0") {
                Ok(native_fn) => {
                    debug!("Releasing media player {:?}", media_player);
                    native_fn(media_player.0);
                }
                Err(e) => error!("Unable to release media player, failed to get libvlc_media_player_release: {}", e),
            }
        }
    }

    fn parse_string<S: Into<Vec<u8>>>(value: S) -> *mut c_char {
        CString::new(value.into())
            .expect("Failed to create C string")
            .into_raw()
    }

    fn change_media(&self, media_player: LibvlcInstanceT<libvlc_media_player_t>, media: LibvlcInstanceT<libvlc_media_t>) -> transcode::Result<()> {
        let native_fn = self.library.get::<libvlc_media_player_set_media>(b"libvlc_media_player_set_media\0")
            .map_err(|e| TranscodeError::Initialization(e.to_string()))?;
        native_fn(media_player.0, media.0);
        debug!("Changed media on media player {:?} to {:?}", media_player, media);
        Ok(())
    }

    fn play(&self, media_player: LibvlcInstanceT<libvlc_media_player_t>) -> transcode::Result<()> {
        let native_fn = self.library.get::<libvlc_media_player_play>(b"libvlc_media_player_play\0")
            .map_err(|e| TranscodeError::Initialization(e.to_string()))?;

        if native_fn(media_player.0) != 0 {
            return Err(TranscodeError::Initialization("transcoding failed to start".to_string()));
        }

        debug!("Started transcoding on media player {:?}", media_player);
        Ok(())
    }

    async fn stop_player(&self) -> transcode::Result<()> {
        if let Some(media_player) = self.media_player.lock().await.clone() {
            let native_fn = self.library.get::<libvlc_media_player_stop>(b"libvlc_media_player_stop\0")
                .map_err(|e| TranscodeError::Initialization(e.to_string()))?;
            native_fn(media_player.0);

            debug!("Stopped transcoding on media player {:?}", media_player);
            self.update_state_async(TranscodeState::Stopped).await;
        }

        Ok(())
    }
}

#[async_trait]
impl Transcoder for VlcTranscoder {
    fn state(&self) -> TranscodeState {
        let mutex = block_in_place(self.state.lock());
        mutex.clone()
    }

    async fn transcode(&self, url: &str) -> transcode::Result<TranscodeOutput> {
        self.update_state_async(TranscodeState::Preparing).await;
        let filename = PathBuf::from(url)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let socket = available_socket();
        let destination = format!("{}/{}", socket, filename);

        let media_player = self.media_player.lock().await.clone();
        let media_player = match media_player {
            Some(media_player) => media_player,
            None => self.create_media_player().await?,
        };
        let media = self.create_media(url, &[
            format!(":sout=#transcode{{vcodec=h264,vb=2048,fps=24,maxwidth=1920,maxheight=1080,acodec=mp3,ab=128,channels=2,threads=0}}:std{{mux=avformat{{mux=matroska,options={{live=1}},reset-ts}},dst={},access=http}}", destination).as_str(),
            ":demux-filter=demux_chromecast",
            ":sout-mux-caching=8192",
            ":sout-all",
            ":sout-keep",
        ]).await?;

        self.update_state_async(TranscodeState::Starting).await;
        self.change_media(media_player, media)?;
        self.play(media_player)?;

        self.update_state_async(TranscodeState::Transcoding).await;
        Ok(TranscodeOutput {
            url: format!("http://{}", destination),
            // VLC transcoding only supports live output
            // this limits the buffering options as well as the ability to seek within the stream
            output_type: TranscodeType::Live,
        })
    }

    async fn stop(&self) {
        let _ = self.stop_player().await;
        self.release_media().await;
        self.release_media_player().await;
    }
}

unsafe impl Sync for VlcTranscoder {}

unsafe impl Send for VlcTranscoder {}

impl Drop for VlcTranscoder {
    fn drop(&mut self) {
        // make sure we release the VLC instances before dropping the transcoder
        block_in_place(self.stop());
    }
}

/// Represents a VLC transcoder discovery mechanism.
pub struct VlcTranscoderDiscovery {}

impl VlcTranscoderDiscovery {
    /// Discovers a VLC transcoder instance.
    ///
    /// This function searches for VLC libraries in well-known directories and attempts to load the `libvlc_new` function.
    ///
    /// # Returns
    ///
    /// An `Option<VlcTranscoder>` containing the VLC transcoder instance if found, otherwise `None`.
    pub fn discover() -> Option<VlcTranscoder> {
        let directories = Self::search_directories();

        Self::do_libvlc_discovery(directories)
            .map(|(instance, library)| VlcTranscoder::new(instance, library))
    }

    /// Execute a VLC library discovery for the given directories.
    ///
    /// This function can be used to construct your own transcoder with VLC, or manipulate the VLC library
    /// before creating the [VlcTranscoder].
    ///
    /// # Returns
    ///
    /// An `Option<(libvlc_instance_t, LibraryHandle)>` containing the VLC library instance and handle if found, otherwise `None`.
    pub fn do_libvlc_discovery(directories: Vec<String>) -> Option<(libvlc_instance_t, LibraryHandle)> {
        for path in directories {
            for filename in LIBVLC_FILENAMES {
                if let Some(library) = Self::search_lib(path.as_str(), filename) {
                    return library.libvlc_instance()
                        .map(|instance| (instance, library));
                }
            }
        }

        debug!("VLC library couldn't be found");
        None
    }

    /// Searches for directories where VLC libraries may be located.
    ///
    /// This function collects directories from the system's PATH variable and includes the current directory.
    ///
    /// # Returns
    ///
    /// A vector of directory paths where VLC libraries may be located.
    fn search_directories() -> Vec<String> {
        let system_path = env::var("PATH").unwrap_or(String::new());
        let mut directories: Vec<String> = system_path
            .split(PATH_SEPARATOR)
            .into_iter()
            .map(|e| e.to_string())
            .collect();

        if let Ok(e) = env::current_dir()
            .map_err(|_| ())
            .and_then(|e| e.to_str().map(|e| Ok(e.to_string())).unwrap_or(Err(())))
        {
            directories.push(e);
        }

        for directory in LIBVLC_WELL_KNOWN_DIRECTORIES {
            directories.push(directory.to_string());
        }

        directories
    }

    /// Searches for a VLC library in the specified path with the given filename.
    ///
    /// Additionally, it attempts to load the core dependency from the same location.
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the VLC library may be located.
    /// * `filename` - The filename of the VLC library.
    ///
    /// # Returns
    ///
    /// An `Option<Library>` containing the loaded VLC library if found, otherwise `None`.
    fn search_lib(lib_path: &str, filename: &str) -> Option<LibraryHandle> {
        let main_buf = PathBuf::from(lib_path).join(filename);
        let filepath = main_buf.to_str().unwrap();
        let core_filename = filename.replace(".", "core.");
        let core_buf = PathBuf::from(lib_path).join(core_filename);
        let core_filepath = core_buf.to_str().unwrap();

        // try to load the core dependency from the same location
        match unsafe { Library::new(core_filepath) } {
            Ok(libcore) => {
                return match unsafe { Library::new(filepath) } {
                    Ok(libvlc) => {
                        debug!("Found libvlc at {}", filepath);
                        if let Some(plugin_path) = Self::search_plugins_path(lib_path) {
                            Some(LibraryHandle::new(lib_path, plugin_path, libvlc, libcore))
                        } else {
                            None
                        }
                    }
                    Err(e) => {
                        trace!("VLC library {} not found, {}", filepath, e);
                        None
                    }
                };
            }
            Err(e) => {
                trace!("VLC library {} not found, {}", filepath, e);
                None
            }
        }
    }

    /// Searches for VLC plugins in the specified library path.
    ///
    /// # Arguments
    ///
    /// * `lib_path` - The path where the VLC library is located.
    ///
    /// # Returns
    ///
    /// An `Option<String>` containing the name of the found plugin directory if any, otherwise `None`.
    fn search_plugins_path(lib_path: &str) -> Option<String> {
        for plugin_dir in LIBVLC_PLUGIN_PATHS {
            let plugin_path = PathBuf::from(lib_path).join(plugin_dir);

            if plugin_path.exists() {
                let plugin_path = plugin_path.to_str().unwrap().to_string();
                debug!("Found VLC plugin at {}", plugin_path);
                return Some(plugin_path);
            }
        }

        warn!("VLC plugins not found");
        None
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::runtime::Runtime;

    use popcorn_fx_core::testing::init_logger;

    use super::*;

    #[test]
    fn test_vlc_transcoder_state() {
        init_logger();
        let transcoder = VlcTranscoderDiscovery::discover().unwrap();
        
        let result = transcoder.state();
        
        assert_eq!(TranscodeState::Unknown, result);
    }

    #[test]
    fn test_vlc_transcoder_discovery() {
        init_logger();

        let result = VlcTranscoderDiscovery::discover();

        assert!(
            result.is_some(),
            "expected a VLC transcoder to have been found"
        );
    }

    #[test]
    fn test_vlc_transcoder_transcode() {
        init_logger();
        let runtime = Arc::new(Runtime::new().unwrap());
        let transcoder = VlcTranscoderDiscovery::discover().unwrap();

        let result = runtime
            .block_on(transcoder.transcode("http://localhost:8900/my-video.mp4"))
            .expect("expected a transcodig stream to be returned");

        assert_ne!(String::new(), result.url);
        assert_eq!(TranscodeType::Live, result.output_type);
        assert_eq!(TranscodeState::Transcoding, transcoder.state());
        runtime.block_on(transcoder.stop());
    }
}
