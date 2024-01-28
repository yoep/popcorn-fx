use std::{mem, ptr};
use std::os::raw::c_char;

use log::trace;

use popcorn_fx_core::{from_c_into_boxed, from_c_owned, from_c_string, into_c_owned, into_c_string};
use popcorn_fx_core::core::events::{Event, PlayerChangedEvent, PlayerStartedEvent, PlayerStoppedEvent};
use popcorn_fx_core::core::playback::PlaybackState;
use popcorn_fx_core::core::players::PlayerChange;

use crate::ffi::MediaItemC;

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
    /// 1ste argument is the new player id, 2nd argument is the new player name
    PlayerChanged(PlayerChangedEventC),
    PlayerStarted(PlayerStartedEventC),
    /// Invoked when the player is being stopped
    PlayerStopped(PlayerStoppedEventC),
    /// Invoked when the playback state is changed
    PlaybackStateChanged(PlaybackState),
    /// Invoked when the watch state of an item is changed
    WatchStateChanged(*const c_char, bool),
}

impl From<Event> for EventC {
    fn from(value: Event) -> Self {
        trace!("Converting Event to C event for {:?}", value);
        match value {
            Event::PlayerChanged(e) => EventC::PlayerChanged(PlayerChangedEventC::from(e)),
            Event::PlayerStarted(e) => EventC::PlayerStarted(PlayerStartedEventC::from(e)),
            Event::PlayerStopped(e) => EventC::PlayerStopped(PlayerStoppedEventC::from(e)),
            Event::PlaybackStateChanged(e) => EventC::PlaybackStateChanged(e),
            Event::WatchStateChanged(id, state) => EventC::WatchStateChanged(into_c_string(id), state),
        }
    }
}

impl From<EventC> for Event {
    fn from(value: EventC) -> Self {
        trace!("Converting from C event {:?}", value);
        match value {
            EventC::PlayerChanged(e) => Event::PlayerChanged(PlayerChangedEvent::from(e)),
            EventC::PlayerStarted(e) => Event::PlayerStarted(PlayerStartedEvent::from(e)),
            EventC::PlayerStopped(event_c) => Event::PlayerStopped(PlayerStoppedEvent::from(event_c)),
            EventC::PlaybackStateChanged(new_state) => Event::PlaybackStateChanged(new_state),
            EventC::WatchStateChanged(id, state) => Event::WatchStateChanged(from_c_string(id), state),
        }
    }
}

/// The player stopped event which indicates a video playback has been stopped.
/// It contains the last known information of the video playback right before it was stopped.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct PlayerStoppedEventC {
    /// The playback url that was being played
    pub url: *const c_char,
    /// The last known video time of the player in millis
    pub time: *const i64,
    /// The duration of the video playback in millis
    pub duration: *const i64,
    /// The optional media item that was being played
    pub media: *mut MediaItemC,
}

impl From<PlayerStoppedEvent> for PlayerStoppedEventC {
    fn from(value: PlayerStoppedEvent) -> Self {
        let time = if let Some(time) = value.time {
            time as *const i64
        } else {
            ptr::null()
        };
        let duration = if let Some(duration) = value.duration {
            duration as *const i64
        } else {
            ptr::null()
        };
        let media = if let Some(media) = value.media {
            into_c_owned(MediaItemC::from(media))
        } else {
            ptr::null_mut()
        };

        Self {
            url: into_c_string(value.url),
            time,
            duration,
            media,
        }
    }
}

impl From<PlayerStoppedEventC> for PlayerStoppedEvent {
    fn from(value: PlayerStoppedEventC) -> Self {
        trace!("Converting PlayerStoppedEvent from C for {:?}", value);
        let media = if !value.media.is_null() {
            trace!("Converting MediaItem from C for {:?}", value.media);
            let media_item = from_c_into_boxed(value.media);
            let identifier = media_item.as_identifier();
            mem::forget(media_item);
            identifier
        } else {
            None
        };
        let time = if !value.time.is_null() {
            trace!("Converting PlayerStoppedEventC.time from C for {:?}", value.time);
            Some(unsafe { value.time.read() as u64 })
        } else {
            None
        };
        let duration = if !value.duration.is_null() {
            trace!("Converting PlayerStoppedEventC.duration from C for {:?}", value.duration);
            Some(unsafe { value.duration.read() as u64 })
        } else {
            None
        };

        PlayerStoppedEvent {
            url: from_c_string(value.url),
            media,
            time,
            duration,
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

    use popcorn_fx_core::{into_c_owned, into_c_string};
    use popcorn_fx_core::core::media::MovieOverview;
    use popcorn_fx_core::testing::init_logger;

    use super::*;

    #[test]
    fn test_from_player_stopped_event() {
        let id = "tt11224455";
        let url = "http://localhost/my-video.mkv";
        let movie = MovieOverview::new(
            "Lorem ipsum".to_string(),
            id.to_string(),
            "2022".to_string(),
        );
        let time = 20000;
        let duration = 1800000;
        let event = PlayerStoppedEventC {
            url: into_c_string(url.to_string()),
            media: into_c_owned(MediaItemC::from(movie.clone())),
            time: into_c_owned(20000),
            duration: into_c_owned(1800000),
        };

        let result = PlayerStoppedEvent::from(event);

        assert_eq!(url, result.url());
        // TODO: enable once memory issues are fixed
        // assert_eq!(id, result.media().expect("expected a media item").imdb_id());
        assert_eq!(Some(&time), result.time());
        assert_eq!(Some(&duration), result.duration());
    }

    #[test]
    fn test_from_event_c_to_event_player_stopped() {
        // Create a PlayerStoppedEventC instance for testing
        let url = into_c_string("http://example.com/video.mp4".to_string());
        let time = Box::new(1000);
        let duration = Box::new(5000);
        let media_item_c = Box::new(MediaItemC {
            movie_overview: ptr::null_mut(),
            movie_details: ptr::null_mut(),
            show_overview: ptr::null_mut(),
            show_details: ptr::null_mut(),
            episode: ptr::null_mut(),
        });
        let player_stopped_event_c = PlayerStoppedEventC {
            url,
            time: Box::into_raw(time),
            duration: Box::into_raw(duration),
            media: Box::into_raw(media_item_c),
        };

        // Convert the PlayerStoppedEventC instance to an Event instance
        let event_c = EventC::PlayerStopped(player_stopped_event_c);
        let event = Event::from(event_c);

        // Verify that the conversion was successful
        match event {
            Event::PlayerStopped(player_stopped_event) => {
                assert_eq!(player_stopped_event.time, Some(1000 as u64));
                assert_eq!(player_stopped_event.duration, Some(5000 as u64));
                assert!(player_stopped_event.media.is_none(), "expected no media item");
            }
            _ => panic!("Expected PlayerStopped event"),
        }
    }

    #[test]
    fn test_from_playback_state_changed() {
        init_logger();

        if let Event::PlaybackStateChanged(state) = Event::from(EventC::PlaybackStateChanged(PlaybackState::BUFFERING)) {
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
