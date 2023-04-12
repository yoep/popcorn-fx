use std::collections::HashMap;

use serde::Deserialize;

/// The latest release version information.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct VersionInfo {
    /// The latest release version as a semantic representation
    pub version: String,
    /// The available platform updates
    pub platforms: HashMap<String, String>,
}

impl VersionInfo {
    /// Retrieve the version reference
    pub fn version(&self) -> &str {
        self.version.as_str()
    }
}