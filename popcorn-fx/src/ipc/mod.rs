pub use application::*;
pub use channel::*;
pub use errors::*;
pub use events::*;
pub use favorites::*;
pub use images::*;
pub use loader::*;
pub use log::*;
pub use media::*;
pub use message::*;
pub use player::*;
pub use playlist::*;
pub use settings::*;
pub use subtitle::*;
pub use torrent::*;
pub use tracking::*;
pub use update::*;
pub use watched::*;

mod application;
mod channel;
mod errors;
mod events;
mod favorites;
mod images;
mod loader;
mod log;
mod mappings;
mod media;
mod message;
mod player;
mod playlist;
mod proto;
mod settings;
mod subtitle;
mod torrent;
mod tracking;
mod update;
mod watched;

#[cfg(test)]
pub(crate) mod test {
    use super::*;

    use crate::ipc::proto::message::FxMessage;

    use async_trait::async_trait;
    use mockall::mock;
    use std::fmt::{Display, Formatter};
    use std::time::Duration;
    use tokio::net::TcpStream;
    use tokio::sync::oneshot;

    mock! {
        #[derive(Debug)]
        pub MessageHandler {}

        #[async_trait]
        impl MessageHandler for MessageHandler {
            fn name(&self) -> &str {
                "MockMessageHandler"
            }
            fn is_supported(&self, message_type: &str) -> bool;
            async fn process(&self, message: FxMessage, channel: &IpcChannel) -> Result<()>;
        }
    }

    impl Display for MockMessageHandler {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "MockMessageHandler")
        }
    }

    /// Create a new socket listener.
    ///
    /// # Returns
    ///
    /// It returns the socket listener and listener socket address.
    #[macro_export]
    macro_rules! create_local_socket {
        () => {{
            use core::net::SocketAddr;
            use tokio::net::TcpListener;

            let socket_address: SocketAddr = ([127, 0, 0, 1], 0).into();
            let listener = TcpListener::bind(socket_address).await.unwrap();
            let socket_address = listener.local_addr().unwrap();

            (listener, socket_address)
        }};
    }

    /// Asynchronously creates a pair of interconnected IPC channels.
    ///
    /// This function sets up a local IPC (inter-process communication) channel using a local socket.
    /// It returns a tuple of two [`IpcChannel`] instances:
    ///
    /// - The first channel is the accepting connection channel.
    /// - The second channel is the outgoing connection channel.
    ///
    /// # Panics
    ///
    /// - Panics if connecting to the local socket fails.
    /// - Panics if the accepted connection cannot be received.
    pub async fn create_channel_pair() -> (IpcChannel, IpcChannel) {
        let (listener, socket_address) = create_local_socket!();
        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            match listener.accept().await {
                Ok((stream, _)) => tx.send(stream).unwrap(),
                Err(e) => panic!("{}", e),
            }
        });

        let stream = TcpStream::connect(socket_address).await.unwrap();
        let outgoing_channel = IpcChannel::new(stream, Duration::from_secs(1));

        let conn = rx.await.expect("failed to receive incoming connection");
        let incoming_channel = IpcChannel::new(conn, Duration::from_secs(1));

        (incoming_channel, outgoing_channel)
    }

    /// A macro wrapper for [`tokio::time::timeout`] that awaits a future with a timeout duration.
    ///
    /// # Returns
    ///
    /// It returns the future result or timeout.
    #[macro_export]
    macro_rules! timeout {
        ($future:expr, $duration:expr) => {{
            timeout!(
                $future,
                $duration,
                format!(
                    "operation timed-out after {}.{:03}s",
                    ($duration).as_secs(),
                    ($duration).subsec_millis()
                )
            )
        }};
        ($future:expr, $duration:expr, $message:expr) => {{
            use tokio::time::timeout;
            let future = $future;
            let duration = $duration;
            let message = $message;
            let __message: &str = ::core::convert::AsRef::<str>::as_ref(&message);

            timeout(duration, future).await.expect(__message)
        }};
    }
}
