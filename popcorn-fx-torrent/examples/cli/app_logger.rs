use chrono::Local;
use log::{Level, Log, Metadata, Record};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// A log entry of the application.
#[derive(Debug)]
pub struct LogEntry {
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct AppLogger {
    inner: Arc<InnerAppLogger>,
}

impl AppLogger {
    pub fn new() -> Self {
        let inner = Arc::new(InnerAppLogger {
            loggers: Mutex::new(
                vec![
                    Logger {
                        name: "DHT".to_string(),
                        target: "popcorn_fx_torrent::torrent::dht".to_string(),
                        level: Level::Info,
                    },
                    Logger {
                        name: "DNS".to_string(),
                        target: "popcorn_fx_torrent::torrent::dns".to_string(),
                        level: Level::Info,
                    },
                    Logger {
                        name: "Operations".to_string(),
                        target: "popcorn_fx_torrent::torrent::operation".to_string(),
                        level: Level::Info,
                    },
                    Logger {
                        name: "Peers".to_string(),
                        target: "popcorn_fx_torrent::torrent::peer".to_string(),
                        level: Level::Info,
                    },
                    Logger {
                        name: "Session".to_string(),
                        target: "popcorn_fx_torrent::torrent::session".to_string(),
                        level: Level::Info,
                    },
                    Logger {
                        name: "Trackers".to_string(),
                        target: "popcorn_fx_torrent::torrent::tracker".to_string(),
                        level: Level::Info,
                    },
                ]
                .into_iter()
                .collect(),
            ),
            logs: Mutex::new(VecDeque::new()),
        });

        Self { inner }
    }

    /// Try to get the next log entry from the logger.
    pub fn next(&self) -> Option<LogEntry> {
        self.inner.logs.lock().ok().and_then(|mut e| e.pop_front())
    }

    /// Get the current configured loggers with their log level
    pub fn loggers(&self) -> Vec<Logger> {
        self.inner
            .loggers
            .lock()
            .ok()
            .map(|e| e.clone())
            .unwrap_or_default()
    }

    /// Update the log level for the given target.
    pub fn update<S: AsRef<str>>(&self, target: S, level: &Level) {
        if let Ok(mut loggers) = self.inner.loggers.lock() {
            if let Some(logger) = loggers.iter_mut().find(|e| &e.target == target.as_ref()) {
                logger.level = *level;
            }
        }
    }
}

impl Log for AppLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.inner.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        self.inner.send_entry(record);
    }

    fn flush(&self) {}
}

#[derive(Debug)]
struct InnerAppLogger {
    loggers: Mutex<Vec<Logger>>,
    logs: Mutex<VecDeque<LogEntry>>,
}

impl InnerAppLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let target = metadata.target();
        let level = self
            .loggers
            .lock()
            .ok()
            .map(|e| {
                let mut level = Level::Info;
                let mut last_overlap_size = 0u32;

                for conf in e.iter() {
                    let overlap = Self::overlap_size(conf.target.as_str(), target);

                    if overlap > last_overlap_size {
                        level = conf.level.clone();
                        last_overlap_size = overlap;
                    }
                }

                level
            })
            .unwrap_or(Level::Info);

        metadata.level().to_level_filter() <= level.to_level_filter()
    }

    fn send_entry(&self, record: &Record) {
        let time = Local::now();
        let target = {
            let target = record.target();
            let target = format!("{}{}", target, " ".repeat(40));
            target[0..40].to_string()
        };

        if let Ok(mut logs) = self.logs.lock() {
            logs.push_back(LogEntry {
                text: format!(
                    "{} {} --- {} : {}",
                    time.format("%Y-%m-%d %H:%M:%S%.f"),
                    record.level(),
                    target,
                    record.args()
                ),
            });
        }
    }

    fn overlap_size(logger: &str, target: &str) -> u32 {
        let mut logger_chars = logger.chars();
        let mut target_chars = target.chars();

        let mut count = 0;
        loop {
            match (logger_chars.next(), target_chars.next()) {
                (Some(a), Some(b)) if a == b => {
                    count += 1;
                }
                _ => return count,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Logger {
    /// The display name of the logger
    pub name: String,
    /// The log target of the logger
    pub target: String,
    /// The configured level of the logger
    pub level: Level,
}
