use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};

use log::{debug, trace};
use mockall::automock;
use thiserror::Error;

use popcorn_fx_common::{LauncherError, LauncherOptions};

const INITIAL_INSTALL_DIRECTORY: &str = "main";

/// The result type specific to the data installer.
pub type Result<T> = std::result::Result<T, DataInstallerError>;

/// Errors that might occur during installation.
#[derive(Debug, Error, PartialEq)]
pub enum DataInstallerError {
    #[error("missing application data in installation at {0:?}")]
    MissingAppData(PathBuf),
    #[error("failed to parse launcher options, {0}")]
    ParsingError(String),
    #[error("an IO error occurred, {0}")]
    IoError(String),
}

impl From<LauncherError> for DataInstallerError{
    fn from(value: LauncherError) -> Self {
        match value {
            LauncherError::ParsingError(e) => DataInstallerError::ParsingError(e),
            LauncherError::IoError(e) => DataInstallerError::IoError(e)
        }
    }
}

/// A trait for installing and preparing application data.
#[automock]
pub trait DataInstaller: Debug + Send + Sync {
    /// Prepares the user's data system with the initial version of the application if needed.
    ///
    /// # Errors
    ///
    /// Returns an error when the data directory couldn't be initialized.
    fn prepare(&self) -> Result<()>;
}

/// The data installer is responsible for making sure that the initial application version libraries and data are available
/// within the user's data system when launching the application.
#[derive(Debug)]
pub struct DefaultDataInstaller {
    pub data_path: PathBuf,
    pub installation_path: PathBuf,
}

impl DefaultDataInstaller {
    /// Verify if the user's data directory has already been initialized.
    fn is_initialized<T: AsRef<Path>>(&self, launcher_options_path: T) -> bool {
        let expected_path = PathBuf::from(launcher_options_path.as_ref());

        trace!("Checking if application data is initialized at {:?}", expected_path);
        expected_path.exists()
    }

    fn copy_directory_contents<T: AsRef<Path>>(src: T, dest: T) -> Result<()> {
        let source = src.as_ref();
        let destination = dest.as_ref();
        if !destination.exists() {
            trace!("Creating application data directory {:?}", destination);
            fs::create_dir(destination)
                .map_err(|e| DataInstallerError::IoError(e.to_string()))?;
        }

        trace!("Copying files from {:?} to {:?}", source, destination);
        for entry in fs::read_dir(source).map_err(|e| DataInstallerError::IoError(e.to_string()))? {
            let entry = entry.map_err(|e| DataInstallerError::IoError(e.to_string()))?;
            let path = entry.path();
            let file_name = entry.file_name();
            let dest_path = destination.join(file_name);

            if path.is_dir() {
                Self::copy_directory_contents(&path, &dest_path)?;
            } else {
                trace!("Copying file {:?} to {:?}", path, dest_path);
                fs::copy(&path, &dest_path)
                    .map_err(|e| DataInstallerError::IoError(e.to_string()))?;
            }
        }

        Ok(())
    }

    fn write_default_launcher_options<T: AsRef<Path>>(launcher_options_path: T) -> Result<()> {
        let options = LauncherOptions::default();
        options.write(launcher_options_path)
            .map_err(|e| DataInstallerError::from(e))
    }
}

impl DataInstaller for DefaultDataInstaller {
    fn prepare(&self) -> Result<()> {
        let launcher_options_path = PathBuf::from(self.data_path.as_path())
            .join(LauncherOptions::filename());

        if !self.is_initialized(launcher_options_path.as_path()) {
            trace!("Initializing application data setup");
            let initial_data_setup_path = self.installation_path.join(INITIAL_INSTALL_DIRECTORY);
            if !initial_data_setup_path.exists() {
                return Err(DataInstallerError::MissingAppData(initial_data_setup_path));
            }

            trace!("Copying application data to user data directory");
            Self::copy_directory_contents(initial_data_setup_path.as_path(), self.data_path.as_path())?;
            Self::write_default_launcher_options(launcher_options_path.as_path())?;
            debug!("Initial application data setup completed");
        } else {
            debug!("Application data setup already initialized, skipping data initialization")
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use popcorn_fx_core::testing::{copy_test_file, init_logger};

    use super::*;

    #[test]
    fn test_prepare_already_initialized() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = PathBuf::from(temp_dir.path());
        copy_test_file(temp_path.to_str().unwrap(), "launcher.yml", None);
        let installer = DefaultDataInstaller {
            data_path: PathBuf::from(temp_dir.path()),
            installation_path: PathBuf::from(temp_dir.path()).join(INITIAL_INSTALL_DIRECTORY),
        };

        let result = installer.prepare();

        assert!(result.is_ok(), "expected the prepare to succeed")
    }

    #[test]
    fn test_prepare_missing_installation_data() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let installer = DefaultDataInstaller {
            data_path: PathBuf::from(temp_dir.path()),
            installation_path: PathBuf::from(temp_dir.path()).join(INITIAL_INSTALL_DIRECTORY),
        };

        let result = installer.prepare();

        if let Err(e) = result {
            match e {
                DataInstallerError::MissingAppData(_path) => {}
                _ => assert!(false, "expected DataInstallerError::MissingAppData, got {:?} instead", e)
            }
        } else {
            assert!(false, "expected an error to be returned")
        }
    }

    #[test]
    fn test_prepare() {
        init_logger();
        let filename = "launcher.yml";
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let installation_path = PathBuf::from(temp_dir.path()).join(INITIAL_INSTALL_DIRECTORY);
        let installation_path_value = installation_path.to_str().unwrap();
        let data_path = PathBuf::from(temp_dir.path()).join("data");
        let file1 = data_path.join(filename);
        let file2 = data_path.join("test1").join(filename);
        copy_test_file(installation_path_value, filename, None);
        copy_test_file(installation_path.join("test1").to_str().unwrap(), filename, None);
        let installer = DefaultDataInstaller {
            data_path: data_path.clone(),
            installation_path: PathBuf::from(temp_dir.path()),
        };

        let result = installer.prepare();

        assert!(result.is_ok(), "expected the preparation to succeed");
        assert!(file1.exists(), "expected {:?} to exist", file1);
        assert!(file2.exists(), "expected {:?} to exist", file2);
    }

    #[test]
    fn test_from_launcher_options() {
        let parser_error = "my parser error";
        let io_error = "my IO error";

        assert_eq!(DataInstallerError::ParsingError(parser_error.to_string()), DataInstallerError::from(LauncherError::ParsingError(parser_error.to_string())));
        assert_eq!(DataInstallerError::IoError(io_error.to_string()), DataInstallerError::from(LauncherError::IoError(io_error.to_string())));
    }
}