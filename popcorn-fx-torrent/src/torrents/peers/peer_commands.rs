use crate::torrents::channel::{CommandInstruction, CommandReceiver, CommandSender};
use crate::torrents::peers::bt_connection::{ExtensionFlag, Message};
use crate::torrents::peers::{PeerId, PeerState, RemotePeer, Result};

/// The command instruction of the action that needs to be taken on the peer
pub(crate) type PeerCommandInstruction = CommandInstruction<PeerCommand, PeerCommandResponse>;

/// The peer specific command sender
pub type PeerCommandSender = CommandSender<PeerCommand, PeerCommandResponse>;

/// The peer specific command receiver
pub(crate) type PeerCommandReceiver = CommandReceiver<PeerCommand, PeerCommandResponse>;

#[derive(Debug, Clone, PartialEq)]
pub enum PeerCommand {
    /// Request the client peer id
    ClientId,
    /// Request the remote peer information, if already known
    Remote,
    /// Retrieve the current state of the peer connection
    State,
    /// Retrieve the supported extensions of the remote peer
    SupportedExtensions,
    /// Send the extended handshake when the remote peer supports it
    SendExtendedHandshake,
    /// Send that we have nothing available
    SendHaveNone,
    /// Send a received peer message
    Message(Message),
    /// Close the peer connection
    Close,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PeerCommandResponse {
    /// The client peer id response
    ClientId(PeerId),
    /// The remote peer information response
    Remote(Option<RemotePeer>),
    /// The current peer state response
    State(PeerState),
    /// The remote peer supported extensions
    SupportedExtensions(ExtensionFlag),
    /// The extended handshake response
    SendExtendedHandshake(Result<()>),
}
