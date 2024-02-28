use std::os::raw::c_char;
use std::ptr;

use log::trace;

use popcorn_fx_core::{from_c_owned, from_c_string, into_c_owned, into_c_string};
use popcorn_fx_core::core::events::{Event, PlayerChangedEvent};
use popcorn_fx_core::core::playback::PlaybackState;
use popcorn_fx_core::core::players::PlayerChange;
use popcorn_fx_core::core::torrents::TorrentInfo;

use crate::ffi::TorrentInfoC;

/// A type alias for a C-compatible callback function that takes an `EventC` parameter.
///
/// This type alias is used to define functions in Rust that can accept C callback functions
/// with the specified signature.
pub type EventCCallback = extern "C" fn(EventC);

/// The C compatible [Event] representation.
#[repr(C)]
#[derive(Debug)]
pub enum EventC {
    /// Invoked when the player is changed
    /// 1st argument is the new player id, 2nd argument is the new player name
    PlayerChanged(PlayerChangedEventC),
    /// Invoked when the player playback has started for a new media item
    PlayerStarted,
    /// Invoked when the player is being stopped
    PlayerStopped,
    /// Invoked when the playback state is changed
    PlaybackStateChanged(PlaybackState),
    /// Invoked when the watch state of an item is changed
    /// 1st argument is a pointer to the imdb id (C string), 2nd argument is a boolean indicating the new watch state
    WatchStateChanged(*mut c_char, bool),
    /// Invoked when the loading of a media item has started
    LoadingStarted,
    /// Invoked when the loading of a media item has completed
    LoadingCompleted,
    /// Invoked when the torrent details have been loaded
    TorrentDetailsLoaded(TorrentInfoC),
    /// Invoked when the player should be closed
    ClosePlayer,
}

impl EventC {
    pub fn into_event(self) -> Option<Event> {
        trace!("Converting from C event {:?}", self);
        match self {
            EventC::PlayerChanged(e) => Some(Event::PlayerChanged(PlayerChangedEvent::from(e))),
            EventC::PlaybackStateChanged(new_state) => Some(Event::PlaybackStateChanged(new_state)),
            EventC::WatchStateChanged(id, state) => Some(Event::WatchStateChanged(from_c_string(id), state)),
            EventC::LoadingStarted => Some(Event::LoadingStarted),
            EventC::LoadingCompleted => Some(Event::LoadingCompleted),
            EventC::TorrentDetailsLoaded(e) => Some(Event::TorrentDetailsLoaded(TorrentInfo::from(e))),
            EventC::ClosePlayer => Some(Event::ClosePlayer),
            _ => None,
        }
    }
}

impl From<Event> for EventC {
    fn from(value: Event) -> Self {
        trace!("Converting Event to C event for {:?}", value);
        match value {
            Event::PlayerChanged(e) => EventC::PlayerChanged(PlayerChangedEventC::from(e)),
            Event::PlayerStarted(_) => EventC::PlayerStarted,
            Event::PlayerStopped(_) => EventC::PlayerStopped,
            Event::PlaybackStateChanged(e) => EventC::PlaybackStateChanged(e),
            Event::WatchStateChanged(id, state) => EventC::WatchStateChanged(into_c_string(id), state),
            Event::LoadingStarted => EventC::LoadingStarted,
            Event::LoadingCompleted => EventC::LoadingCompleted,
            Event::TorrentDetailsLoaded(e) => EventC::TorrentDetailsLoaded(TorrentInfoC::from(e)),
            Event::ClosePlayer => EventC::ClosePlayer,
        }
    }
}

/// A C-compatible struct representing a player change event.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct PlayerChangedEventC {
    /// The (nullable) old player id
    pub old_player_id: *mut c_char,
    /// The new player id
    pub new_player_id: *mut c_char,
    /// The new player name
    pub new_player_name: *mut c_char,
}

impl From<PlayerChangedEvent> for PlayerChangedEventC {
    fn from(value: PlayerChangedEvent) -> Self {
        let old_player_id = if let Some(id) = value.old_player_id {
            into_c_string(id)
        } else {
            ptr::null_mut()
        };

        Self {
            old_player_id,
            new_player_id: into_c_string(value.new_player_id),
            new_player_name: into_c_string(value.new_player_name),
        }
    }
}

impl From<PlayerChange> for PlayerChangedEventC {
    fn from(value: PlayerChange) -> Self {
        let old_player_id = if let Some(id) = &value.old_player_id {
            into_c_string(id.clone())
        } else {
            ptr::null_mut()
        };

        Self {
            old_player_id,
            new_player_id: into_c_string(value.new_player_id.clone()),
            new_player_name: into_c_string(value.new_player_name.clone()),
        }
    }
}

impl From<PlayerChangedEventC> for PlayerChangedEvent {
    fn from(value: PlayerChangedEventC) -> Self {
        let old_player_id = if !value.old_player_id.is_null() {
            Some(from_c_string(value.old_player_id))
        } else {
            None
        };

        Self {
            old_player_id,
            new_player_id: from_c_string(value.new_player_id),
            new_player_name: from_c_string(value.new_player_name),
        }
    }
}

#[cfg(test)]
mod test {
    use popcorn_fx_core::testing::init_logger;

    use super::*;

    #[test]
    fn test_from_event_c_to_event() {
        let event = EventC::PlaybackStateChanged(PlaybackState::PAUSED).into_event().unwrap();
        assert_eq!(Event::PlaybackStateChanged(PlaybackState::PAUSED), event);

        let event = EventC::ClosePlayer.into_event().unwrap();
        assert_eq!(Event::ClosePlayer, event);

        let event = EventC::LoadingStarted.into_event().unwrap();
        assert_eq!(Event::LoadingStarted, event);

        let event = EventC::LoadingCompleted.into_event().unwrap();
        assert_eq!(Event::LoadingCompleted, event);
    }

    #[test]
    fn test_from_event_c_player_stopped_to_event() {
        let event = EventC::PlayerStopped.into_event();
        assert_eq!(None, event);
    }

    #[test]
    fn test_from_playback_state_changed() {
        init_logger();

        if let Event::PlaybackStateChanged(state) = EventC::PlaybackStateChanged(PlaybackState::BUFFERING).into_event().unwrap() {
            assert_eq!(PlaybackState::BUFFERING, state)
        } else {
            assert!(false, "expected Event::PlaybackStateChanged")
        }
    }

    #[test]
    fn test_player_changed_event_c_from_player_changed_event() {
        let old_player_id = "oldId";
        let new_player_id = "newId";
        let new_player_name = "newName";
        let event = PlayerChangedEvent {
            old_player_id: Some(old_player_id.to_string()),
            new_player_id: new_player_id.to_string(),
            new_player_name: new_player_name.to_string(),
        };

        let result = PlayerChangedEventC::from(event);

        assert_eq!(old_player_id.to_string(), from_c_string(result.old_player_id));
        assert_eq!(new_player_id.to_string(), from_c_string(result.new_player_id));
        assert_eq!(new_player_name.to_string(), from_c_string(result.new_player_name));
    }

    #[test]
    fn test_player_changed_event_c_from_player_changed() {
        let old_player_id = "Foo1";
        let new_player_id = "Foo2";
        let new_player_name = "Foo3";
        let event = PlayerChange {
            old_player_id: Some(old_player_id.to_string()),
            new_player_id: new_player_id.to_string(),
            new_player_name: new_player_name.to_string(),
        };

        let result = PlayerChangedEventC::from(event);

        assert_eq!(old_player_id.to_string(), from_c_string(result.old_player_id));
        assert_eq!(new_player_id.to_string(), from_c_string(result.new_player_id));
        assert_eq!(new_player_name.to_string(), from_c_string(result.new_player_name));
    }
}
