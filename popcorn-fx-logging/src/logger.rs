use crate::{Error, Result};
use log::{info, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::{Appender, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::{Config, Handle};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

const LOG_FORMAT_CONSOLE: &str = "\x1B[37m{d(%Y-%m-%d %H:%M:%S%.3f)}\x1B[0m {h({l:>5.5})} \x1B[35m{I:>6.6}\x1B[0m \x1B[37m---\x1B[0m \x1B[37m[{T:>15.15}]\x1B[0m \x1B[36m{t:<40.40}\x1B[0m \x1B[37m:\x1B[0m {m}{n}";
const LOG_FORMAT_FILE: &str =
    "{d(%Y-%m-%d %H:%M:%S%.3f)} {h({l:>5.5})} {I:>6.6} --- [{T:>15.15}] {t:<40.40} : {m}{n}";
const CONSOLE_APPENDER: &str = "stdout";
const FILE_APPENDER: &str = "file";
const LOG_FILE_SIZE: u64 = 50 * 1024 * 1024;

static INITIALIZED: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub struct FxLogger {
    handle: Handle,
}

impl FxLogger {
    /// Returns a builder instance for the logger.
    pub fn builder() -> FxLoggerBuilder {
        FxLoggerBuilder::default()
    }

    /// Create a new logging instance.
    pub fn new(
        root_level: LevelFilter,
        config_path: Option<impl AsRef<Path>>,
        log_path: Option<impl AsRef<Path>>,
        loggers: Vec<(String, LevelFilter)>,
    ) -> Result<Self> {
        if INITIALIZED.load(Ordering::Relaxed) {
            return Err(Error::AlreadyInitialized);
        }

        INITIALIZED.store(true, Ordering::Relaxed);
        let config = match config_path {
            Some(path) => Self::load_from_config(path)?,
            None => Self::create_config(root_level, log_path, loggers)?,
        };

        let handle =
            log4rs::init_config(config).map_err(|e| Error::InvalidConfig(e.to_string()))?;
        info!("Popcorn FX logger has been initialized");
        Ok(Self { handle })
    }

    /// Returns the root logging level of the logger.
    pub fn root_log_level(&self) -> LevelFilter {
        self.handle.max_log_level()
    }

    fn load_from_config(path: impl AsRef<Path>) -> Result<Config> {
        log4rs::config::load_config_file(path, Default::default())
            .map_err(|e| Error::InvalidConfig(e.to_string()))
    }

    fn create_config(
        root_level: LevelFilter,
        log_path: Option<impl AsRef<Path>>,
        loggers: Vec<(String, LevelFilter)>,
    ) -> Result<Config> {
        let mut root = Root::builder().appender(CONSOLE_APPENDER);
        let mut config_builder = Config::builder().appender(
            Appender::builder().build(
                CONSOLE_APPENDER,
                Box::new(
                    ConsoleAppender::builder()
                        .encoder(Box::new(PatternEncoder::new(LOG_FORMAT_CONSOLE)))
                        .build(),
                ),
            ),
        );

        // append the file logger, if one is given
        if let Some(path) = log_path {
            config_builder = config_builder.appender(Self::create_file_appender(path)?);
            root = root.appender(FILE_APPENDER);
        }

        // configure the package log levels
        for (logger, level) in loggers.into_iter() {
            config_builder = config_builder.logger(Logger::builder().build(logger, level));
        }

        config_builder
            .build(root.build(root_level))
            .map_err(|e| Error::InvalidConfig(e.to_string()))
    }

    fn create_file_appender(path: impl AsRef<Path>) -> Result<Appender> {
        if let Err(e) = path
            .as_ref()
            .parent()
            .map(std::fs::create_dir_all)
            .transpose()
        {
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                return Err(Error::from(e));
            }
        }

        let policy = CompoundPolicy::new(
            Box::new(SizeTrigger::new(LOG_FILE_SIZE)),
            Box::new(
                FixedWindowRoller::builder()
                    .base(1)
                    .build("popcorn-time.{}.log", 5)
                    .expect("expected the window roller to be valid"),
            ),
        );

        Ok(Appender::builder().build(
            FILE_APPENDER,
            Box::new(
                RollingFileAppender::builder()
                    .encoder(Box::new(PatternEncoder::new(LOG_FORMAT_FILE)))
                    .append(false)
                    .build(path, Box::new(policy))
                    .map_err(|e| Error::InvalidConfig(e.to_string()))?,
            ),
        ))
    }
}

#[derive(Debug, Default)]
pub struct FxLoggerBuilder {
    root_level: Option<LevelFilter>,
    config_path: Option<PathBuf>,
    log_path: Option<PathBuf>,
    loggers: HashMap<String, LevelFilter>,
}

impl FxLoggerBuilder {
    /// Set the root level of the logger.
    pub fn root_level(&mut self, level: LevelFilter) -> &mut Self {
        self.root_level = Some(level);
        self
    }

    /// Set the path of the `log4.yml` config to load.
    pub fn config_path(&mut self, path: PathBuf) -> &mut Self {
        self.config_path = Some(path);
        self
    }

    /// Set the log file path of the logger.
    pub fn log_path(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.log_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Add a log level filter for the given package.
    pub fn logger<S: AsRef<str>>(&mut self, package: S, level: LevelFilter) -> &mut Self {
        self.loggers.insert(package.as_ref().to_string(), level);
        self
    }

    /// Consumes the [FxLoggerBuilder] and creates a new logging instance.
    pub fn build(&mut self) -> Result<FxLogger> {
        let root_level = self.root_level.take().unwrap_or(LevelFilter::Info);
        let config_path = self.config_path.take();
        let log_path = self.log_path.take();
        let loggers = self.loggers.drain().collect::<Vec<_>>();

        FxLogger::new(root_level, config_path, log_path, loggers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_new() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let logger = FxLogger::builder()
            .root_level(LevelFilter::Trace)
            .log_path(PathBuf::from(temp_path).join("popcorn-time.log"))
            .logger("popcorn_fx_core::core::event", LevelFilter::Debug)
            .build()
            .expect("expected a logger");

        // get the root log level
        let result = logger.root_log_level();
        assert_eq!(LevelFilter::Trace, result);

        // try to create a second instance
        let result = FxLogger::builder()
            .build()
            .err()
            .expect("expected an error to have been returned");
        assert_eq!(Error::AlreadyInitialized, result);
    }
}
