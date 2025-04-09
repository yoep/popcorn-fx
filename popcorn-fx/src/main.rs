use crate::fx::{PopcornFX, PopcornFxArgs};
use crate::ipc::channel::IpcChannel;
use clap::{CommandFactory, FromArgMatches};
use interprocess::local_socket::tokio::prelude::LocalSocketStream;
use interprocess::local_socket::traits::tokio::Stream;
use interprocess::local_socket::{
    GenericFilePath, GenericNamespaced, NameType, ToFsName, ToNsName,
};
use log::info;
use std::time::Instant;
use std::{env, io};
use tokio::select;

mod fx;
mod ipc;

#[tokio::main]
async fn main() -> io::Result<()> {
    let socket_path_str = env::args().nth(1).expect("expected a socket/pipe path");
    let socket_path = if GenericNamespaced::is_supported() {
        socket_path_str
            .replace("/tmp/", "")
            .to_ns_name::<GenericNamespaced>()?
    } else {
        socket_path_str.to_fs_name::<GenericFilePath>()?
    };

    let conn = LocalSocketStream::connect(socket_path).await?;

    let start = Instant::now();
    let popcorn_fx = new_instance()
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let time_taken = start.elapsed();
    info!(
        "Created new Popcorn FX instance in {}.{:03} seconds",
        time_taken.as_secs(),
        time_taken.subsec_millis()
    );

    let channel = IpcChannel::new(conn, popcorn_fx);

    select! {
        _ = tokio::signal::ctrl_c() => channel.close(),
        _ = channel.execute() => (),
    }

    Ok(())
}

async fn new_instance() -> fx::Result<PopcornFX> {
    let args = env::args().skip(2).collect::<Vec<String>>();
    let matches = PopcornFxArgs::command()
        .allow_external_subcommands(true)
        .ignore_errors(true)
        .get_matches_from(args);
    let args = PopcornFxArgs::from_arg_matches(&matches).expect("expected valid args");

    PopcornFX::new_async(args).await
}
