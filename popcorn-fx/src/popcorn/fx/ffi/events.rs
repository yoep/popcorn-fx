use std::os::raw::c_char;

use popcorn_fx_core::{from_c_owned, from_c_string, MediaItemC};
use popcorn_fx_core::core::events::PlayerStoppedEvent;

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

impl From<&PlayerStoppedEventC> for PlayerStoppedEvent {
    fn from(value: &PlayerStoppedEventC) -> Self {
        let media = if !value.media.is_null() {
            from_c_owned(value.media).into_identifier()
        } else {
            None
        };
        let time = if !value.time.is_null() {
            Some(unsafe { value.time.read() as u64 })
        } else {
            None
        };
        let duration = if !value.duration.is_null() {
            Some(unsafe { value.duration.read() as u64 })
        } else {
            None
        };

        PlayerStoppedEvent::new(
            from_c_string(value.url),
            media,
            time,
            duration,
        )
    }
}

#[cfg(test)]
mod test {
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

        let result = PlayerStoppedEvent::from(&event);
        let media_result = result.media()
            .expect("expected a media item");

        assert_eq!(url, result.url());
        assert_eq!(id, media_result.imdb_id());
        assert_eq!(Some(&time), result.time());
        assert_eq!(Some(&duration), result.duration());
    }
}
