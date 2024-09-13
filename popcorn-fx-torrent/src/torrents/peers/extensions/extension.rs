use crate::torrents::peers::extensions::errors;
use crate::torrents::peers::peer_commands::PeerCommandSender;
use errors::Result;
use std::collections::HashMap;
use std::fmt::Debug;

/// The extension unique name
pub type ExtensionName = String;
/// The extension unique identifier
pub type ExtensionNumber = u8;
/// The extensions used in the BitTorrent protocol.
pub type Extensions = HashMap<ExtensionName, ExtensionNumber>;

pub trait Extension: Debug {
    /// Retrieve the name of the extension
    fn name(&self) -> String;

    /// Process the given extension message payload.
    ///
    /// # Arguments
    ///
    /// * `payload` - The payload of the extension message
    /// * `command_sender` - The command sender to interact with the underlying peer
    ///
    /// # Returns
    ///
    /// Return an error when the extension fails to process the payload successfully.
    fn handle(&mut self, payload: Vec<u8>, command_sender: PeerCommandSender) -> Result<()>;

    fn to_bytes(&self, command_sender: PeerCommandSender) -> Vec<u8>;

    /// Create a new instance of the extension.
    ///
    /// # Returns
    ///
    /// A new instance of this extension.
    fn clone_box(&self) -> Box<dyn Extension>;
}
