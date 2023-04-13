#![windows_subsystem = "windows"]

use std::env;
use std::env::VarError;
use std::path::PathBuf;

use crate::bootstrapper::{BootstrapError, Bootstrapper};

mod data_installer;
mod bootstrapper;
mod launcher;

const ENV_INSTALLATION_DIR: &str = "INSTALLATION_DIR";

/// The main entry of the bootstrap application.
///
/// This function creates a `Bootstrapper` instance and launches the application. The `Bootstrapper`
/// provides the ability for the application to self-update. It also initializes the logging system
/// and sets up environment variables.
fn main() -> Result<(), BootstrapError> {
    let bootstrapper = Bootstrapper::builder()
        .path(env::var("PATH")
            .or_else(|e| {
                eprintln!("PATH variable is invalid, {}", e);
                Ok::<String, VarError>("".to_string())
            })
            .unwrap())
        .args(env::args().collect())
        .installation_path(env::var(ENV_INSTALLATION_DIR)
            .map(|e| Some(PathBuf::from(e)))
            .unwrap_or(None))
        .build();

    bootstrapper.launch()
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_main() {
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        env::set_var(ENV_INSTALLATION_DIR, temp_path);

        let result = main();

        if let Err(e) = result {
            match e {
                BootstrapError::InitialSetupFailed(_) => {}
                _ => assert!(false, "expected BootstrapError::InitialSetupFailed, got {:?} instead", e),
            }
        } else {
            assert!(false, "expected the main fn to failed")
        }
    }
}