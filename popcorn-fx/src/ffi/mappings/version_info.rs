use std::os::raw::c_char;

use popcorn_fx_core::{from_c_string, into_c_string};
use popcorn_fx_core::core::updater::{PatchInfo, VersionInfo};

/// The C compatible representation of version information from the update channel.
#[repr(C)]
#[derive(Debug)]
pub struct VersionInfoC {
    /// The latest release version on the update channel.
    pub application: PatchInfoC,
    /// The runtime version of the application.
    pub runtime: PatchInfoC,
}

impl From<&VersionInfo> for VersionInfoC {
    /// Convert a `VersionInfo` instance to a C-compatible `VersionInfoC` instance.
    fn from(value: &VersionInfo) -> Self {
        Self {
            application: PatchInfoC::from(&value.application),
            runtime: PatchInfoC::from(&value.runtime),
        }
    }
}

impl PartialEq for VersionInfoC {
    /// Check whether two `VersionInfoC` instances are equal.
    fn eq(&self, other: &Self) -> bool {
        from_c_string(self.application.version) == from_c_string(other.application.version) &&
            from_c_string(self.runtime.version) == from_c_string(other.runtime.version)
    }
}

/// The C compatible representation of the application runtime information.
#[repr(C)]
#[derive(Debug)]
pub struct PatchInfoC {
    /// The runtime version of the application.
    pub version: *mut c_char,
}

impl From<&PatchInfo> for PatchInfoC {
    /// Convert a `RuntimeInfo` instance to a C-compatible `RuntimeInfoC` instance.
    fn from(value: &PatchInfo) -> Self {
        Self {
            version: into_c_string(value.version().to_string()),
        }
    }
}

#[cfg(test)]
mod test {
    use popcorn_fx_core::from_c_string;

    use super::*;

    #[test]
    fn test_from_version_info() {
        let version = "1.0.5";
        let runtime_version = "10.0.3";
        let version_info = VersionInfo {
            application: PatchInfo {
                version: version.to_string(),
                platforms: Default::default(),
            },
            runtime: PatchInfo {
                version: runtime_version.to_string(),
                platforms: Default::default(),
            },
        };

        let result = VersionInfoC::from(&version_info);

        assert_eq!(version.to_string(), from_c_string(result.application.version));
        assert_eq!(runtime_version.to_string(), from_c_string(result.runtime.version));
    }
}