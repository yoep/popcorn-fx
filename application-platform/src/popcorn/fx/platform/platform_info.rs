use std::ffi::CString;
use std::os::raw::c_char;

#[cfg(target_arch = "x86_64")]
const X64: &str = "x86-64";
#[cfg(target_arch = "arm")]
const ARM: &str = "arm";
#[cfg(target_arch = "aarch64")]
const ARCH64: &str = "aarch64";

#[repr(C)]
pub enum PlatformType {
    Windows = 0,
    MacOs = 1,
    Linux = 2,
}

#[repr(C)]
pub struct PlatformInfo {
    pub platform_type: PlatformType,
    pub arch: *const c_char
}

impl PlatformInfo {
    #[cfg(target_os = "windows")]
    #[cfg(target_arch = "x86_64")]
    pub fn new() -> PlatformInfo {
        PlatformInfo {
            platform_type: PlatformType::Windows,
            arch: CString::new(X64.to_string()).unwrap().into_raw()
        }
    }

    #[cfg(target_os = "macos")]
    #[cfg(target_arch = "x86_64")]
    pub fn new() -> PlatformInfo {
        PlatformInfo {
            platform_type: PlatformType::MacOs,
            arch: CString::new(X64.to_string()).unwrap().into_raw()
        }
    }

    #[cfg(target_os = "linux")]
    #[cfg(target_arch = "x86_64")]
    pub fn new() -> PlatformInfo {
        PlatformInfo {
            platform_type: PlatformType::Linux,
            arch: CString::new(X64.to_string()).unwrap().into_raw()
        }
    }

    #[cfg(target_os = "linux")]
    #[cfg(target_arch = "aarch64")]
    pub fn new() -> PlatformInfo {
        PlatformInfo {
            platform_type: PlatformType::Linux,
            arch: CString::new(ARCH64.to_string()).unwrap().into_raw()
        }
    }

    #[cfg(target_os = "linux")]
    #[cfg(target_arch = "arm")]
    pub fn new() -> PlatformInfo {
        PlatformInfo {
            platform_type: PlatformType::Linux,
            arch: CString::new(ARM.to_string()).unwrap().into_raw()
        }
    }
}