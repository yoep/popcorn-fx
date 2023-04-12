use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::string::ToString;

use log::{debug, trace, warn};
use serde::{Deserialize, Serialize};

const FILENAME: &str = "launcher";
const EXTENSIONS: [&str; 2] = [
    "yml",
    "yaml"
];
const DEFAULT_VERSION: fn() -> String = || "0.6.5".to_string();
const DEFAULT_RUNTIME_VERSION: fn() -> String = || "17.0.6".to_string();
const DEFAULT_VM_ARGS: fn() -> Vec<String> = || vec![
    "-Dsun.awt.disablegrab=true".to_string(),
    "-Dprism.dirtyopts=false".to_string(),
    "-Xms100M".to_string(),
    "-XX:+UseG1GC".to_string(),
];

/// The options for launching an application.
///
/// `LauncherOptions` is a struct that contains options used to bootstrap an application. It includes the application version to launch,
/// the default Java Virtual Machine (JVM) runtime version to use, and the JVM arguments to apply to the application.
///
/// # Examples
///
/// ```
/// use my_crate::LauncherOptions;
///
/// let options = LauncherOptions {
///     version: "1.0.0".to_string(),
///     runtime_version: "11".to_string(),
///     vm_args: vec!["-Xms512m".to_string(), "-Xmx1024m".to_string()],
/// };
/// ```
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct LauncherOptions {
    /// The application version to launch.
    #[serde(default = "DEFAULT_VERSION")]
    pub version: String,
    /// The default JVM runtime version to use.
    #[serde(default = "DEFAULT_RUNTIME_VERSION")]
    pub runtime_version: String,
    /// The JVM arguments to apply to the application.
    #[serde(default = "DEFAULT_VM_ARGS")]
    pub vm_args: Vec<String>,
}

impl LauncherOptions {
    /// Automatically discover the launcher options for the given application data path.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        debug!("Searching for options with filename \"{}\"", FILENAME);
        let config_value = Self::find_existing_file(path.as_ref(), FILENAME)
            .map(|mut e| {
                let mut data = String::new();
                e.read_to_string(&mut data).expect("Unable to read the config file");
                data
            })
            .or_else(|| Some(String::new()))
            .expect("Properties should have been loaded");

        Self::from(config_value.as_str())
    }

    fn find_existing_file(path: &Path, filename: &str) -> Option<File> {
        let mut result: Option<File> = None;

        for extension in EXTENSIONS {
            let path = PathBuf::from(path)
                .join(filename)
                .with_extension(extension);
            match File::open(&path) {
                Ok(file) => {
                    debug!("Found config file {:?}", &path);
                    result = Some(file);
                    break;
                }
                Err(_) => trace!("Config file location {:?} doesn't exist", &path)
            }
        }

        result
    }
}

impl Default for LauncherOptions {
    fn default() -> Self {
        Self {
            version: DEFAULT_VERSION(),
            runtime_version: DEFAULT_RUNTIME_VERSION(),
            vm_args: DEFAULT_VM_ARGS(),
        }
    }
}

impl From<&str> for LauncherOptions {
    fn from(value: &str) -> Self {
        trace!("Parsing launcher options data {}", value);
        let options: LauncherOptions = match serde_yaml::from_str(value) {
            Ok(properties) => properties,
            Err(err) => {
                warn!("Failed to parse launcher options, using defaults instead, {}", err);
                serde_yaml::from_str(String::new().as_str()).unwrap()
            }
        };

        debug!("Parsed launcher options {:?}", &options);
        options
    }
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use popcorn_fx_core::testing::{copy_test_file, init_logger};

    use super::*;

    #[test]
    fn test_from() {
        init_logger();
        let expected_result = LauncherOptions {
            version: "0.1.0".to_string(),
            runtime_version: "17.0.0".to_string(),
            vm_args: vec!["test".to_string()],
        };

        let options = LauncherOptions::from(r#"
version: 0.1.0
runtime_version: 17.0.0
vm_args:
    - test
        "#);

        assert_eq!(expected_result, options)
    }

    #[test]
    fn test_new() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        copy_test_file(temp_path, "launcher.yml", None);
        let expected_result = LauncherOptions {
            version: "99.0.0".to_string(),
            runtime_version: "101.0.0".to_string(),
            vm_args: vec![
                "lorem".to_string(),
                "ipsum".to_string(),
            ],
        };

        let result = LauncherOptions::new(Path::new(temp_path));

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_new_invalid_options() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        copy_test_file(temp_path, "invalid_launcher.yml", Some("launcher.yaml"));
        let expected_result = LauncherOptions {
            version: DEFAULT_VERSION(),
            runtime_version: DEFAULT_RUNTIME_VERSION(),
            vm_args: DEFAULT_VM_ARGS(),
        };

        let result = LauncherOptions::new(Path::new(temp_path));

        assert_eq!(expected_result, result)
    }
}