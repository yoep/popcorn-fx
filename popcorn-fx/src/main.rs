use crate::fx::{PopcornFX, PopcornFxArgs};
use crate::ipc::{
    ApplicationMessageHandler, EventMessageHandler, FavoritesMessageHandler, ImagesMessageHandler,
    IpcChannel, IpcChannelProcessor, LoaderMessageHandler, LogMessageHandler, MediaMessageHandler,
    MessageHandler, PlayerMessageHandler, PlaylistMessageHandler, SettingsMessageHandler,
    SubtitleMessageHandler, TorrentMessageHandler, TrackingMessageHandler, UpdateMessageHandler,
    WatchedMessageHandler,
};
use clap::{CommandFactory, Error, FromArgMatches};
use interprocess::local_socket;
use interprocess::local_socket::tokio::prelude::LocalSocketStream;
use interprocess::local_socket::traits::tokio::Stream;
use interprocess::local_socket::{
    GenericFilePath, GenericNamespaced, NameType, ToFsName, ToNsName,
};
use log::info;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{env, io};
use tokio::select;

mod fx;
mod ipc;

#[tokio::main]
async fn main() -> io::Result<()> {
    let socket_path_str = env::args().nth(1).ok_or(io::Error::new(
        io::ErrorKind::NotFound,
        "expected a socket/pipe path",
    ))?;
    let socket_path = if GenericNamespaced::is_supported() {
        socket_path_str
            .replace("/tmp/", "")
            .to_ns_name::<GenericNamespaced>()?
    } else {
        socket_path_str.to_fs_name::<GenericFilePath>()?
    };

    let conn = LocalSocketStream::connect(socket_path).await?;

    popcorn_fx_args()
        .map(|args| start(conn, args))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
        .await
}

/// Start the Popcorn FX application with the given local socket connection and arguments.
/// This future will keep running until the application is being terminated.
/// Ends immediately if an error occurred while creating the application instance.
async fn start(conn: local_socket::tokio::Stream, args: PopcornFxArgs) -> io::Result<()> {
    let start = Instant::now();
    let popcorn_fx = PopcornFX::new(args)
        .await
        .map(Arc::new)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let time_taken = start.elapsed();
    info!(
        "Created new Popcorn FX instance in {}.{:03} seconds",
        time_taken.as_secs(),
        time_taken.subsec_millis()
    );

    let channel = IpcChannel::new(conn, Duration::from_secs(3));
    let handlers: Vec<Box<dyn MessageHandler>> = vec![
        Box::new(ApplicationMessageHandler::new(popcorn_fx.clone())),
        Box::new(EventMessageHandler::new(
            popcorn_fx.clone(),
            channel.clone(),
        )),
        Box::new(FavoritesMessageHandler::new(
            popcorn_fx.clone(),
            channel.clone(),
        )),
        Box::new(ImagesMessageHandler::new(popcorn_fx.clone())),
        Box::new(LoaderMessageHandler::new(
            popcorn_fx.clone(),
            channel.clone(),
        )),
        Box::new(LogMessageHandler::new()),
        Box::new(MediaMessageHandler::new(popcorn_fx.clone())),
        Box::new(PlayerMessageHandler::new(
            popcorn_fx.clone(),
            channel.clone(),
        )),
        Box::new(PlaylistMessageHandler::new(
            popcorn_fx.clone(),
            channel.clone(),
        )),
        Box::new(SettingsMessageHandler::new(popcorn_fx.clone())),
        Box::new(SubtitleMessageHandler::new(popcorn_fx.clone())),
        Box::new(SubtitleMessageHandler::new(popcorn_fx.clone())),
        Box::new(TorrentMessageHandler::new(
            popcorn_fx.clone(),
            channel.clone(),
        )),
        Box::new(TrackingMessageHandler::new(
            popcorn_fx.clone(),
            channel.clone(),
        )),
        Box::new(UpdateMessageHandler::new(
            popcorn_fx.clone(),
            channel.clone(),
        )),
        Box::new(WatchedMessageHandler::new(
            popcorn_fx.clone(),
            channel.clone(),
        )),
    ];
    let processor = IpcChannelProcessor::new(channel, handlers);

    select! {
        _ = tokio::signal::ctrl_c() => processor.stop(),
        _ = processor.stopped() => (),
    }

    Ok(())
}

/// Try to get the Popcorn FX application arguments.
fn popcorn_fx_args() -> Result<PopcornFxArgs, Error> {
    let args = env::args().skip(2).collect::<Vec<String>>();

    PopcornFxArgs::from_arg_matches(
        &PopcornFxArgs::command()
            .allow_external_subcommands(true)
            .ignore_errors(true)
            .get_matches_from(args),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ipc::test::create_local_socket;

    use interprocess::local_socket::traits::tokio::Listener;
    use log::error;
    use popcorn_fx_core::init_logger;
    use std::path::PathBuf;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::sync::oneshot::channel;
    use tokio::time;

    /// The default set of [PopcornFxArgs] for testing purposes.
    /// This makes it easier to reuse and adopt the args struct when needed without the need to
    /// modify it in each test.
    pub fn default_args(temp_path: &str) -> PopcornFxArgs {
        PopcornFxArgs {
            disable_logger: true,
            disable_mouse: false,
            enable_youtube_video_player: true,
            enable_fx_video_player: true,
            enable_vlc_video_player: true,
            tv: false,
            maximized: false,
            kiosk: false,
            insecure: false,
            app_directory: temp_path.to_string(),
            data_directory: PathBuf::from(temp_path)
                .join("data")
                .to_str()
                .unwrap()
                .to_string(),
            properties: Default::default(),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_start() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (tx, rx) = channel();
        let (name, listener) = create_local_socket();

        tokio::spawn(async move {
            match listener.accept().await {
                Ok(conn) => tx.send(conn).unwrap(),
                Err(e) => error!("Failed to accept incoming connection, {}", e),
            }
        });

        let conn = LocalSocketStream::connect(name).await.unwrap();
        let args = default_args(temp_path);

        tokio::spawn(async move {
            time::sleep(Duration::from_millis(200)).await;
            let conn = rx.await.unwrap();
            drop(conn);
        });

        start(conn, args).await.expect("expected start to succeed");
    }

    #[test]
    fn test_popcorn_fx_args() {
        let result = popcorn_fx_args();

        assert!(
            result.is_ok(),
            "expected the popcorn args to have been returned, got {:?} instead",
            result
        );
    }
}
