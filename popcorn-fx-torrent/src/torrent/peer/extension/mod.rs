pub use errors::*;

#[cfg(feature = "extension-donthave")]
pub mod donthave;
mod errors;
#[cfg(feature = "extension-holepunch")]
pub mod holepunch;
#[cfg(feature = "extension-metadata")]
pub mod metadata;
#[cfg(feature = "extension-pex")]
pub mod pex;

use crate::torrent::peer::{PeerContext, PeerEvent};
use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;

/// The extension unique name
pub type ExtensionName = String;
/// The extension unique identifier
pub type ExtensionNumber = u8;
/// The registry of the known extensions and their identifiers
pub type ExtensionRegistry = HashMap<ExtensionName, ExtensionNumber>;
/// The list type of enabled extensions
pub type Extensions = Vec<Box<dyn Extension>>;

/// A peer extension that is used within the BitTorrent protocol.
/// An extension can only be activated when the remote peer supports **BEP10**.
///
/// Extensions are registered at the [crate::torrent::Session] level.
/// An extension is then cloned through the [Extension::clone_boxed] method for each created peer connection in a torrent.
/// This means that the extension can store peer related information internally for later use.
#[async_trait]
pub trait Extension: Debug + Send + Sync {
    /// Get the unique extension protocol name.
    fn name(&self) -> &str;

    /// Handle the given extension message payload which has been received from the remote peer.
    /// If you want to store data internally, make use of [tokio::sync::Mutex] or [tokio::sync::RwLock].
    ///
    /// # Arguments
    ///
    /// * `payload` - The payload message of the extension from the remote peer
    /// * `command_sender` - The command sender to interact with the underlying peer
    ///
    /// # Returns
    ///
    /// Return an error when the extension fails to process the payload successfully.
    async fn handle<'a>(&'a self, payload: &'a [u8], peer: &'a PeerContext) -> Result<()>;

    /// Invoked when an event is raised by a peer and this extension is supported.
    /// Keep in mind that the [PeerEvent::HandshakeCompleted] event will never be received by an extension
    /// as the supported remote extensions are only known after the extended handshake.
    ///
    /// # Arguments
    ///
    /// * `event` - The event raised by the peer
    /// * `peer` - The peer that raised the event
    async fn on<'a>(&'a self, event: &'a PeerEvent, peer: &'a PeerContext);
}
