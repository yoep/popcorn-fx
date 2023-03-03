use std::collections::HashMap;

use serde::Deserialize;

const DEFAULT_CHANGELOG: fn() -> Vec<String> = Vec::new;

/// The changelog information of a certain application release.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ChangeLog {
    /// The new features of the release
    #[serde(default = "DEFAULT_CHANGELOG")]
    pub features: Vec<String>,
    /// The new bugfixes of the release
    #[serde(default = "DEFAULT_CHANGELOG")]
    pub bugfixes: Vec<String>,
}

/// The latest release version information.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct VersionInfo {
    /// The latest release version as a semantic representation
    pub version: String,
    /// The changelog information of the release
    pub changelog: ChangeLog,
    /// The available platform updates
    pub platforms: HashMap<String, String>,
}

impl VersionInfo {
    /// Retrieve the version reference
    pub fn version(&self) -> &str {
        self.version.as_str()
    }
}