mod app;
mod app_logger;
mod dht_info;
mod menu;
mod torrent_info;
mod tracker_info;
mod widget;

use crate::app::App;
use crate::app_logger::AppLogger;
use log::LevelFilter;
use std::io;
use tokio::select;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> io::Result<()> {
    let app_logger = AppLogger::new();
    let mut app = App::new(app_logger.clone()).await?;
    let terminal = ratatui::init();

    log::set_boxed_logger(Box::new(app_logger))
        .map(|()| log::set_max_level(LevelFilter::Trace))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let result = select! {
        _ = tokio::signal::ctrl_c() => Ok(()),
        result = app.run(terminal) => result,
    };

    ratatui::restore();
    result
}
