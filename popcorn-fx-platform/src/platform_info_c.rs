use std::ffi::CString;
use std::os::raw::c_char;

use crate::popcorn::fx::platform::platform_info::{PlatformInfo, PlatformType};

#[repr(C)]
pub struct PlatformInfoC {
    /// The platform type
    pub platform_type: PlatformType,
    /// The cpu architecture of the platform
    pub arch: *const c_char,
}

impl PlatformInfoC {
    pub fn from(info: &PlatformInfo) -> PlatformInfoC {
        PlatformInfoC {
            platform_type: info.platform_type.clone(),
            arch: match CString::new(info.arch.clone()) {
                Err(ex) => panic!("failed to transform arch string to cstring, {}", ex),
                Ok(string) => string.into_raw(),
            },
        }
    }
}