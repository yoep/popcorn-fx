use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::str::FromStr;

use directories::BaseDirs;
use log::{debug, error, LevelFilter, trace, warn};
use log4rs::append::console::ConsoleAppender;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use thiserror::Error;

use crate::launcher::LauncherOptions;

const CONSOLE_APPENDER: &str = "stdout";
const LOG_FORMAT_CONSOLE: &str = "\x1B[37m{d(%Y-%m-%d %H:%M:%S%.3f)}\x1B[0m {h({l:>5.5})} \x1B[35m{I:>6.6}\x1B[0m \x1B[37m---\x1B[0m \x1B[37m[{T:>15.15}]\x1B[0m \x1B[36m{t:<40.40}\x1B[0m \x1B[37m:\x1B[0m {m}{n}";
const DATA_DIRECTORY_NAME: &str = "popcorn-fx";
#[cfg(target_family = "windows")]
const EXECUTABLE_NAME: &str = "javaw.exe";
#[cfg(target_family = "windows")]
const PATH_SEPARATOR: &str = ";";
#[cfg(target_family = "unix")]
const EXECUTABLE_NAME: &str = "java";
#[cfg(target_family = "unix")]
const PATH_SEPARATOR: &str = ":";
const JAR_NAME: &str = "popcorn-time.jar";

/// The bootstrap specific results.
pub type Result<T> = std::result::Result<T, BootstrapError>;

/// The bootstrap errors.
#[derive(Debug, Error)]
pub enum BootstrapError {
    #[error("the child process failed to start, {0}")]
    ExecuteFailed(String),
    #[error("invalid process handle, {0}")]
    InvalidHandle(String),
}

/// The action to take after an instance process has completed.
#[derive(Debug, Clone, PartialEq)]
enum Action {
    Shutdown,
    Restart,
}

/// The `Bootstrapper` is responsible for launching the correct application version, and restarting the application when needed.
///
/// It holds the `$PATH` variable value, program arguments, and the path to the system data directory (which doesn't include the application directory prefix [DATA_DIRECTORY_NAME]).
///
/// # Examples
///
/// ```no_run
/// use popcorn_fx::bootstrapper::Bootstrapper;
///
/// let bootstrapper = Bootstrapper::builder()
///     .path(env::var("PATH").unwrap())
///     .args(env::args().collect())
///     .build();
///
/// bootstrapper.launch();
/// ```
#[derive(Debug)]
pub struct Bootstrapper {
    pub path: String,
    pub args: Vec<String>,
    pub data_base_path: PathBuf,
    pub options: LauncherOptions,
}

impl Bootstrapper {
    /// Create a new instance builder.
    pub fn builder() -> BootstrapperBuilder {
        BootstrapperBuilder::default()
    }

    /// Launch the application.
    /// The application will be automatically restarted when needed.
    pub fn launch(&self) -> Result<()> {
        loop {
            match self.launch_instance() {
                Ok(action) => {
                    if action == Action::Shutdown {
                        debug!("Shutting down application");
                        return Ok(());
                    } else {
                        debug!("Restarting application");
                    }
                }
                Err(e) => {
                    error!("Unable to start application, {}", e);
                    return Err(e);
                }
            }
        }
    }

    fn launch_instance(&self) -> Result<Action> {
        let mut command = self.command();
        trace!("Spawning process {:?}", command);
        let mut child = command
            .spawn()
            .map_err(|e| BootstrapError::ExecuteFailed(e.to_string()))?;

        let exit_status = child.wait()
            .map_err(|e| BootstrapError::InvalidHandle(e.to_string()))?;

        Ok(Self::handle_exit_status(exit_status))
    }

    /// Build the application command that will be bootstrapped.
    fn command(&self) -> Command {
        let data_path = self.data_base_path
            .join(DATA_DIRECTORY_NAME);
        let data_path_value = data_path.to_str().unwrap();
        let process_path = data_path
            .join("jre")
            .join("bin")
            .join(EXECUTABLE_NAME);
        let jar_path = data_path
            .join(JAR_NAME);

        trace!("Creating process command for {:?} with {:?}", process_path, self.args);
        let mut command = Command::new(process_path);
        command
            .arg(format!("-Djna.library.path={}{}{}", data_path_value, PATH_SEPARATOR, self.path.as_str()).as_str())
            .arg(format!("-Djava.library.path={}{}{}", data_path_value, PATH_SEPARATOR, self.path.as_str()).as_str())
            .arg("-Dsun.awt.disablegrab=true")
            .arg("-Dprism.dirtyopts=false")
            .arg("-Xms100M")
            .arg("-XX:+UseG1GC")
            .arg("-jar")
            .arg(jar_path.to_str().unwrap())
            .args(self.args.clone());

        command
    }

    fn handle_exit_status(exit_status: ExitStatus) -> Action {
        exit_status.code()
            .map(|e| if e == 0 {
                trace!("Application process exited with {}", exit_status);
                Action::Shutdown
            } else {
                warn!("Application process exited with {}", exit_status);
                Action::Restart
            })
            .unwrap_or(Action::Shutdown)
    }

    fn initialize_logger() {
        let root_level = env::var("LOG_LEVEL").unwrap_or("Info".to_string());
        let config = Config::builder()
            .appender(Appender::builder().build(CONSOLE_APPENDER, Box::new(ConsoleAppender::builder()
                .encoder(Box::new(PatternEncoder::new(LOG_FORMAT_CONSOLE)))
                .build())))
            .build(Root::builder()
                .appender(CONSOLE_APPENDER)
                .build(LevelFilter::from_str(root_level.as_str()).unwrap()))
            .unwrap();

        match log4rs::init_config(config) {
            Ok(_) => trace!("Popcorn FX bootstrap logger has been initialized"),
            Err(e) => eprintln!("Failed to configure logger, {}", e),
        }
    }
}

/// The `Bootstrapper` builder.
#[derive(Default)]
pub struct BootstrapperBuilder {
    path: Option<String>,
    args: Option<Vec<String>>,
    data_base_path: Option<PathBuf>,
    options: Option<LauncherOptions>,
    disable_logger: bool,
}

impl BootstrapperBuilder {
    /// Sets the `$PATH` variable value for the `Bootstrapper`.
    pub fn path(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }

    /// Sets the program arguments to pass to the `Bootstrapper`.
    pub fn args(mut self, args: Vec<String>) -> Self {
        self.args = Some(args);
        self
    }

    /// Sets the data base path for the `Bootstrapper`.
    pub fn data_base_path(mut self, path: PathBuf) -> Self {
        self.data_base_path = Some(path);
        self
    }

    /// Sets the launcher options for the `Bootstrapper`.
    pub fn options(mut self, options: LauncherOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// Disables the logger for the `Bootstrapper`.
    pub fn disable_logger(mut self, disable_logger: bool) -> Self {
        self.disable_logger = disable_logger;
        self
    }

    /// Builds a new `Bootstrapper` instance using the current builder state.
    ///
    /// # Panics
    ///
    /// This method will panic if either the `path` or `args` fields have not been set.
    pub fn build(self) -> Bootstrapper {
        if !self.disable_logger {
            Bootstrapper::initialize_logger();
        }
        let mut args = self.args.expect("Args are not set").into_iter();
        let _program_name = args.next().unwrap();
        let data_path = self.data_base_path.unwrap_or_else(|| BaseDirs::new()
            .map(|e| PathBuf::from(e.data_dir()))
            .expect("expected a system data directory"));

        Bootstrapper {
            path: self.path.expect("Path is not set"),
            args: args.collect(),
            options: Self::get_launcher_options(data_path.as_path()),
            data_base_path: data_path,
        }
    }

    /// Discover the launcher options based on the given data path.
    fn get_launcher_options(path: &Path) -> LauncherOptions {
        LauncherOptions::new(path)
    }
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use popcorn_fx_core::testing::init_logger;

    use super::*;

    #[tokio::test]
    async fn test_initialize_logger() {
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();

        Bootstrapper::builder()
            .args(vec!["popcorn-fx".to_string()])
            .path("".to_string())
            .data_base_path(PathBuf::from(temp_path))
            .build();
    }

    #[test]
    fn test_launch() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let bootstrap = Bootstrapper::builder()
            .disable_logger(true)
            .args(vec!["popcorn-fx".to_string()])
            .path("".to_string())
            .data_base_path(PathBuf::from(temp_path))
            .build();

        bootstrap.launch();
    }
}