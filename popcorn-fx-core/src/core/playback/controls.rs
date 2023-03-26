use std::sync::Arc;

use log::warn;

use crate::core::events::{DEFAULT_ORDER, Event, EventPublisher, PlayVideoEvent};
use crate::core::platform::PlatformData;
use crate::core::playback::{MediaInfo, MediaNotificationEvent, PlaybackState};

/// Manages media playback state and communication with the operating system's media control system for
/// the application.
///
/// The `PlaybackControls` struct is responsible for notifying the operating system of media playback
/// state and handling any incoming media control events from the system. It provides an interface for
/// managing media playback in the application and allows the application to respond to media control
/// events from the user or the operating system.
///
/// This struct requires a platform-specific implementation of the `Platform` trait, which provides
/// the necessary system-level functionality for managing media playback and handling media control
/// events. An `EventPublisher` can also be provided to listen on events related to media playback state
/// changes. The `EventPublisher` is optional and is responsible for automatically translating
/// playback state events to the operating system's media notifications. If an `EventPublisher` is not
/// provided, the `PlaybackControls` struct will still function normally, but the application will not
/// be able to publish media playback state events to the operating system on it's own.
///
/// # Examples
///
/// ```no_run
/// use std::sync::Arc;
/// use popcorn_fx_core::core::events::EventPublisher;
/// use popcorn_fx_core::core::playback::PlaybackControls;
///
/// let controls = PlaybackControls::builder()
///     .platform(Arc::new(MyPlatform::new()))
///     .event_publisher(Arc::new(EventPublisher::new()))
///     .build();
/// ```
#[derive(Debug)]
pub struct PlaybackControls {
    inner: Arc<InnerPlaybackControls>,
}

impl PlaybackControls {
    /// Creates a new `PlaybackControlsBuilder`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use popcorn_fx_core::core::events::EventPublisher;
    /// use popcorn_fx_core::core::playback::PlaybackControls;
    ///
    /// let controls = PlaybackControls::builder()
    ///     .platform(Arc::new(MyPlatform::new()))
    ///     .event_publisher(Arc::new(EventPublisher::new()))
    ///     .build();
    /// ```
    pub fn builder() -> PlaybackControlsBuilder {
        PlaybackControlsBuilder::default()
    }
}

/// A builder for `PlaybackControls`.
///
/// # Examples
///
/// ```no_run
/// use std::sync::Arc;
/// use popcorn_fx_core::core::events::EventPublisher;
/// use popcorn_fx_core::core::playback::PlaybackControls;
///
/// let controls = PlaybackControls::builder()
///     .platform(Arc::new(MyPlatform::new()))
///     .event_publisher(Arc::new(EventPublisher::new()))
///     .build();
/// ```
#[derive(Default)]
pub struct PlaybackControlsBuilder {
    platform: Option<Arc<Box<dyn PlatformData>>>,
    event_publisher: Option<Arc<EventPublisher>>,
}

impl PlaybackControlsBuilder {
    /// Sets the `platform` field for the `PlaybackControls`.
    ///
    /// # Panics
    ///
    /// Panics if the `platform` is not set when `build()` is called.
    pub fn platform(mut self, platform: Arc<Box<dyn PlatformData>>) -> Self {
        self.platform = Some(platform);
        self
    }

    /// Sets the `event_publisher` field for the `PlaybackControls`.
    /// When not set, the [PlaybackControls] won't subscribe to any events.
    pub fn event_publisher(mut self, event_publisher: Arc<EventPublisher>) -> Self {
        self.event_publisher = Some(event_publisher);
        self
    }

    /// Builds a new `PlaybackControls`.
    ///
    /// # Panics
    ///
    /// Panics if the `platform` is not set.
    pub fn build(self) -> PlaybackControls {
        let instance = PlaybackControls {
            inner: Arc::new(InnerPlaybackControls {
                platform: self.platform.expect("Platform not set"),
            }),
        };

        let inner = instance.inner.clone();
        instance.inner.platform.register(Box::new(move |event| {

        }));

        let inner = instance.inner.clone();
        if let Some(event_publisher) = self.event_publisher {
            event_publisher.register(Box::new(move |event| {
                match &event {
                    Event::PlayVideo(play_event) => {
                        inner.notify_media_playback(play_event.clone());
                    }
                    Event::PlaybackStateChanged(new_state) => {
                        inner.notify_media_state_changed(new_state.clone())
                    }
                    _ => {}
                }
                Some(event)
            }), DEFAULT_ORDER);
        } else {
            warn!("Unable to handle control events for PlaybackControls, EventPublisher has not been set");
        }

        instance
    }
}

#[derive(Debug)]
struct InnerPlaybackControls {
    platform: Arc<Box<dyn PlatformData>>,
}

impl InnerPlaybackControls {
    fn notify_media_playback(&self, event: PlayVideoEvent) {
        self.platform.notify_media_event(MediaNotificationEvent::PlaybackStarted(MediaInfo {
            title: event.title.clone(),
            show_name: event.show_name,
            thumb: event.thumb,
        }))
    }

    fn notify_media_state_changed(&self, state: PlaybackState) {
        match state {
            PlaybackState::PLAYING => self.platform.notify_media_event(MediaNotificationEvent::StatePlaying),
            PlaybackState::PAUSED => self.platform.notify_media_event(MediaNotificationEvent::StatePaused),
            _ => {}
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use crate::core::platform::MockDummyPlatformData;
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_on_play_video_event() {
        init_logger();
        let (tx, rx) = channel();
        let mut platform = MockDummyPlatformData::new();
        platform.expect_notify_media_event()
            .returning(move |notification: MediaNotificationEvent| tx.send(notification).unwrap());
        let event_publisher = Arc::new(EventPublisher::default());
        let _controls = PlaybackControls::builder()
            .platform(Arc::new(Box::new(platform)))
            .event_publisher(event_publisher.clone())
            .build();

        event_publisher.publish(Event::PlayVideo(PlayVideoEvent {
            url: "http://localhost/video.mp4".to_string(),
            title: "Lorem ipsum".to_string(),
            show_name: Some("My showname".to_string()),
            thumb: None,
        }));

        let notif_result = rx.recv_timeout(Duration::from_millis(100)).unwrap();
        match notif_result {
            MediaNotificationEvent::PlaybackStarted(info) => assert_eq!(info, MediaInfo {
                title: "Lorem ipsum".to_string(),
                show_name: Some("My showname".to_string()),
                thumb: None,
            }),
            _=> panic!("Expected MediaNotificationEvent::PlaybackStarted")
        }
    }
}