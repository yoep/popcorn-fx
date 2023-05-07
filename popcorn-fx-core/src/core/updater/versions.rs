use std::collections::HashMap;

use serde::Deserialize;

/// Latest release version information, including version number, platform updates, and runtime information.
///
/// Use this struct to represent information about the latest release version of an application, including the version number, platform updates, and runtime information.
///
/// # Fields
///
/// * `application` - Information about the application update, including the version number and platform-specific updates.
/// * `runtime` - Information about the runtime update, including the version number and platform-specific downloads.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct VersionInfo {
    pub application: PatchInfo,
    pub runtime: PatchInfo,
}

/// The patch information for the latest version, including the OS-specific downloads.
///
/// Use this struct to represent the patch information for the latest release version, including the version number and platform-specific updates.
///
/// # Fields
///
/// * `version` - The version number of the patch in semantic format.
/// * `platforms` - A mapping of platform names to update versions.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PatchInfo {
    pub version: String,
    pub platforms: HashMap<String, String>,
}

impl PatchInfo {
    /// Returns the version number of the patch.
    ///
    /// # Returns
    ///
    /// A string slice representing the version number in semantic format.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns the download link for the specified platform.
    ///
    /// # Arguments
    ///
    /// * `platform` - A string slice representing the name of the platform to retrieve the download link for.
    ///
    /// # Returns
    ///
    /// An optional string slice representing the download link for the specified platform. Returns `None` if the specified platform is not found.
    pub fn download_link(&self, platform: &str) -> Option<&String> {
        self.platforms.get(platform)
    }
}