use std::os::raw::c_char;

use popcorn_fx_core::{from_c_string, into_c_string};
use popcorn_fx_core::core::updater::VersionInfo;

/// The version information from the update channel.
#[repr(C)]
#[derive(Debug)]
pub struct VersionInfoC {
    /// The latest release version on the update channel
    pub version: *const c_char,
}

impl From<&VersionInfo> for VersionInfoC {
    fn from(value: &VersionInfo) -> Self {
        Self {
            version: into_c_string(value.version().to_string()),
        }
    }
}

impl PartialEq for VersionInfoC {
    fn eq(&self, other: &Self) -> bool {
        from_c_string(self.version) == from_c_string(other.version)
    }
}

#[cfg(test)]
mod test {
    use popcorn_fx_core::{from_c_string, from_c_vec};

    use super::*;

    #[test]
    fn test_from_version_info() {
        let version = "1.0.5";
        let features = vec!["lorem".to_string()];
        let version_info = VersionInfo {
            version: version.to_string(),
            platforms: Default::default(),
        };

        let result = VersionInfoC::from(&version_info);

        assert_eq!(version.to_string(), from_c_string(result.version));
    }
}