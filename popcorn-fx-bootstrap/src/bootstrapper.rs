use std::{env, thread};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use directories::BaseDirs;
use log::{debug, error, LevelFilter, trace, warn};
use log4rs::append::console::ConsoleAppender;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use thiserror::Error;

use crate::data_installer::{DataInstaller, DefaultDataInstaller};
use crate::launcher::LauncherOptions;

const CONSOLE_APPENDER: &str = "stdout";
const LOG_FORMAT_CONSOLE: &str = "\x1B[37m{d(%Y-%m-%d %H:%M:%S%.3f)}\x1B[0m {h({l:>5.5})} \x1B[35m{I:>6.6}\x1B[0m \x1B[37m---\x1B[0m \x1B[37m[{T:>15.15}]\x1B[0m \x1B[36m{t:<40.40}\x1B[0m \x1B[37m:\x1B[0m {m}{n}";
const DATA_DIRECTORY_NAME: &str = "popcorn-fx";
const RUNTIMES_DIRECTORY_NAME: &str = "runtimes";
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
    #[error("failed to create the initial data setup, {0}")]
    InitialSetupFailed(String),
    #[error("child process failed to execute, {1}\nCommand: {0:?}")]
    ExecuteFailed(Command, String),
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
    pub data_path: PathBuf,
    pub process_path: Option<String>,
    pub data_installer: Box<dyn DataInstaller>,
    pub shutting_down: Arc<AtomicBool>,
}

impl Bootstrapper {
    /// Create a new instance builder.
    pub fn builder() -> BootstrapperBuilder {
        BootstrapperBuilder::default()
    }

    /// Launch the application.
    /// The application will be automatically restarted when needed.
    pub fn launch(&self) -> Result<()> {
        // prepare the user's data system with the initial installation of the application if needed
        self.data_installer.prepare()
            .map_err(|e| BootstrapError::InitialSetupFailed(e.to_string()))?;

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

    /// Shutdown the current running application within the bootstrapper.
    pub fn shutdown(&self) {
        debug!("Received bootstrapper shutdown request");
        self.shutting_down.store(true, Ordering::SeqCst);
    }

    fn launch_instance(&self) -> Result<Action> {
        let mut command = self.command();
        trace!("Spawning process {:?}", command);
        let mut child = command
            .spawn()
            .map_err(|e| BootstrapError::ExecuteFailed(command, e.to_string()))?;

        while !self.shutting_down.load(Ordering::Relaxed) {
            match child.try_wait() {
                Ok(None) => thread::sleep(Duration::from_millis(100)),
                Ok(Some(exit_status)) => return Ok(Self::handle_exit_status(exit_status)),
                Err(e) => {
                    error!("Failed to wait for the application process, {}", e);
                    return Err(BootstrapError::InvalidHandle(e.to_string()));
                }
            }
        }

        // shutdown the current running process
        match child.kill() {
            Ok(_) => trace!("Application process has been terminated"),
            Err(_) => trace!("Application has already been terminated"),
        }

        Ok(Action::Shutdown)
    }

    /// Build the application command that will be bootstrapped.
    fn command(&self) -> Command {
        let options = Self::get_launcher_options(&self.data_path);
        let data_version_path = self.data_path
            .join(options.version.as_str());
        let data_version_path_value = data_version_path
            .to_str()
            .unwrap();
        let process_path = self.process_path.as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| Self::build_process_path(&self.data_path, &options));
        let jar_path = self.data_path
            .join(options.version.as_str())
            .join(JAR_NAME);

        trace!("Creating process command for {:?} with {:?}", process_path, self.args);
        let mut command = Command::new(process_path);
        command
            .arg(format!("-Djna.library.path={}{}{}", data_version_path_value, PATH_SEPARATOR, self.path.as_str()).as_str())
            .arg(format!("-Djava.library.path={}{}{}", data_version_path_value, PATH_SEPARATOR, self.path.as_str()).as_str());

        for vm_arg in options.vm_args.iter() {
            command.arg(vm_arg.as_str());
        }

        command.arg("-jar")
            .arg(jar_path.to_str().unwrap())
            .args(self.args.clone());

        command
    }

    fn build_process_path(data_path: &Path, options: &LauncherOptions) -> PathBuf {
        trace!("Creating process path with runtime {}", options.runtime_version);
        data_path
            .join(RUNTIMES_DIRECTORY_NAME)
            .join(options.runtime_version.as_str())
            .join("jre")
            .join("bin")
            .join(EXECUTABLE_NAME)
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
            .unwrap_or(Action::Restart)
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

    fn get_launcher_options<P: AsRef<Path>>(path: P) -> LauncherOptions {
        LauncherOptions::new(path)
    }
}

/// The `BootstrapperBuilder` struct is used to configure and create a new `Bootstrapper` instance.
///
/// # Examples
///
/// ```no_run
/// let bootstrapper = BootstrapperBuilder::default()
///     .path("/usr/bin/my_program".to_string())
///     .args(vec!["arg1".to_string(), "arg2".to_string()])
///     .data_base_path("/var/lib/my_program".into())
///     .process_path("echo")
///     .disable_logger(true)
///     .build();
/// ```
#[derive(Default)]
pub struct BootstrapperBuilder {
    path: Option<String>,
    args: Option<Vec<String>>,
    data_base_path: Option<PathBuf>,
    installation_path: Option<PathBuf>,
    disable_logger: bool,
    process_path: Option<String>,
}

impl BootstrapperBuilder {
    /// Sets the `$PATH` variable value for the `Bootstrapper`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let bootstrapper = BootstrapperBuilder::default()
    ///     .path("/usr/bin/my_program".to_string())
    ///     .build();
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    ///
    /// let bootstrapper = BootstrapperBuilder::default()
    ///     .data_base_path(PathBuf::from("/var/lib/my_program"))
    ///     .build();
    /// ```
    pub fn data_base_path(mut self, path: Option<PathBuf>) -> Self {
        self.data_base_path = path;
        self
    }

    /// Sets the installation path of the `Bootstrapper` application.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    ///
    /// let bootstrapper = BootstrapperBuilder::default()
    ///     .installation_path(Some(PathBuf::from("/usr/local/bin")))
    ///     .build();
    /// ```
    pub fn installation_path(mut self, path: Option<PathBuf>) -> Self {
        self.installation_path = path;
        self
    }

    /// Disables the log4rs logger used by `Bootstrapper`.
    /// This allows you to use your own logger if needed, or modify the default logger settings.
    ///
    /// # Examples
    ///
    /// ```
    /// let bootstrapper = BootstrapperBuilder::default()
    ///     .disable_logger(true)
    ///     .build();
    /// ```
    pub fn disable_logger(mut self, disable_logger: bool) -> Self {
        self.disable_logger = disable_logger;
        self
    }

    /// Sets the static path to the process executable for the `Bootstrapper`.
    pub fn process_path(mut self, process_path: String) -> Self {
        self.process_path = Some(process_path);
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
        let data_base_path = self.data_base_path.unwrap_or_else(|| BaseDirs::new()
            .map(|e| PathBuf::from(e.data_dir()))
            .expect("expected a system data directory"));
        let data_path = data_base_path.join(DATA_DIRECTORY_NAME);

        Bootstrapper {
            path: self.path.expect("Path is not set"),
            args: args.collect(),
            data_installer: Box::new(DefaultDataInstaller {
                data_path: data_path.clone(),
                installation_path: self.installation_path.unwrap_or_else(|| env::current_dir().expect("expected a working directory")),
            }),
            data_path,
            data_base_path,
            process_path: self.process_path,
            shutting_down: Arc::new(Default::default()),
        }
    }
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use popcorn_fx_core::testing::init_logger;

    use crate::data_installer::{DataInstallerError, MockDataInstaller};

    use super::*;

    #[tokio::test]
    async fn test_initialize_logger() {
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();

        Bootstrapper::builder()
            .args(vec!["popcorn-fx".to_string()])
            .path("".to_string())
            .data_base_path(Some(PathBuf::from(temp_path)))
            .build();
    }

    #[test]
    fn test_builder_disable_logger() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();

        Bootstrapper::builder()
            .args(vec!["popcorn-fx".to_string()])
            .path("".to_string())
            .data_base_path(Some(PathBuf::from(temp_path)))
            .disable_logger(true)
            .build();
    }

    #[test]
    fn test_launch() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut data_installer = MockDataInstaller::new();
        data_installer.expect_prepare()
            .returning(|| Ok(()));
        let bootstrap = Bootstrapper {
            path: "".to_string(),
            args: vec!["popcorn-fx".to_string()],
            data_base_path: PathBuf::from(temp_path).join(DATA_DIRECTORY_NAME),
            data_path: PathBuf::from(temp_path),
            process_path: Some("echo".to_string()),
            data_installer: Box::new(data_installer),
            shutting_down: Arc::new(Default::default()),
        };

        let result = bootstrap.launch();

        assert!(result.is_ok(), "expected the process to be completed with success")
    }

    #[test]
    fn test_launch_prepare_failure() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut data_installer = MockDataInstaller::new();
        data_installer.expect_prepare()
            .returning(|| Err(DataInstallerError::MissingAppData(PathBuf::from("."))));
        let bootstrap = Bootstrapper {
            path: "".to_string(),
            args: vec![],
            data_base_path: PathBuf::from(temp_path).join(DATA_DIRECTORY_NAME),
            data_path: PathBuf::from(temp_path),
            process_path: Some("echo".to_string()),
            data_installer: Box::new(data_installer),
            shutting_down: Arc::new(Default::default()),
        };

        let result = bootstrap.launch();

        if let Err(e) = result {
            match e {
                BootstrapError::InitialSetupFailed(_) => {}
                _ => assert!(false, "expected BootstrapError::InitialSetupFailed")
            }
        } else {
            assert!(false, "expected an error to be returned");
        }
    }

    #[test]
    fn test_launch_failure() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut data_installer = MockDataInstaller::new();
        data_installer.expect_prepare()
            .returning(|| Ok(()));
        let bootstrap = Bootstrapper {
            path: "".to_string(),
            args: vec![],
            data_base_path: PathBuf::from(temp_path).join(DATA_DIRECTORY_NAME),
            data_path: PathBuf::from(temp_path),
            process_path: Some("lorem".to_string()),
            data_installer: Box::new(data_installer),
            shutting_down: Arc::new(Default::default()),
        };

        let result = bootstrap.launch();

        if let Err(error) = result {
            match error {
                BootstrapError::ExecuteFailed(_command, _message) => {}
                _ => assert!(false, "expected BootstrapError::ExecuteFailed")
            }
        } else {
            assert!(false, "expected an error to have been returned")
        }
    }

    #[test]
    fn test_build_process_path() {
        init_logger();
        let temp_dir = tempdir().expect("expected a temp dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let data_path = PathBuf::from(temp_path);
        let expected_result = data_path
            .join("runtimes")
            .join("10.0.3")
            .join("jre")
            .join("bin")
            .join(EXECUTABLE_NAME);

        let result = Bootstrapper::build_process_path(data_path.as_path(), &LauncherOptions {
            version: "1.0.0".to_string(),
            runtime_version: "10.0.3".to_string(),
            vm_args: vec![],
        });

        assert_eq!(expected_result.to_str().unwrap(), result.to_str().unwrap())
    }
}