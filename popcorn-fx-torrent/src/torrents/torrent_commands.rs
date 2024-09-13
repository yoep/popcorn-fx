use crate::torrents::channel::{CommandInstruction, CommandReceiver, CommandSender};
use crate::torrents::trackers::Announcement;
use crate::torrents::{TorrentCallback, TorrentInfo};
use popcorn_fx_core::core::CallbackHandle;
use std::fmt::Debug;
use url::Url;

/// The command instruction of the action that needs to be taken on the torrent
pub(crate) type TorrentCommandInstruction =
    CommandInstruction<TorrentCommand, TorrentCommandResponse>;

/// The torrent specific command sender
pub(crate) type TorrentCommandSender = CommandSender<TorrentCommand, TorrentCommandResponse>;

/// The torrent specific command receiver
pub(crate) type TorrentCommandReceiver = CommandReceiver<TorrentCommand, TorrentCommandResponse>;

pub(crate) enum TorrentCommand {
    /// Retrieve the metadata of the torrent
    Metadata,
    /// Retrieve the active trackers of the torrent
    ActiveTrackers,
    /// Start announcing the torrent to the trackers
    StartAnnouncing,
    /// Announce the torrent to all trackers
    AnnounceAll,
    /// Add a new tracker to the torrent
    AddTracker(Url, u8),
    /// Register a new torrent callback
    AddCallback(TorrentCallback),
    /// Remove an existing callback handle from the torrent
    RemoveCallback(CallbackHandle),
}

impl PartialEq for TorrentCommand {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TorrentCommand::Metadata, TorrentCommand::Metadata) => true,
            (TorrentCommand::ActiveTrackers, TorrentCommand::ActiveTrackers) => true,
            (TorrentCommand::StartAnnouncing, TorrentCommand::StartAnnouncing) => true,
            (TorrentCommand::AnnounceAll, TorrentCommand::AnnounceAll) => true,
            (TorrentCommand::AddTracker(_, _), TorrentCommand::AddTracker(_, _)) => true,
            (TorrentCommand::AddCallback(_), TorrentCommand::AddCallback(_)) => true,
            (TorrentCommand::RemoveCallback(_), TorrentCommand::RemoveCallback(_)) => true,
            _ => false,
        }
    }
}

impl Debug for TorrentCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TorrentCommand::Metadata => write!(f, "TorrentCommand::Metadata"),
            TorrentCommand::ActiveTrackers => write!(f, "TorrentCommand::ActiveTrackers"),
            TorrentCommand::StartAnnouncing => write!(f, "TorrentCommand::StartAnnouncing"),
            TorrentCommand::AnnounceAll => write!(f, "TorrentCommand::AnnounceAll"),
            TorrentCommand::AddTracker(url, tier) => {
                write!(f, "TorrentCommand::AddTracker({}, {})", url, tier)
            }
            TorrentCommand::AddCallback(_) => write!(f, "TorrentCommand::AddCallback"),
            TorrentCommand::RemoveCallback(e) => write!(f, "TorrentCommand::RemoveCallback({})", e),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum TorrentCommandResponse {
    /// Returns the last known metadata info of the torrent
    Metadata(TorrentInfo),
    /// Returns the announcement result for the torrent
    AnnounceAll(Announcement),
    /// Returns the active trackers result for the torrent
    ActiveTrackers(Vec<Url>),
    /// Returns the callback handle of the registered callback
    AddCallback(CallbackHandle),
}
