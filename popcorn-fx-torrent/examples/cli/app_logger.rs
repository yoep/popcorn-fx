use crate::app::AppCommand;
use chrono::Local;
use log::{Level, Log, Metadata, Record};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct AppLogger {
    command_sender: UnboundedSender<AppCommand>,
}

impl AppLogger {
    pub fn new(command_sender: UnboundedSender<AppCommand>) -> Self {
        Self { command_sender }
    }
}

impl Log for AppLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() >= Level::Info
    }

    fn log(&self, record: &Record) {
        let time = Local::now();
        let target = {
            let target = record.target();
            let target = format!("{}{}", target, " ".repeat(40));
            target[0..40].to_string()
        };

        if self.enabled(record.metadata()) {
            let _ = self.command_sender.send(AppCommand::Log(format!(
                "{} {} --- {} : {}",
                time.format("%Y-%m-%d %H:%M:%s%.f"),
                record.level(),
                target,
                record.args()
            )));
        }
    }

    fn flush(&self) {}
}
