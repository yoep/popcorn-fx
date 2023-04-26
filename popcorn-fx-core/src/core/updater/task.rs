use std::path::{Path, PathBuf};

use log::info;
use semver::Version;
use url::Url;

use crate::core::updater;
use crate::core::updater::UpdateError;

/// An update task which allows updating a component from the application.
///
/// Use this struct to represent an update task for a component of your application. An update task consists of the current version of the component, the new version to be installed, and a download link from which the new version can be obtained.
///
/// # Fields
///
/// * `current_version` - The current version of the component to be updated.
/// * `new_version` - The new version of the component to be installed.
/// * `download_link` - The URL where the new version of the component can be downloaded.
/// * `archive_location` - An optional file path where the downloaded update archive should be stored. This is only used if the update is being downloaded as a file rather than in memory.
///
/// # Example
///
/// ```no_run
/// use semver::Version;
/// use url::Url;
///
/// let task = UpdateTask::builder()
///     .current_version(Version::parse("1.0.0").unwrap())
///     .new_version(Version::parse("1.1.0").unwrap())
///     .download_link(Url::parse("https://example.com/update").unwrap())
///     .build();
///
/// assert_eq!(task.current_version, Version::parse("1.0.0").unwrap());
/// assert_eq!(task.new_version, Version::parse("1.1.0").unwrap());
/// assert_eq!(task.download_link, Url::parse("https://example.com/update").unwrap());
/// assert_eq!(task.archive_location(), None);
///
/// task.set_archive_location("/path/to/archive.zip").unwrap();
/// assert_eq!(task.archive_location(), Some(&"/path/to/archive.zip".into()));
/// ```
///
/// This example shows how to create an `UpdateTask` instance using the `builder` method.
/// The `builder` method is used to set the current version, new version, and download link of the task.
/// After constructing the task, the example demonstrates how to retrieve and set the optional archive location using the `archive_location` and `set_archive_location` methods, respectively.
#[derive(Debug, Clone)]
pub struct UpdateTask {
    pub current_version: Version,
    pub new_version: Version,
    pub download_link: Url,
    install_directory: String,
    archive_location: Option<PathBuf>,
}

impl UpdateTask {
    /// Returns a new `UpdateTaskBuilder` instance.
    pub fn builder() -> UpdateTaskBuilder {
        UpdateTaskBuilder::default()
    }

    /// Returns the installation subdirectory in which the archive should be extracted.
    pub fn install_directory(&self) -> &str {
        self.install_directory.as_str()
    }

    /// Returns the current archive location, if one has been set.
    pub fn archive_location(&self) -> Option<&PathBuf> {
        self.archive_location.as_ref()
    }

    /// Sets the archive location for the downloaded update archive.
    ///
    /// If an archive location has already been set, this method will return an error.
    pub fn set_archive_location<P: AsRef<Path>>(&mut self, archive_location: P) -> updater::Result<()> {
        if self.archive_location.is_some() {
            return Err(UpdateError::ArchiveLocationAlreadyExists);
        }

        self.archive_location = Some(PathBuf::from(archive_location.as_ref()));
        info!("Download task {} has been stored in {:?}", self.download_link, archive_location.as_ref());
        Ok(())
    }
}

#[derive(Default)]
pub struct UpdateTaskBuilder {
    current_version: Option<Version>,
    new_version: Option<Version>,
    download_link: Option<Url>,
    install_directory: Option<String>,
}

impl UpdateTaskBuilder {
    /// Sets the current version.
    pub fn current_version(mut self, version: Version) -> Self {
        self.current_version = Some(version);
        self
    }

    /// Sets the new version.
    pub fn new_version(mut self, version: Version) -> Self {
        self.new_version = Some(version);
        self
    }

    /// Sets the download link.
    pub fn download_link(mut self, download_link: Url) -> Self {
        self.download_link = Some(download_link);
        self
    }

    /// Sets the directory within the installation location in which the task will be extracted.
    pub fn install_directory(mut self, install_directory: String) -> Self {
        self.install_directory = Some(install_directory);
        self
    }

    /// Builds an `UpdateTask` object with the specified parameters.
    ///
    /// # Panics
    ///
    /// This function will panic if the `current_version`, `new_version`, or `download_link` fields have not been set.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use semver::Version;
    ///
    /// let task = UpdateTask::builder()
    ///     .current_version(Version::parse("1.0.0").unwrap())
    ///     .new_version(Version::parse("1.1.0").unwrap())
    ///     .download_link("https://example.com/update".parse().unwrap())
    ///     .build();
    /// ```
    pub fn build(self) -> UpdateTask {
        let current_version = self.current_version.expect("Current version has not been set");
        let new_version = self.new_version.expect("New version has not been set");
        let download_link = self.download_link.expect("Download link has not been set");
        let install_directory = self.install_directory.expect("Install directory has not been set");

        UpdateTask {
            current_version,
            new_version,
            download_link,
            install_directory,
            archive_location: None,
        }
    }
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use crate::core::updater::UpdaterBuilder;
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_archive_location_none() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let update = UpdateTask::builder()
            .current_version(Version::parse("1.0.0").unwrap())
            .new_version(Version::parse("1.1.0").unwrap())
            .download_link(Url::parse("http://localhost/update").unwrap())
            .install_directory("install".to_string())
            .build();

        assert_eq!(None, update.archive_location())
    }

    #[test]
    fn test_set_archive_location_is_none() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut update = UpdateTask::builder()
            .current_version(Version::parse("1.0.0").unwrap())
            .new_version(Version::parse("1.1.0").unwrap())
            .download_link(Url::parse("http://localhost/update").unwrap())
            .install_directory("install".to_string())
            .build();

        let result = update.set_archive_location(temp_path);

        assert!(result.is_ok(), "expected the archive location to succeed");
        assert_eq!(Some(&PathBuf::from(temp_path)), update.archive_location())
    }

    #[test]
    fn test_set_archive_location_is_some() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut update = UpdateTask::builder()
            .current_version(Version::parse("1.0.0").unwrap())
            .new_version(Version::parse("1.2.0").unwrap())
            .download_link(Url::parse("http://localhost/update").unwrap())
            .install_directory("install".to_string())
            .build();

        update.archive_location = Some(PathBuf::from(temp_path));
        let result = update.set_archive_location(temp_path);

        if let Err(e) = result {
            assert_eq!(UpdateError::ArchiveLocationAlreadyExists, e);
        } else {
            assert!(false, "expected the archive location to error");
        }
    }
}