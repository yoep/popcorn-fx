mod app;
mod app_logger;
mod menu;
mod torrent_info;

use crate::app::App;
use crate::app_logger::AppLogger;
use log::LevelFilter;
use std::io;
use tokio::select;
use tokio::sync::mpsc::unbounded_channel;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> io::Result<()> {
    let (log_sender, log_receiver) = unbounded_channel();
    let app_logger = AppLogger::new(log_sender.clone());
    let mut app = App::new(log_receiver)?;
    let terminal = ratatui::init();

    log::set_boxed_logger(Box::new(app_logger))
        .map(|()| log::set_max_level(LevelFilter::Info))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let result = select! {
        _ = tokio::signal::ctrl_c() => Ok(()),
        result = app.run(terminal) => result,
    };

    ratatui::restore();
    result
}
