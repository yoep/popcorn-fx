use std::os::raw::c_char;

use popcorn_fx_core::{from_c_string, into_c_string, to_c_vec};
use popcorn_fx_core::core::updater::{ChangeLog, VersionInfo};

/// The version information from the update channel.
#[repr(C)]
#[derive(Debug)]
pub struct VersionInfoC {
    /// The latest release version on the update channel
    pub version: *const c_char,
    pub changelog: ChangelogC,
}

impl From<&VersionInfo> for VersionInfoC {
    fn from(value: &VersionInfo) -> Self {
        Self {
            version: into_c_string(value.version().to_string()),
            changelog: ChangelogC::from(&value.changelog),
        }
    }
}

impl PartialEq for VersionInfoC {
    fn eq(&self, other: &Self) -> bool {
        from_c_string(self.version) == from_c_string(other.version)
    }
}

/// The C compatible changelog
#[repr(C)]
#[derive(Debug)]
pub struct ChangelogC {
    /// The new features array string
    pub features: *mut *const c_char,
    /// The length of the features array
    pub features_len: i32,
    /// The new bugfixes array string
    pub bugfixes: *mut *const c_char,
    /// The length of the bugfixes array
    pub bugfixes_len: i32,
}

impl From<&ChangeLog> for ChangelogC {
    fn from(value: &ChangeLog) -> Self {
        let (features, features_len) = to_c_vec(value.features.iter()
            .map(|e| into_c_string(e.clone()))
            .collect());
        let (bugfixes, bugfixes_len) = to_c_vec(value.bugfixes.iter()
            .map(|e| into_c_string(e.clone()))
            .collect());

        Self {
            features,
            features_len,
            bugfixes,
            bugfixes_len,
        }
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
            changelog: ChangeLog {
                features: features.clone(),
                bugfixes: vec![],
            },
            platforms: Default::default(),
        };

        let result = VersionInfoC::from(&version_info);
        let features_result: Vec<String> = from_c_vec(result.changelog.features, result.changelog.features_len).into_iter()
            .map(|e| from_c_string(e))
            .collect();

        assert_eq!(version.to_string(), from_c_string(result.version));
        assert_eq!(features, features_result);
    }

    #[test]
    fn test_from_changelog() {
        let features = vec!["lorem".to_string()];
        let bugfixes = vec!["ipsum".to_string()];
        let changelog = ChangeLog {
            features: features.clone(),
            bugfixes: bugfixes.clone(),
        };

        let result = ChangelogC::from(&changelog);
        let features_result: Vec<String> = from_c_vec(result.features, result.features_len).into_iter()
            .map(|e| from_c_string(e))
            .collect();
        let bugfixes_result: Vec<String> = from_c_vec(result.bugfixes, result.bugfixes_len).into_iter()
            .map(|e| from_c_string(e))
            .collect();

        assert_eq!(features, features_result);
        assert_eq!(bugfixes, bugfixes_result);
    }
}