use std::os::raw::c_char;
use std::ptr;

use log::trace;

use popcorn_fx_core::{from_c_owned, from_c_string, into_c_owned, into_c_string};
use popcorn_fx_core::core::events::{Event, PlayerChangedEvent, PlayerStartedEvent};
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
    PlayerStarted(PlayerStartedEventC),
    /// Invoked when the player is being stopped
    PlayerStopped,
    /// Invoked when the playback state is changed
    PlaybackStateChanged(PlaybackState),
    /// Invoked when the watch state of an item is changed
    /// 1st argument is a pointer to the imdb id (C string), 2nd argument is a boolean indicating the new watch state
    WatchStateChanged(*const c_char, bool),
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
            EventC::PlayerStarted(e) => Some(Event::PlayerStarted(PlayerStartedEvent::from(e))),
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
            Event::PlayerStarted(e) => EventC::PlayerStarted(PlayerStartedEventC::from(e)),
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
    pub old_player_id: *const c_char,
    /// The new player id
    pub new_player_id: *const c_char,
    /// The new player name
    pub new_player_name: *const c_char,
}

impl From<PlayerChangedEvent> for PlayerChangedEventC {
    fn from(value: PlayerChangedEvent) -> Self {
        let old_player_id = if let Some(id) = value.old_player_id {
            into_c_string(id)
        } else {
            ptr::null()
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
            ptr::null()
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

#[repr(C)]
#[derive(Debug, Clone)]
pub struct PlayerStartedEventC {
    pub url: *const c_char,
    pub title: *const c_char,
    pub thumbnail: *const c_char,
    pub quality: *const c_char,
    pub auto_resume_timestamp: *mut u64,
    pub subtitles_enabled: bool,
}

impl From<PlayerStartedEvent> for PlayerStartedEventC {
    fn from(value: PlayerStartedEvent) -> Self {
        let thumbnail = if let Some(e) = value.thumbnail {
            into_c_string(e)
        } else {
            ptr::null()
        };
        let quality = if let Some(e) = value.quality {
            into_c_string(e)
        } else {
            ptr::null()
        };
        let auto_resume_timestamp = if let Some(e) = value.auto_resume_timestamp {
            into_c_owned(e)
        } else {
            ptr::null_mut()
        };

        Self {
            url: into_c_string(value.url),
            title: into_c_string(value.title),
            thumbnail,
            quality,
            auto_resume_timestamp,
            subtitles_enabled: value.subtitles_enabled,
        }
    }
}

impl From<PlayerStartedEventC> for PlayerStartedEvent {
    fn from(value: PlayerStartedEventC) -> Self {
        let thumbnail = if !value.thumbnail.is_null() {
            Some(from_c_string(value.thumbnail))
        } else {
            None
        };
        let quality = if !value.quality.is_null() {
            Some(from_c_string(value.quality))
        } else {
            None
        };
        let auto_resume_timestamp = if !value.auto_resume_timestamp.is_null() {
            Some(from_c_owned(value.auto_resume_timestamp))
        } else {
            None
        };

        Self {
            url: from_c_string(value.url),
            title: from_c_string(value.title),
            thumbnail,
            quality,
            auto_resume_timestamp,
            subtitles_enabled: value.subtitles_enabled,
        }
    }
}


#[cfg(test)]
mod test {
    use std::ptr;

    use popcorn_fx_core::into_c_string;
    use popcorn_fx_core::testing::init_logger;

    use crate::ffi::MediaItemC;

    use super::*;

    #[test]
    fn test_from_event_c_to_event() {
        // Create a PlayerStoppedEventC instance for testing
        let url = into_c_string("https://example.com/video.mp4".to_string());
        let time = 1000;
        let duration = 5000;
        let media_item_c = Box::new(MediaItemC {
            movie_overview: ptr::null_mut(),
            movie_details: ptr::null_mut(),
            show_overview: ptr::null_mut(),
            show_details: ptr::null_mut(),
            episode: ptr::null_mut(),
        });

        // Convert the PlayerStoppedEventC instance to an Event instance
        let event = EventC::ClosePlayer.into_event().unwrap();

        // Verify that the conversion was successful
        assert_eq!(Event::ClosePlayer, event);
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

    #[test]
    fn test_player_started_event_c_from() {
        let url = "https://localhost:8081/my-video.mkv";
        let title = "MyTitle";
        let thumb = "https://imgur.com/MyThumb.jpg";
        let event = PlayerStartedEvent {
            url: url.to_string(),
            title: title.to_string(),
            thumbnail: Some(thumb.to_string()),
            quality: None,
            auto_resume_timestamp: None,
            subtitles_enabled: true,
        };

        let result = PlayerStartedEventC::from(event);

        assert_eq!(url.to_string(), from_c_string(result.url));
        assert_eq!(title.to_string(), from_c_string(result.title));
        assert_eq!(thumb.to_string(), from_c_string(result.thumbnail));
        assert_eq!(true, result.subtitles_enabled, "expected the subtitles to have been enabled");
    }
}
