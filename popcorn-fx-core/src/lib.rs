use std::backtrace::Backtrace;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::{mem, ptr};

use log::{error, trace, warn};

/// The current application version of Popcorn FX.
pub const VERSION: &str = "0.8.2";

pub mod core;

/// Converts the given value into a C compatible string.
///
/// This function will consume the provided data and use the underlying bytes to construct a new string, ensuring that there is a trailing 0 byte.
/// This trailing 0 byte will be appended by this function; the provided data should not contain any 0 bytes in it.
///
/// # Arguments
///
/// * `value` - The value to convert into a C string.
///
/// # Returns
///
/// A pointer to the C string.
pub fn into_c_string<S: Into<String>>(value: S) -> *mut c_char {
    let value = value.into();

    if value.len() > 0 {
        let c_string = CString::new(value).expect("expected valid C string");
        c_string.into_raw()
    } else {
        warn!(
            "Unable to create C string from empty string\n{}",
            Backtrace::capture()
        );
        ptr::null_mut()
    }
}

/// Converts the given C string pointer into a Rust string.
///
/// # Safety
///
/// This function is marked as `unsafe` because it dereferences a raw pointer.
///
/// # Arguments
///
/// * `ptr` - The pointer to the C string.
///
/// # Returns
///
/// The owned Rust String.
pub fn from_c_string(ptr: *const c_char) -> String {
    if !ptr.is_null() {
        let slice = unsafe { CStr::from_ptr(ptr).to_bytes() };

        std::str::from_utf8(slice)
            .map(|e| e.to_string())
            .unwrap_or_else(|e| {
                error!(
                    "Failed to read C string, using empty string instead ({})",
                    e
                );
                String::new()
            })
    } else {
        error!(
            "Unable to read C string, pointer is null\n{}",
            Backtrace::capture()
        );
        String::new()
    }
}

/// Converts a C string into a Rust owned string, consuming the provided pointer.
///
/// # Arguments
///
/// * `ptr` - A pointer to the C string.
///
/// # Returns
///
/// Returns the Rust owned string if the conversion is successful, otherwise an empty string.
pub fn from_c_string_owned(ptr: *mut c_char) -> String {
    if !ptr.is_null() {
        let value = unsafe { CString::from_raw(ptr) };

        value.into_string().unwrap_or_else(|e| {
            error!(
                "Failed to read C string, using empty string instead ({})",
                e
            );
            String::new()
        })
    } else {
        error!(
            "Unable to read C string, pointer is null\n{}",
            Backtrace::capture()
        );
        String::new()
    }
}

/// Move the ownership of the given value to the C caller.
/// For more info, see [Box::into_raw].
/// * `value` - The value to convert to a pointer
pub fn into_c_owned<T>(value: T) -> *mut T {
    Box::into_raw(Box::new(value))
}

/// Retrieve a C owned value as an owned value.
/// For more info, see [Box::from_raw].
///
/// * `ptr` - The pointer value to convert
pub fn from_c_owned<T>(ptr: *mut T) -> T {
    let value = from_c_into_boxed(ptr);
    *value
}

/// Retrieve a C value as a [Box] value.
///
/// This function is preferred over `into_c_owned` when you want to obtain a Rust [Box] without
/// taking ownership of the underlying C memory. Using this function, you are responsible for
/// managing the C memory manually.
///
/// # Safety
///
/// This function is marked as `unsafe` because it involves raw pointer manipulation.
///
/// # Arguments
///
/// * `ptr` - The pointer value to convert.
///
/// # Panics
///
/// Panics if the provided `ptr` is null.
///
/// # Returns
///
/// Returns a [Box] containing the value referred to by the provided pointer.
///
/// # Example
///
/// ```no_run
/// use std::mem;
/// use popcorn_fx_core::from_c_into_boxed;
///
/// // Assume you have a C value `value` with a media pointer.
/// let media_item = from_c_into_boxed(value.media);
///
/// // Perform operations with the media_item...
/// let identifier = media_item.as_identifier();
///
/// // Don't forget to manually manage the C memory, as ownership has not been transferred to Rust.
/// mem::forget(media_item);
/// ```
pub fn from_c_into_boxed<T>(ptr: *mut T) -> Box<T> {
    if !ptr.is_null() {
        unsafe { Box::from_raw(ptr) }
    } else {
        panic!("Unable to read C instance, pointer is null")
    }
}

/// Convert the given [Vec] into a C array tuple which is owned by the caller.
/// The return tuple is as follows: `(pointer, length)`
pub fn into_c_vec<T>(mut vec: Vec<T>) -> (*mut T, i32) {
    // check if the vec contains items
    // if not, we return a ptr::null as ABI can't handle empty arrays
    if !vec.is_empty() {
        vec.shrink_to_fit();
        let mut boxed = vec.into_boxed_slice();
        let ptr = boxed.as_mut_ptr();
        let len = boxed.len() as i32;
        mem::forget(boxed);

        (ptr, len)
    } else {
        (ptr::null_mut(), 0)
    }
}

/// Converts the given C array into a **copied** `Vec`.
///
/// This function **does not** take ownership of the C array pointer and reads it as a new Rust `Vec`.
/// If you want to clean the C memory after reading the array, use [from_c_vec_owned] instead.
/// If the `ptr` is null, it returns an empty `Vec`.
///
/// For more information, see [`std::slice::from_raw_parts_mut`].
///
/// # Arguments
///
/// * `ptr` - The pointer to the C array.
/// * `len` - The length of the C array.
///
/// # Returns
///
/// The resulting `Vec` on success, or an empty `Vec` if the `ptr` is null.
pub fn from_c_vec<T: Clone>(ptr: *mut T, len: i32) -> Vec<T> {
    trace!("Converting C ptr: {:?}, len: {} into a Vec", ptr, len);
    if !ptr.is_null() {
        if len >= 0 {
            let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len as usize) };
            slice.into()
        } else {
            error!("Unable to read C array, array length is negative");
            vec![]
        }
    } else {
        error!("Unable to read C array, array pointer is null");
        vec![]
    }
}

/// Converts the given C array into an owned `Vec`.
///
/// This function takes ownership of the C array pointer and transfers it into a Rust `Vec`.
/// If the `ptr` is null, it returns an empty `Vec`.
///
/// For more information, see [`Vec::from_raw_parts`].
///
/// # Arguments
///
/// * `ptr` - The pointer to the C array.
/// * `len` - The length of the C array.
///
/// # Returns
///
/// The resulting `Vec` on success, or an empty `Vec` if the `ptr` is null.
pub fn from_c_vec_owned<T>(ptr: *mut T, len: i32) -> Vec<T> {
    trace!("Converting C ptr: {:?}, len: {} into a owned Vec", ptr, len);
    if !ptr.is_null() {
        if len > 0 {
            let len = len as usize;
            let slice = unsafe { Vec::from_raw_parts(ptr, len, len) };
            slice.into()
        } else {
            trace!("C array is empty, returning empty Vector");
            vec![]
        }
    } else {
        error!("Unable to read C array, array pointer is null");
        vec![]
    }
}

#[cfg(feature = "testing")]
pub mod testing {
    use crate::core::platform::{Platform, PlatformCallback, PlatformData, PlatformInfo};
    use crate::core::playback::MediaNotificationEvent;
    use crate::core::players::{PlayRequest, Player, PlayerEvent, PlayerState};
    use crate::core::subtitles::model::SubtitleInfo;
    use crate::core::subtitles::{SubtitleEvent, SubtitleManager, SubtitlePreference};
    use crate::core::torrents::{
        StreamHandle, Torrent, TorrentEvent, TorrentHandle, TorrentState, TorrentStream,
        TorrentStreamEvent, TorrentStreamState, TorrentStreamingResourceWrapper,
    };
    use crate::core::{torrents, Callbacks, CoreCallback};

    use async_trait::async_trait;
    use fx_callback::{Callback, CallbackHandle, Subscriber, Subscription};
    use fx_handle::Handle;
    use log::{debug, trace, LevelFilter};
    use log4rs::append::console::ConsoleAppender;
    use log4rs::config::{Appender, Logger, Root};
    use log4rs::encode::pattern::PatternEncoder;
    use log4rs::Config;
    use mockall::mock;
    use popcorn_fx_torrent::torrent;
    use popcorn_fx_torrent::torrent::TorrentStats;
    use std::fmt::{Display, Formatter};
    use std::fs::OpenOptions;
    use std::io::Read;
    use std::ops::Range;
    use std::path::PathBuf;
    use std::sync::{Once, Weak};
    use std::time::Duration;
    use std::{env, fs};
    use tempfile::TempDir;
    use tokio::select;
    use tokio::sync::mpsc::UnboundedReceiver;
    use url::Url;

    static INIT: Once = Once::new();

    /// Initializes the logger with the specified log level.
    #[macro_export]
    macro_rules! init_logger {
        ($level:expr) => {
            popcorn_fx_core::testing::init_logger_level($level)
        };
        () => {
            popcorn_fx_core::testing::init_logger_level(log::LevelFilter::Trace)
        };
    }

    /// Initializes the logger with the specified log level.
    pub fn init_logger_level(level: LevelFilter) {
        INIT.call_once(|| {
            log4rs::init_config(Config::builder()
                .appender(Appender::builder().build("stdout", Box::new(ConsoleAppender::builder()
                    .encoder(Box::new(PatternEncoder::new("\x1B[37m{d(%Y-%m-%d %H:%M:%S%.3f)}\x1B[0m {h({l:>5.5})} \x1B[35m{I:>6.6}\x1B[0m \x1B[37m---\x1B[0m \x1B[37m[{T:>15.15}]\x1B[0m \x1B[36m{t:<60.60}\x1B[0m \x1B[37m:\x1B[0m {m}{n}")))
                    .build())))
                .logger(Logger::builder().build("async_io", LevelFilter::Info))
                .logger(Logger::builder().build("fx_callback", LevelFilter::Info))
                .logger(Logger::builder().build("httpmock::server", LevelFilter::Debug))
                .logger(Logger::builder().build("hyper", LevelFilter::Info))
                .logger(Logger::builder().build("hyper_util", LevelFilter::Info))
                .logger(Logger::builder().build("mdns_sd", LevelFilter::Info))
                .logger(Logger::builder().build("mio", LevelFilter::Info))
                .logger(Logger::builder().build("neli", LevelFilter::Info))
                .logger(Logger::builder().build("polling", LevelFilter::Info))
                .logger(Logger::builder().build("rustls", LevelFilter::Info))
                .logger(Logger::builder().build("serde_xml_rs", LevelFilter::Info))
                .logger(Logger::builder().build("tracing", LevelFilter::Info))
                .logger(Logger::builder().build("want", LevelFilter::Info))
                .build(Root::builder().appender("stdout").build(level))
                .unwrap())
                .unwrap();
        })
    }

    /// Copy a file from the test resources to the given temp directory.
    /// It will use the same `filename` as the source when `output_filename` is [None].
    ///
    /// * `filename`        - The original filename to copy
    /// * `output_filename` - The new filename within the temp directory
    pub fn copy_test_file(temp_dir: &str, filename: &str, output_filename: Option<&str>) -> String {
        let root_dir = &env::var("CARGO_MANIFEST_DIR").expect("$CARGO_MANIFEST_DIR");
        let source = PathBuf::from(root_dir).join("test").join(filename);
        let destination = PathBuf::from(temp_dir).join(output_filename.unwrap_or(filename));

        // make sure the parent dir exists
        fs::create_dir_all(destination.parent().unwrap()).unwrap();

        trace!("Copying test file {} to {:?}", filename, destination);
        fs::copy(&source, &destination).unwrap();

        destination.to_str().unwrap().to_string()
    }

    /// Retrieve the path to the testing resource directory.
    ///
    /// It returns the [PathBuf] to the testing resources directory.
    pub fn test_resource_directory() -> PathBuf {
        let root_dir = &env::var("CARGO_MANIFEST_DIR").expect("$CARGO_MANIFEST_DIR");
        let mut source = PathBuf::from(root_dir);
        source.push("test");

        source
    }

    /// Retrieve the filepath of a testing resource file.
    /// These are files located within the "test" directory of the crate.
    ///
    /// It returns the created [PathBuf] for the given filename.
    pub fn test_resource_filepath(filename: &str) -> PathBuf {
        let mut source = test_resource_directory();
        source.push(filename);

        source
    }

    /// Read a test resource file as a [String].
    pub fn read_test_file_to_string(filename: &str) -> String {
        let source = test_resource_filepath(filename);

        fs::read_to_string(&source).unwrap()
    }

    pub fn read_test_file_to_bytes(filename: &str) -> Vec<u8> {
        let source = test_resource_filepath(filename);

        fs::read(&source).unwrap()
    }

    /// Read a file from the temp directory.
    pub fn read_temp_dir_file_as_string(temp_dir: &TempDir, filename: &str) -> String {
        let path = temp_dir.path().join(filename);

        trace!("Reading temp filepath {:?}", path);
        if path.exists() {
            let mut content = String::new();
            match OpenOptions::new()
                .read(true)
                .open(&path)
                .unwrap()
                .read_to_string(&mut content)
            {
                Ok(e) => {
                    debug!("Read temp file {:?} with size {}", path, e);
                    content
                }
                Err(e) => panic!("Failed to read temp file, {}", e),
            }
        } else {
            panic!("Temp filepath {:?} does not exist", path)
        }
    }

    pub fn read_temp_dir_file_as_bytes(temp_dir: &TempDir, filename: &str) -> Vec<u8> {
        let path = temp_dir.path().join(filename);
        let mut buffer = vec![];

        trace!("Reading temp filepath {:?}", path);
        if path.exists() {
            match OpenOptions::new()
                .read(true)
                .open(&path)
                .unwrap()
                .read_to_end(&mut buffer)
            {
                Ok(e) => {
                    debug!("Read temp file {:?} with size {}", path, e);
                    buffer
                }
                Err(e) => panic!("Failed to read temp file, {}", e),
            }
        } else {
            panic!("Temp filepath {:?} does not exist", path)
        }
    }

    pub fn write_tmp_dir_file(temp_dir: &TempDir, filename: &str, contents: impl AsRef<[u8]>) {
        let path = temp_dir.path().join(filename);
        trace!("Writing test file {:?}", path);
        fs::write(path, contents).unwrap();
    }

    mock! {
        #[derive(Debug)]
        pub Player {}

        #[async_trait]
        impl Player for Player {
            fn id(&self) -> &str;
            fn name(&self) -> &str;
            fn description(&self) -> &str;
            fn graphic_resource(&self) -> Vec<u8>;
            async fn state(&self) -> PlayerState;
            async fn request(&self) -> Option<Weak<Box<dyn PlayRequest>>>;
            async fn play(&self, request: Box<dyn PlayRequest>);
            fn pause(&self);
            fn resume(&self);
            fn seek(&self, time: u64);
            fn stop(&self);
        }

        impl Callback<PlayerEvent> for Player {
            fn subscribe(&self) -> Subscription<PlayerEvent>;
            fn subscribe_with(&self, subscriber: Subscriber<PlayerEvent>);
        }
    }

    impl Display for MockPlayer {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "MockPlayer")
        }
    }

    mock! {
        #[derive(Debug)]
        pub SubtitleManager {}

        #[async_trait]
        impl SubtitleManager for SubtitleManager {
            async fn preference(&self) -> SubtitlePreference;
            async fn preference_async(&self) -> SubtitlePreference;
            async fn update_preference(&self, preference: SubtitlePreference);
            async fn select_or_default(&self, subtitles: &[SubtitleInfo]) -> SubtitleInfo;
            async fn reset(&self);
            async fn cleanup(&self);
        }

         impl Callbacks<SubtitleEvent> for SubtitleManager {
            fn add_callback(&self, callback: CoreCallback<SubtitleEvent>) -> CallbackHandle;
            fn remove_callback(&self, handle: CallbackHandle);
        }
    }

    mock! {
        #[derive(Debug)]
        pub InnerTorrentStream {
            pub fn stream_handle(&self) -> Handle;
            pub fn url(&self) -> Url;
            pub async fn stream(&self) -> torrents::Result<TorrentStreamingResourceWrapper>;
            pub async fn stream_offset(&self, offset: u64, len: Option<u64>) -> torrents::Result<TorrentStreamingResourceWrapper>;
            pub async fn stream_state(&self) -> TorrentStreamState;
            pub fn stop_stream(&self);
            pub fn subscribe_stream(&self) -> Subscription<TorrentStreamEvent>;
            pub fn subscribe_stream_with(&self, subscriber: Subscriber<TorrentStreamEvent>);
        }

        #[async_trait]
        impl Torrent for InnerTorrentStream {
            fn handle(&self) -> TorrentHandle;
            async fn files(&self) -> Vec<torrent::File>;
            async fn largest_file(&self) -> Option<torrent::File>;
            async fn has_bytes(&self, bytes: &std::ops::Range<usize>) -> bool;
            async fn has_piece(&self, piece: usize) -> bool;
            async fn prioritize_bytes(&self, bytes: &std::ops::Range<usize>);
            async fn prioritize_pieces(&self, pieces: &[u32]);
            async fn total_pieces(&self) -> usize;
            async fn sequential_mode(&self);
            async fn state(&self) -> TorrentState;
            async fn stats(&self) -> TorrentStats;
        }

        impl Callback<TorrentEvent> for InnerTorrentStream {
            fn subscribe(&self) -> Subscription<TorrentEvent>;
            fn subscribe_with(&self, subscriber: Subscriber<TorrentEvent>);
        }
    }

    #[derive(Debug)]
    pub struct MockTorrentStream {
        pub inner: MockInnerTorrentStream,
    }

    impl MockTorrentStream {
        pub fn new() -> Self {
            Self {
                inner: MockInnerTorrentStream::new(),
            }
        }
    }

    #[async_trait]
    impl Torrent for MockTorrentStream {
        fn handle(&self) -> TorrentHandle {
            self.inner.handle()
        }
        async fn files(&self) -> Vec<torrent::File> {
            self.inner.files().await
        }
        async fn largest_file(&self) -> Option<torrent::File> {
            self.inner.largest_file().await
        }
        async fn has_bytes(&self, bytes: &Range<usize>) -> bool {
            self.inner.has_bytes(bytes).await
        }
        async fn has_piece(&self, piece: usize) -> bool {
            self.inner.has_piece(piece).await
        }
        async fn prioritize_bytes(&self, bytes: &Range<usize>) {
            self.inner.prioritize_bytes(bytes).await
        }
        async fn prioritize_pieces(&self, pieces: &[u32]) {
            self.inner.prioritize_pieces(pieces).await
        }
        async fn total_pieces(&self) -> usize {
            self.inner.total_pieces().await
        }
        async fn sequential_mode(&self) {
            self.inner.sequential_mode().await
        }
        async fn state(&self) -> TorrentState {
            self.inner.state().await
        }
        async fn stats(&self) -> TorrentStats {
            self.inner.stats().await
        }
    }

    #[async_trait]
    impl TorrentStream for MockTorrentStream {
        fn stream_handle(&self) -> StreamHandle {
            self.inner.stream_handle()
        }

        fn url(&self) -> Url {
            self.inner.url()
        }

        async fn stream(&self) -> torrents::Result<TorrentStreamingResourceWrapper> {
            self.inner.stream().await
        }

        async fn stream_offset(
            &self,
            offset: u64,
            len: Option<u64>,
        ) -> torrents::Result<TorrentStreamingResourceWrapper> {
            self.inner.stream_offset(offset, len).await
        }

        async fn stream_state(&self) -> TorrentStreamState {
            self.inner.stream_state().await
        }

        fn stop_stream(&self) {
            self.inner.stop_stream()
        }
    }

    impl Callback<TorrentEvent> for MockTorrentStream {
        fn subscribe(&self) -> Subscription<TorrentEvent> {
            self.inner.subscribe()
        }

        fn subscribe_with(&self, subscriber: Subscriber<TorrentEvent>) {
            self.inner.subscribe_with(subscriber)
        }
    }

    impl Callback<TorrentStreamEvent> for MockTorrentStream {
        fn subscribe(&self) -> Subscription<TorrentStreamEvent> {
            self.inner.subscribe_stream()
        }

        fn subscribe_with(&self, subscriber: Subscriber<TorrentStreamEvent>) {
            self.inner.subscribe_stream_with(subscriber)
        }
    }

    impl Display for MockInnerTorrentStream {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "MockInnerTorrentStream")
        }
    }

    impl Display for MockTorrentStream {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "MockTorrentStream")
        }
    }

    mock! {
        #[derive(Debug)]
        pub DummyPlatform {}

        impl Platform for DummyPlatform {
            fn disable_screensaver(&self) -> bool;

            fn enable_screensaver(&self) -> bool;

            fn notify_media_event(&self, notification: MediaNotificationEvent);

            fn register(&self, callback: PlatformCallback);
        }
    }

    mock! {
        #[derive(Debug)]
        pub DummyPlatformData {}

        impl PlatformData for DummyPlatformData {
            fn info(&self) -> PlatformInfo;
        }

        impl Platform for DummyPlatformData {
            fn disable_screensaver(&self) -> bool;

            fn enable_screensaver(&self) -> bool;

            fn notify_media_event(&self, notification: MediaNotificationEvent);

            fn register(&self, callback: PlatformCallback);
        }
    }

    #[macro_export]
    macro_rules! assert_timeout {
        ($timeout:expr, $condition:expr) => {{
            let result = tokio::select! {
                _ = tokio::time::sleep($timeout) => false,
                result = async {
                    loop {
                        if $condition {
                            return true;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    }
                } => result,
            };

            if !result {
                assert!(false, "Timeout assertion failed after {:?}", $timeout);
            }
        }};
        ($timeout:expr, $condition:expr, $message:expr) => {{
            let result = tokio::select! {
                _ = tokio::time::sleep($timeout) => false,
                result = async {
                    loop {
                        if $condition {
                            return true;
                        }

                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    }
                } => result,
            };

            if !result {
                assert!(
                    false,
                    concat!("Timeout assertion failed after {:?}: ", $message),
                    $timeout
                );
            }
        }};
    }

    #[macro_export]
    macro_rules! assert_timeout_eq {
        ($timeout:expr, $left:expr, $right:expr) => {{
            let mut actual_value = $right;
            let result = tokio::select! {
                _ = tokio::time::sleep($timeout) => false,
                result = async {
                    loop {
                        actual_value = $right;
                        if $left == actual_value {
                            return true;
                        }

                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    }
                } => result,
            };

            if !result {
                assert!(
                    false,
                    "Assertion timed out after {:?}, expected {} but got {} instead",
                    $timeout, $left, actual_value
                );
            }
        }};
    }

    /// Receive a message from the given receiver, or panic if the timeout is reached.
    #[macro_export]
    macro_rules! recv_timeout {
        ($receiver:expr, $timeout:expr) => {
            $crate::testing::recv_timeout($receiver, $timeout, "expected to receive an instance")
                .await
        };
        ($receiver:expr, $timeout:expr, $message:expr) => {
            $crate::testing::recv_timeout($receiver, $timeout, $message).await
        };
    }

    /// Receive a message from the given receiver, or panic if the timeout is reached.
    ///
    /// # Arguments
    ///
    /// * `receiver` - The receiver to receive the message from.
    /// * `timeout` - The timeout to wait for the message.
    /// * `message` - The message to print if the timeout is reached.
    ///
    /// # Returns
    ///
    /// It returns the received instance of `T`.
    pub async fn recv_timeout<T>(
        receiver: &mut UnboundedReceiver<T>,
        timeout: Duration,
        message: &str,
    ) -> T {
        select! {
            _ = tokio::time::sleep(timeout) => panic!("receiver timed-out after {}ms, {}", timeout.as_millis(), message),
            result = receiver.recv() => result.expect(message)
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use httpmock::MockServer;
    use tempfile::TempDir;

    use crate::core::config::{ApplicationConfig, PopcornProperties, ProviderProperties};

    use super::*;

    pub fn start_mock_server(temp_dir: &TempDir) -> (MockServer, ApplicationConfig) {
        let server = MockServer::start();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = ApplicationConfig::builder()
            .storage(temp_path)
            .properties(PopcornProperties {
                loggers: Default::default(),
                update_channel: String::new(),
                providers: create_providers(&server),
                enhancers: Default::default(),
                subtitle: Default::default(),
                tracking: Default::default(),
            })
            .build();

        (server, settings)
    }

    fn create_providers(server: &MockServer) -> HashMap<String, ProviderProperties> {
        let mut map: HashMap<String, ProviderProperties> = HashMap::new();
        map.insert(
            "movies".to_string(),
            ProviderProperties {
                uris: vec![server.url("")],
                genres: vec![],
                sort_by: vec![],
            },
        );
        map.insert(
            "series".to_string(),
            ProviderProperties {
                uris: vec![server.url("")],
                genres: vec![],
                sort_by: vec![],
            },
        );
        map
    }

    #[repr(C)]
    #[derive(Debug, Clone, PartialEq)]
    struct Example {
        pub a: i32,
    }

    #[test]
    fn test_c_string() {
        let value = "lorem".to_string();

        let c = into_c_string(value.clone());
        let result = from_c_string(c);

        assert_eq!(value, result)
    }

    #[test]
    fn test_owned() {
        let value = Example { a: 13 };

        let c = into_c_owned(value.clone());
        let result = from_c_owned(c);

        assert_eq!(value, result)
    }

    #[test]
    fn test_owned_boxed() {
        let value = Example { a: 54 };

        let c = into_c_owned(value.clone());
        let result = from_c_into_boxed(c);

        assert_eq!(Box::new(value), result)
    }

    #[test]
    fn test_c_array() {
        let example = vec![0, 13, 5];

        let (ptr, len) = into_c_vec(example.clone());
        let result = from_c_vec(ptr, len);

        assert_eq!(example, result)
    }

    #[test]
    fn test_from_c_vec() {
        init_logger!();
        let value = Example { a: 25 };
        let array = vec![value.clone()];

        let (ptr, len) = into_c_vec(array);
        let result = from_c_vec(ptr, len);

        assert_eq!(
            &value,
            result
                .get(0)
                .expect("expected the value item to have been present")
        );
    }

    #[test]
    fn test_from_c_vec_owned() {
        init_logger!();
        let value = Example { a: 25 };
        let array = vec![value.clone()];

        let (ptr, len) = into_c_vec(array);
        let result = from_c_vec_owned(ptr, len);

        assert_eq!(
            &value,
            result
                .get(0)
                .expect("expected the value item to have been present")
        );
    }
}
