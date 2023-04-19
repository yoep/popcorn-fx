#![windows_subsystem = "windows"]

use std::{env, thread};
use std::env::VarError;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use log::error;
use signal_hook::consts::TERM_SIGNALS;
use signal_hook::flag;
use tokio::runtime::Runtime;

use crate::bootstrapper::{BootstrapError, Bootstrapper};

mod data_installer;
mod bootstrapper;

const ENV_INSTALLATION_DIR: &str = "INSTALLATION_DIR";
const DATA_DIR: &str = "DATA_DIR";

/// The main entry point of the bootstrap application.
///
/// This function creates a `Bootstrapper` instance and launches the application. The `Bootstrapper`
/// provides the ability for the application to self-update. It also initializes the logging system
/// and sets up environment variables for the application.
///
/// # Environment Variables
///
/// The following environment variables are used by the application:
///
/// * `PATH`: The system `$PATH` variable. This is used to find the libraries when launching the application.
///
/// * `INSTALLATION_DIR`: The directory where the application is installed. This can be used to locate
///   the application's configuration files and other resources.
///
/// * `DATA_DIR`: The directory where the application's data files are stored. This is used to store
///   the application's data files from which the application is actually launched.
///
/// # Errors
///
/// This function returns a `BootstrapError` if there was an issue with bootstrapping the application.
fn main() -> Result<(), BootstrapError> {
    let runtime = Runtime::new().unwrap();
    let term_now = Arc::new(AtomicBool::new(false));
    for signal in TERM_SIGNALS {
        flag::register_conditional_shutdown(*signal, 1, term_now.clone()).unwrap();
        flag::register(*signal, term_now.clone()).unwrap();
    }

    // Initialize the `Bootstrapper` instance with environment variables and launch the application.
    let bootstrapper = Arc::new(Bootstrapper::builder()
        .path(env::var("PATH")
            .or_else(|e| {
                eprintln!("PATH variable is invalid, {}", e);
                Ok::<String, VarError>("".to_string())
            })
            .unwrap())
        .args(env::args().collect())
        .data_base_path(env::var(DATA_DIR)
            .map(|e| Some(PathBuf::from(e)))
            .unwrap_or(None))
        .installation_path(env::var(ENV_INSTALLATION_DIR)
            .map(|e| Some(PathBuf::from(e)))
            .unwrap_or(None))
        .build());

    let bootstrapper_shutdown = bootstrapper.clone();
    let term_now_shutdown = term_now.clone();
    runtime.spawn(async move {
        while !term_now_shutdown.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(100));
        }

        bootstrapper_shutdown.shutdown();
    });

    let result = bootstrapper.launch()
        .map_err(|e| {
            error!("Bootstrap error: {}", e);
            e
        });
    term_now.store(true, Ordering::SeqCst);
    result
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_main() {
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        env::set_var("LOG_LEVEL", "trace");
        env::set_var(DATA_DIR, temp_path);
        env::set_var(ENV_INSTALLATION_DIR, PathBuf::from(temp_path).join("main"));

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