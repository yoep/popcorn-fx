use std::collections::HashMap;

use serde::Deserialize;

/// Latest release version information, including version number, platform updates, and runtime information.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct VersionInfo {
    /// The latest release version number in semantic format.
    pub version: String,
    /// Available platform updates, with keys representing the platform name and values representing the update version.
    /// This is the legacy update information of the application and no longer used.
    pub platforms: HashMap<String, String>,
    /// Available platform patch updates of the application, with keys representing the platform name and values representing the update version.
    pub patch: HashMap<String, String>,
    /// Runtime information for the latest version.
    pub runtime: RuntimeInfo,
}

impl VersionInfo {
    /// Retrieves the version number for the latest release.
    ///
    /// # Returns
    ///
    /// A string slice representing the version number in semantic format.
    pub fn version(&self) -> &str {
        self.version.as_str()
    }
}

/// Runtime update information for the latest version, including runtime version and OS-specific downloads.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct RuntimeInfo {
    /// The version of the runtime to use.
    pub version: String,
    /// Available OS-specific downloads for the runtime, with keys representing the platform name and values representing the download URL.
    pub platforms: HashMap<String, String>,
}

impl RuntimeInfo {
    /// Retrieves the version number of the runtime.
    ///
    /// # Returns
    ///
    /// A string slice representing the version number in semantic format.
    pub fn version(&self) -> &str {
        self.version.as_str()
    }
}