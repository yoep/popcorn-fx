use std::os::raw::c_char;

use log::trace;

use popcorn_fx_core::{from_c_owned, from_c_string};
use popcorn_fx_core::core::events::{Event, PlayerStoppedEvent, PlayVideoEvent};
use popcorn_fx_core::core::playback::PlaybackState;

use crate::ffi::MediaItemC;

/// The C compatible [Event] representation.
#[repr(C)]
#[derive(Debug)]
pub enum EventC {
    /// Invoked when the player is being stopped
    PlayerStopped(PlayerStoppedEventC),
    /// Invoked when a new video playback is started
    PlayVideo(PlayVideoEventC),
    /// Invoked when the playback state is changed
    PlaybackStateChanged(PlaybackState),
}

impl From<EventC> for Event {
    fn from(value: EventC) -> Self {
        trace!("Converting from C event {:?}", value);
        match value {
            EventC::PlayerStopped(event_c) => Event::PlayerStopped(PlayerStoppedEvent::from(event_c)),
            EventC::PlayVideo(event_c) => Event::PlayVideo(PlayVideoEvent::from(event_c)),
            EventC::PlaybackStateChanged(new_state) => Event::PlaybackStateChanged(new_state),
        }
    }
}

/// The player stopped event which indicates a video playback has been stopped.
/// It contains the last known information of the video playback right before it was stopped.
#[repr(C)]
#[derive(Debug)]
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

impl From<PlayerStoppedEventC> for PlayerStoppedEvent {
    fn from(value: PlayerStoppedEventC) -> Self {
        trace!("Converting PlayerStoppedEvent from C for {:?}", value);
        let media = if !value.media.is_null() {
            trace!("Converting MediaItem from C for {:?}", value.media);
            from_c_owned(value.media).into_identifier()
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

/// The C compatible [PlayVideo] representation.
#[repr(C)]
#[derive(Debug)]
pub struct PlayVideoEventC {
    /// The video playback url
    pub url: *const c_char,
    /// The video playback title
    pub title: *const c_char,
    /// The media playback show name
    pub show_name: *const c_char,
    /// The optional video playback thumbnail
    pub thumb: *const c_char,
}

impl From<PlayVideoEventC> for PlayVideoEvent {
    fn from(value: PlayVideoEventC) -> Self {
        trace!("Converting PlayVideoEvent from C for {:?}", value);
        let show_name = if !value.show_name.is_null() {
            Some(from_c_string(value.show_name))
        } else {
            None
        };
        let thumb = if !value.thumb.is_null() {
            Some(from_c_string(value.thumb))
        } else {
            None
        };

        Self {
            url: from_c_string(value.url),
            title: from_c_string(value.title),
            subtitle: show_name,
            thumb,
        }
    }
}

#[cfg(test)]
mod test {
    use std::ptr;

    use popcorn_fx_core::{into_c_owned, into_c_string};
    use popcorn_fx_core::core::media::MovieOverview;

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
        let media_result = result.media()
            .expect("expected a media item");

        assert_eq!(url, result.url());
        assert_eq!(id, media_result.imdb_id());
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
    fn test_from_play_video_event_c_to_play_video_event() {
        // Create a PlayVideoEventC instance for testing
        let url = "http://example.com/video.mp4";
        let title = "Test Video";
        let thumb = "http://example.com/video_thumb.png";
        let play_video_event_c = PlayVideoEventC {
            url: into_c_string(url.to_string()),
            title: into_c_string(title.to_string()),
            show_name: ptr::null(),
            thumb: into_c_string(thumb.to_string()),
        };

        // Convert the PlayVideoEventC instance to a PlayVideoEvent instance
        let play_video_event = PlayVideoEvent::from(play_video_event_c);

        // Verify that the conversion was successful
        assert_eq!(play_video_event,
                   PlayVideoEvent {
                       url: url.to_string(),
                       title: title.to_string(),
                       subtitle: None,
                       thumb: Some(thumb.to_string()),
                   }
        );
    }
}
