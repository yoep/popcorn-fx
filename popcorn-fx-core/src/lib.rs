use std::{mem, ptr};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use log::{error, trace};

pub use popcorn_fx_common::VERSION;

pub use crate::torrent_collection_c::*;

pub mod core;

mod torrent_collection_c;

/// Convert the given [String] into a C compatible string.
///
/// This function will consume the provided data and use the underlying bytes to construct a new string, ensuring that there is a trailing 0 byte.
/// This trailing 0 byte will be appended by this function; the provided data should not contain any 0 bytes in it.
pub fn into_c_string(value: String) -> *const c_char {
    // DO NOT use [CString::into_raw] as Rust still cleans the original string which the pointer uses
    let c_string = CString::new(value.as_bytes()).unwrap();
    let ptr = c_string.as_ptr();
    mem::forget(c_string);
    ptr
}

/// Convert the given C string to an owned rust [String].
pub fn from_c_string(ptr: *const c_char) -> String {
    if !ptr.is_null() {
        let slice = unsafe { CStr::from_ptr(ptr).to_bytes() };

        return std::str::from_utf8(slice)
            .map(|e| e.to_string())
            .unwrap_or_else(|e| {
                error!("Failed to read C string, using empty string instead ({})", e);
                String::new()
            });
    } else {
        error!("Unable to read C string, pointer is null");
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
        let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len as usize) };
        slice.into()
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
pub fn from_c_vec_owned<T: Clone>(ptr: *mut T, len: i32) -> Vec<T> {
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
    use std::{env, fs};
    use std::fmt::{Display, Formatter};
    use std::fs::OpenOptions;
    use std::io::Read;
    use std::path::PathBuf;
    use std::sync::{Once, Weak};

    use async_trait::async_trait;
    use log::{debug, LevelFilter, trace};
    use log4rs::append::console::ConsoleAppender;
    use log4rs::Config;
    use log4rs::config::{Appender, Logger, Root};
    use log4rs::encode::pattern::PatternEncoder;
    use mockall::mock;
    use tempfile::TempDir;

    use crate::core::{CallbackHandle, Callbacks, CoreCallback};
    use crate::core::players::{Player, PlayerEvent, PlayerState, PlayRequest};
    use crate::core::subtitles::{SubtitleEvent, SubtitleManager};
    use crate::core::subtitles::language::SubtitleLanguage;
    use crate::core::subtitles::model::SubtitleInfo;

    static INIT: Once = Once::new();

    pub fn init_logger() {
        INIT.call_once(|| {
            log4rs::init_config(Config::builder()
                .appender(Appender::builder().build("stdout", Box::new(ConsoleAppender::builder()
                    .encoder(Box::new(PatternEncoder::new("\x1B[37m{d(%Y-%m-%d %H:%M:%S%.3f)}\x1B[0m {h({l:>5.5})} \x1B[35m{I:>6.6}\x1B[0m \x1B[37m---\x1B[0m \x1B[37m[{T:>15.15}]\x1B[0m \x1B[36m{t:<40.40}\x1B[0m \x1B[37m:\x1B[0m {m}{n}")))
                    .build())))
                .logger(Logger::builder().build("httpmock::server", LevelFilter::Debug))
                .logger(Logger::builder().build("want", LevelFilter::Info))
                .logger(Logger::builder().build("polling", LevelFilter::Info))
                .logger(Logger::builder().build("hyper", LevelFilter::Info))
                .logger(Logger::builder().build("tracing", LevelFilter::Info))
                .build(Root::builder().appender("stdout").build(LevelFilter::Trace))
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
        let source = PathBuf::from(root_dir)
            .join("test")
            .join(filename);
        let destination = PathBuf::from(temp_dir)
            .join(output_filename.unwrap_or(filename));

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
                .read_to_string(&mut content) {
                Ok(e) => {
                    debug!("Read temp file {:?} with size {}", path, e);
                    content
                }
                Err(e) => panic!("Failed to read temp file, {}", e)
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
                .read_to_end(&mut buffer) {
                Ok(e) => {
                    debug!("Read temp file {:?} with size {}", path, e);
                    buffer
                }
                Err(e) => panic!("Failed to read temp file, {}", e)
            }
        } else {
            panic!("Temp filepath {:?} does not exist", path)
        }
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
            fn state(&self) -> PlayerState;
            fn request(&self) -> Option<Weak<Box<dyn PlayRequest>>>;
            async fn play(&self, request: Box<dyn PlayRequest>);
            fn pause(&self);
            fn resume(&self);
            fn seek(&self, time: u64);
            fn stop(&self);
        }

        impl Callbacks<PlayerEvent> for Player {
            fn add(&self, callback: CoreCallback<PlayerEvent>) -> CallbackHandle;
            fn remove(&self, handle: CallbackHandle);
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
            fn preferred_subtitle(&self) -> Option<SubtitleInfo>;
            fn preferred_language(&self) -> SubtitleLanguage;
            fn is_disabled(&self) -> bool;
            async fn is_disabled_async(&self) -> bool;
            fn update_subtitle(&self, subtitle: SubtitleInfo);
            fn update_custom_subtitle(&self, subtitle_file: &str);
            fn disable_subtitle(&self);
            fn reset(&self);
            fn cleanup(&self);
        }

         impl Callbacks<SubtitleEvent> for SubtitleManager {
            fn add(&self, callback: CoreCallback<SubtitleEvent>) -> CallbackHandle;
            fn remove(&self, handle: CallbackHandle);
        }
    }

    #[macro_export]
    macro_rules! assert_timeout {
    ($timeout:expr, $condition:expr) => {{
        use std::thread;
        use std::time::{Duration, Instant};

        let start_time = Instant::now();
        let timeout: Duration = $timeout;

        let result = loop {
            if $condition {
                break true;
            }
            if start_time.elapsed() >= timeout {
                break false;
            }
            thread::sleep(Duration::from_millis(10));
        };

        if !result {
            assert!(false, "Timeout assertion failed after {:?}", $timeout);
        }
    }};
    ($timeout:expr, $condition:expr, $message:expr) => {{
        use std::thread;
        use std::time::{Duration, Instant};

        let start_time = Instant::now();
        let timeout: Duration = $timeout;

        let result = loop {
            if $condition {
                break true;
            }
            if start_time.elapsed() >= timeout {
                break false;
            }
            thread::sleep(Duration::from_millis(10));
        };

        if !result {
            assert!(false, concat!("Timeout assertion failed after {:?}: ", $message), $timeout);
        }
    }};
    }

    #[macro_export]
    macro_rules! assert_timeout_eq {
    ($timeout:expr, $left:expr, $right:expr) => {{
        use std::thread;
        use std::time::{Duration, Instant};

        let start_time = Instant::now();
        let timeout: Duration = $timeout;
        let mut actual_value;

        let result = loop {
            actual_value = $right;
            if $left == actual_value {
                break true;
            }
            if start_time.elapsed() >= timeout {
                break false;
            }
            thread::sleep(Duration::from_millis(10));
        };

        if !result {
            assert!(false, "Assertion timed out after {:?}, expected {} but got {} instead", $timeout, $left, actual_value);
        }
    }};
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::sync::Arc;

    use httpmock::MockServer;
    use tempfile::TempDir;

    use crate::core::config::{ApplicationConfig, PopcornProperties, ProviderProperties};
    use crate::testing::init_logger;

    use super::*;

    pub fn start_mock_server(temp_dir: &TempDir) -> (MockServer, Arc<ApplicationConfig>) {
        let server = MockServer::start();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = Arc::new(ApplicationConfig::builder()
            .storage(temp_path)
            .properties(PopcornProperties {
                loggers: Default::default(),
                update_channel: String::new(),
                providers: create_providers(&server),
                enhancers: Default::default(),
                subtitle: Default::default(),
            })
            .build());

        (server, settings)
    }

    fn create_providers(server: &MockServer) -> HashMap<String, ProviderProperties> {
        let mut map: HashMap<String, ProviderProperties> = HashMap::new();
        map.insert("movies".to_string(), ProviderProperties {
            uris: vec![
                server.url("")
            ],
            genres: vec![],
            sort_by: vec![],
        });
        map.insert("series".to_string(), ProviderProperties {
            uris: vec![
                server.url("")
            ],
            genres: vec![],
            sort_by: vec![],
        });
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
        let value = Example {
            a: 13
        };

        let c = into_c_owned(value.clone());
        let result = from_c_owned(c);

        assert_eq!(value, result)
    }

    #[test]
    fn test_owned_boxed() {
        let value = Example {
            a: 54
        };

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
        init_logger();
        let value = Example {
            a: 25,
        };
        let array = vec![value.clone()];

        let (ptr, len) = into_c_vec(array);
        let result = from_c_vec(ptr, len);

        assert_eq!(&value, result.get(0).expect("expected the value item to have been present"));
    }

    #[test]
    fn test_from_c_vec_owned() {
        init_logger();
        let value = Example {
            a: 25,
        };
        let array = vec![value.clone()];

        let (ptr, len) = into_c_vec(array);
        let result = from_c_vec_owned(ptr, len);

        assert_eq!(&value, result.get(0).expect("expected the value item to have been present"));
    }
}