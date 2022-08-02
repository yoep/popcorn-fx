use std::ffi::CString;
use std::os::raw::c_char;

use crate::{PlatformInfo, PlatformType};

#[repr(C)]
pub struct PlatformInfoC {
    /// The platform type
    pub platform_type: PlatformType,
    /// The cpu architecture of the platform
    pub arch: *const c_char,
}

impl PlatformInfoC {
    pub fn from(info: PlatformInfo) -> PlatformInfoC {
        PlatformInfoC {
            platform_type: info.platform_type,
            arch: CString::new(info.arch).unwrap().into_raw(),
        }
    }
}