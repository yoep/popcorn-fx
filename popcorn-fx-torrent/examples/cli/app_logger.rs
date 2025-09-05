use chrono::Local;
use log::{Level, Log, Metadata, Record};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct AppLogger {
    command_sender: UnboundedSender<LogEntry>,
}

impl AppLogger {
    pub fn new(command_sender: UnboundedSender<LogEntry>) -> Self {
        Self { command_sender }
    }
}

impl Log for AppLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        matches!(metadata.level(), Level::Info | Level::Warn | Level::Error)
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let time = Local::now();
        let target = {
            let target = record.target();
            let target = format!("{}{}", target, " ".repeat(40));
            target[0..40].to_string()
        };

        if self.enabled(record.metadata()) {
            let _ = self.command_sender.send(LogEntry {
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

    fn flush(&self) {}
}

#[derive(Debug)]
pub struct LogEntry {
    pub text: String,
}
