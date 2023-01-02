use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::c_char;

use log::error;

pub use crate::media_c::*;
pub use crate::subtitle_c::*;

pub mod core;
pub mod observer;

mod media_c;
mod subtitle_c;

/// Convert the given [String] into a C compatible string.
pub fn to_c_string(value: String) -> *const c_char {
    // DO NOT use [CString::into_raw] as Rust still cleans the original string which the pointer uses
    let c_string = CString::new(value.as_bytes()).unwrap();
    let ptr = c_string.as_ptr();
    mem::forget(c_string);
    ptr
}

/// Convert the given C string to an owned rust [String].
pub fn from_c_string(ptr: *const c_char) -> String {
    let slice = unsafe { CStr::from_ptr(ptr).to_bytes() };

    match std::str::from_utf8(slice) {
        Ok(e) => e.to_string(),
        Err(e) => {
            error!("Failed to read C string, using empty string instead ({})", e);
            String::new()
        }
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
/// * `ptr` - The pointer value to convert
pub fn from_c_owned<T>(ptr: *mut T) -> T {
    let value = unsafe { Box::from_raw(ptr) };
    *value
}

/// Convert the given [Vec] into a C array tuple which is owned by the caller.
/// The return tuple is as follows: `(pointer, length, capacity)`
pub fn to_c_vec<T>(mut vec: Vec<T>) -> (*mut T, i32, i32) {
    let ptr = vec.as_mut_ptr();
    let len = vec.len() as i32;
    let capacity = vec.capacity() as i32;
    mem::forget(vec);

    (ptr, len, capacity)
}

#[cfg(feature = "testing")]
pub mod test {
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

    pub fn copy_test_file(temp_dir: &str, filename: &str) -> String {
        let root_dir = &env::var("CARGO_MANIFEST_DIR").expect("$CARGO_MANIFEST_DIR");
        let mut source = PathBuf::from(root_dir);
        source.push("test");
        source.push(filename);
        let mut destination = PathBuf::from(temp_dir);
        destination.push(filename);

        fs::copy(&source, &destination).unwrap();

        destination.as_path().to_str().unwrap().to_string()
    }

    pub fn read_test_file(filename: &str) -> String {
        let root_dir = &env::var("CARGO_MANIFEST_DIR").expect("$CARGO_MANIFEST_DIR");
        let mut source = PathBuf::from(root_dir);
        source.push("test");
        source.push(filename);

        fs::read_to_string(&source).unwrap()
    }
}