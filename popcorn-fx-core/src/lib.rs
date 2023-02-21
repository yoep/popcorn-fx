use std::{mem, ptr};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use log::error;

pub use crate::event_c::*;
pub use crate::media_c::*;
pub use crate::properties_c::*;
pub use crate::subtitle_c::*;
pub use crate::torrent_collection_c::*;

pub mod core;

mod event_c;
mod media_c;
mod properties_c;
mod subtitle_c;
mod torrent_collection_c;

/// Convert the given [String] into a C compatible string.
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

        match std::str::from_utf8(slice) {
            Ok(e) => e.to_string(),
            Err(e) => {
                error!("Failed to read C string, using empty string instead ({})", e);
                String::new()
            }
        }
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

/// Retrieve a C value as an [Box]] value.
/// For more info, see [Box::from_raw].
///
/// * `ptr` - The pointer value to convert
pub fn from_c_into_boxed<T>(ptr: *mut T) -> Box<T> {
    if !ptr.is_null() {
        unsafe { Box::from_raw(ptr) }
    } else {
        panic!("Unable to read C instance, pointer is null")
    }
}

/// Convert the given [Vec] into a C array tuple which is owned by the caller.
/// The return tuple is as follows: `(pointer, length)`
pub fn to_c_vec<T>(vec: Vec<T>) -> (*mut T, i32) {
    // check if the vec contains items
    // if not, we return a ptr::null as ABI can't handle empty arrays
    if !vec.is_empty() {
        let mut boxed = vec.into_boxed_slice();
        let ptr = boxed.as_mut_ptr();
        let len = boxed.len() as i32;
        mem::forget(boxed);

        (ptr, len)
    } else {
        (ptr::null_mut(), 0)
    }
}

/// Convert the given C array into an owned [Vec].
/// For more info, see [Vec::from_raw_parts].
///
/// It returns the [Vec] on success, else an empty vec if the `ptr` is `null`.
pub fn from_c_vec<T: Clone>(ptr: *mut T, len: i32) -> Vec<T> {
    if !ptr.is_null() {
        let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len as usize) };
        slice.into()
    } else {
        error!("Unable to read C array, array pointer is null");
        vec![]
    }
}

#[cfg(feature = "testing")]
pub mod testing {
    use std::{env, fs};
    use std::path::PathBuf;
    use std::sync::Once;

    use log4rs::append::console::ConsoleAppender;
    use log4rs::Config;
    use log4rs::config::{Appender, Root};
    use log::LevelFilter;

    static INIT: Once = Once::new();

    pub fn init_logger() {
        INIT.call_once(|| {
            log4rs::init_config(Config::builder()
                .appender(Appender::builder().build("stdout", Box::new(ConsoleAppender::builder().build())))
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
        let mut source = PathBuf::from(root_dir);
        source.push("test");
        source.push(filename);
        let mut destination = PathBuf::from(temp_dir);
        destination.push(output_filename.or_else(|| Some(filename)).unwrap());

        fs::copy(&source, &destination).unwrap();

        destination.as_path().to_str().unwrap().to_string()
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
    pub fn read_test_file(filename: &str) -> String {
        let source = test_resource_filepath(filename);

        fs::read_to_string(&source).unwrap()
    }

    /// Read a file from the temp directory.
    pub fn read_temp_dir_file(temp_dir: PathBuf, filename: &str) -> String {
        let mut path = temp_dir.clone();
        path.push(filename);

        fs::read_to_string(&path).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Example {
        a: i32,
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

        let (ptr, len) = to_c_vec(example.clone());
        let result = from_c_vec(ptr, len);

        assert_eq!(example, result)
    }
}