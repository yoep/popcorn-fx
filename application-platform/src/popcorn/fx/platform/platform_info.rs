use log::trace;

#[cfg(target_arch = "x86_64")]
const X64: &str = "x86-64";
#[cfg(target_arch = "arm")]
const ARM: &str = "arm";
#[cfg(target_arch = "aarch64")]
const ARCH64: &str = "aarch64";

/// The platform type
#[repr(C)]
#[derive(PartialEq)]
pub enum PlatformType {
    /// The windows platform
    Windows = 0,
    /// The macos platform
    MacOs = 1,
    /// The linux platform
    Linux = 2,
}

/// PlatformInfo defines the info of the current platform
pub struct PlatformInfo {
    /// The platform type
    pub platform_type: PlatformType,
    /// The cpu architecture of the platform
    pub arch: String,
}

impl PlatformInfo {
    /// Create a new platform information instance
    #[cfg(target_os = "windows")]
    #[cfg(target_arch = "x86_64")]
    pub fn new() -> PlatformInfo {
        trace!("Retrieving windows platform info");
        PlatformInfo {
            platform_type: PlatformType::Windows,
            arch: String::from(X64),
        }
    }

    /// Create a new platform information instance
    #[cfg(target_os = "macos")]
    #[cfg(target_arch = "x86_64")]
    pub fn new() -> PlatformInfo {
        trace!("Retrieving macos platform info");
        PlatformInfo {
            platform_type: PlatformType::MacOs,
            arch: String::from(X64),
        }
    }

    /// Create a new platform information instance
    #[cfg(target_os = "linux")]
    #[cfg(target_arch = "x86_64")]
    pub fn new() -> PlatformInfo {
        trace!("Retrieving linux platform info");
        PlatformInfo {
            platform_type: PlatformType::Linux,
            arch: String::from(X64),
        }
    }

    /// Create a new platform information instance
    #[cfg(target_os = "linux")]
    #[cfg(target_arch = "aarch64")]
    pub fn new() -> PlatformInfo {
        trace!("Retrieving linux platform info");
        PlatformInfo {
            platform_type: PlatformType::Linux,
            arch: String::from(ARCH64.to_string()),
        }
    }

    /// Create a new platform information instance
    #[cfg(target_os = "linux")]
    #[cfg(target_arch = "arm")]
    pub fn new() -> PlatformInfo {
        trace!("Retrieving linux platform info");
        PlatformInfo {
            platform_type: PlatformType::Linux,
            arch: String::from(ARM.to_string()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn test_platform_info_new_should_return_windows_info() {
        let info = PlatformInfo::new();

        assert!(matches!(info.platform_type, PlatformType::Windows));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_platform_info_new_should_return_linux_info() {
        let info = PlatformInfo::new();

        assert!(matches!(info.platform_type, PlatformType::Linux));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_platform_info_new_should_return_macos_info() {
        let info = PlatformInfo::new();

        assert!(matches!(info.platform_type, PlatformType::MacOs));
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_platform_info_new_should_return_x64_info() {
        let info = PlatformInfo::new();

        assert_eq!(X64, String::from(info.arch))
    }

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn test_platform_info_new_should_return_aarch64_info() {
        let info = PlatformInfo::new();

        assert_eq!(ARCH64, String::from(info.arch))
    }

    #[test]
    #[cfg(target_arch = "arm")]
    fn test_platform_info_new_should_return_arm_info() {
        let info = PlatformInfo::new();

        assert_eq!(ARM, String::from(info.arch))
    }
}