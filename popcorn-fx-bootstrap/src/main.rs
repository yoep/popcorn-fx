#![windows_subsystem = "windows"]

use std::{env, process};
use std::env::VarError;

use crate::bootstrapper::Bootstrapper;

mod bootstrapper;
mod launcher;

/// The main entry of the bootstrap application.
///
/// This function creates a `Bootstrapper` instance and launches the application. The `Bootstrapper`
/// provides the ability for the application to self-update. It also initializes the logging system
/// and sets up environment variables.
fn main() {
    let bootstrapper = Bootstrapper::builder()
        .path(env::var("PATH")
            .or_else(|e| {
                eprintln!("PATH variable is invalid, {}", e);
                Ok::<String, VarError>("".to_string())
            })
            .unwrap())
        .args(env::args().collect())
        .build();

    if bootstrapper.launch().is_err() {
        process::exit(1)
    }
}