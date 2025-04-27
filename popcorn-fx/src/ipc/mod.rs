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
    use interprocess::local_socket;
    use interprocess::local_socket::tokio::prelude::LocalSocketStream;
    use interprocess::local_socket::traits::tokio::{Listener, Stream};
    use interprocess::local_socket::{GenericNamespaced, ListenerOptions, Name, ToNsName};
    use mockall::mock;
    use rand::{rng, Rng};
    use std::fmt::{Display, Formatter};
    use std::time::Duration;
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

    /// Create a new local socket listener.
    ///
    /// # Returns
    ///
    /// It returns the unique socket name and the listener which accepts incoming connections.
    pub fn create_local_socket<'s>() -> (Name<'s>, local_socket::tokio::Listener) {
        let name_id = rng().random_range(0..60000);
        let name = format!("fx-{}.sock", name_id)
            .to_ns_name::<GenericNamespaced>()
            .unwrap();
        let opts = ListenerOptions::new().name(name.clone());

        (name, opts.create_tokio().unwrap())
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
        let (name, listener) = create_local_socket();
        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            match listener.accept().await {
                Ok(conn) => tx.send(conn).unwrap(),
                Err(e) => panic!("{}", e),
            }
        });

        let conn = LocalSocketStream::connect(name)
            .await
            .expect("failed to connect to local socket");
        let outgoing_channel = IpcChannel::new(conn, Duration::from_secs(1));

        let conn = rx.await.expect("failed to receive incoming connection");
        let incoming_channel = IpcChannel::new(conn, Duration::from_secs(1));

        (incoming_channel, outgoing_channel)
    }

    #[macro_export]
    macro_rules! try_recv {
        ($receiver:expr, $timeout:expr) => {
            tokio::select! {
                _ = tokio::time::sleep($timeout) => panic!("receiver timed out after {}ms", $timeout.as_secs()),
                result = $receiver => result,
            }
        };
    }
}
