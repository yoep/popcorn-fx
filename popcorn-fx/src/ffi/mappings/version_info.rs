use std::os::raw::c_char;

use popcorn_fx_core::{from_c_string, into_c_string};
use popcorn_fx_core::core::updater::{RuntimeInfo, VersionInfo};

/// The C compatible representation of version information from the update channel.
#[repr(C)]
#[derive(Debug)]
pub struct VersionInfoC {
    /// The latest release version on the update channel.
    pub version: *const c_char,
    /// The runtime version of the application.
    pub runtime: RuntimeInfoC,
}

impl From<&VersionInfo> for VersionInfoC {
    /// Convert a `VersionInfo` instance to a C-compatible `VersionInfoC` instance.
    fn from(value: &VersionInfo) -> Self {
        Self {
            version: into_c_string(value.version().to_string()),
            runtime: RuntimeInfoC::from(&value.runtime),
        }
    }
}

impl PartialEq for VersionInfoC {
    /// Check whether two `VersionInfoC` instances are equal.
    fn eq(&self, other: &Self) -> bool {
        from_c_string(self.version) == from_c_string(other.version)
    }
}

/// The C compatible representation of the application runtime information.
#[repr(C)]
#[derive(Debug)]
pub struct RuntimeInfoC {
    /// The runtime version of the application.
    pub version: *const c_char,
}

impl From<&RuntimeInfo> for RuntimeInfoC {
    /// Convert a `RuntimeInfo` instance to a C-compatible `RuntimeInfoC` instance.
    fn from(value: &RuntimeInfo) -> Self {
        Self {
            version: into_c_string(value.version().to_string()),
        }
    }
}

#[cfg(test)]
mod test {
    use popcorn_fx_core::core::updater::RuntimeInfo;
    use popcorn_fx_core::from_c_string;

    use super::*;

    #[test]
    fn test_from_version_info() {
        let version = "1.0.5";
        let runtime_version = "10.0.3";
        let features = vec!["lorem".to_string()];
        let version_info = VersionInfo {
            version: version.to_string(),
            platforms: Default::default(),
            runtime: RuntimeInfo {
                version: runtime_version.to_string(),
                platforms: Default::default(),
            },
        };

        let result = VersionInfoC::from(&version_info);

        assert_eq!(version.to_string(), from_c_string(result.version));
        assert_eq!(runtime_version.to_string(), from_c_string(result.runtime.version));
    }
}