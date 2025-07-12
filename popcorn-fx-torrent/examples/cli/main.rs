mod app;
mod app_logger;

use crate::app::App;
use crate::app_logger::AppLogger;
use log::set_boxed_logger;
use std::{env, io};
use tokio::select;
use tokio::sync::mpsc::unbounded_channel;

#[tokio::main]
async fn main() -> io::Result<()> {
    let torrent_uri = env::args().nth(1).ok_or(io::Error::new(
        io::ErrorKind::NotFound,
        "expected a torrent uri to have been provided",
    ))?;
    let (command_sender, command_receiver) = unbounded_channel();
    let app_logger = AppLogger::new(command_sender);
    let mut app = App::new()?;
    let terminal = ratatui::init();

    set_boxed_logger(Box::new(app_logger)).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let result = select! {
        _ = tokio::signal::ctrl_c() => Ok(()),
        result = app.run(terminal, command_receiver, torrent_uri.as_str()) => result,
    };

    ratatui::restore();
    result
}
