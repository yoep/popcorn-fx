use std::os::raw::c_char;

use log::{debug, error, info, trace, warn};

use popcorn_fx_core::from_c_string;

use crate::ffi::LogLevel;

/// Logs a message sent over FFI using the Rust logger.
///
/// # Arguments
///
/// * `message` - A pointer to the null-terminated C string containing the log message to be logged.
/// * `level` - The log level of the message. Determines the verbosity of the message and how it will be formatted by the Rust logger.
#[no_mangle]
pub extern "C" fn log(target: *const c_char, message: *const c_char, level: LogLevel) {
    let target = from_c_string(target);
    let message = from_c_string(message);
    match level {
        LogLevel::Trace => trace!(target: target.as_str(), "{}", message),
        LogLevel::Debug => debug!(target: target.as_str(), "{}",message),
        LogLevel::Info => info!(target: target.as_str(), "{}",message),
        LogLevel::Warn => warn!(target: target.as_str(), "{}",message),
        LogLevel::Error => error!(target: target.as_str(), "{}",message),
        _ => {}
    }
}

#[cfg(test)]
mod test {
    use popcorn_fx_core::into_c_string;
    use popcorn_fx_core::testing::init_logger;

    use crate::ffi::LogLevel::{Debug, Error, Info, Trace, Warn};

    use super::*;

    #[test]
    fn test_log() {
        init_logger();

        log(into_c_string("ffi::test1".to_string()),into_c_string("lorem".to_string()), Trace);
        log(into_c_string("ffi::test2".to_string()),into_c_string("ipsum".to_string()), Debug);
        log(into_c_string("ffi::test3".to_string()),into_c_string("dolor".to_string()), Info);
        log(into_c_string("ffi::test4".to_string()),into_c_string("sit".to_string()), Warn);
        log(into_c_string("ffi::test5".to_string()),into_c_string("amet".to_string()), Error);
    }
}