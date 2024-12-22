use std::sync::Arc;

use log::{debug, trace, warn};

use crate::core::event::{Event, EventPublisher, PlayerStartedEvent, DEFAULT_ORDER};
use crate::core::platform::{PlatformData, PlatformEvent};
use crate::core::playback::{
    MediaInfo, MediaNotificationEvent, PlaybackControlCallback, PlaybackControlEvent, PlaybackState,
};
use crate::core::{Callbacks, CoreCallbacks};

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
/// use popcorn_fx_core::core::event::EventPublisher;
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
    /// use popcorn_fx_core::core::event::EventPublisher;
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

    /// Register a new callback listener for the [PlaybackControlEvent]'s.
    pub fn register(&self, callback: PlaybackControlCallback) {
        self.inner.register(callback);
    }
}

/// A builder for `PlaybackControls`.
///
/// # Examples
///
/// ```no_run
/// use std::sync::Arc;
/// use popcorn_fx_core::core::event::EventPublisher;
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
                callbacks: Default::default(),
            }),
        };

        let inner = instance.inner.clone();
        instance
            .inner
            .platform
            .register(Box::new(move |event| inner.handle_event(event)));

        let inner = instance.inner.clone();
        if let Some(event_publisher) = self.event_publisher {
            event_publisher.register(
                Box::new(move |event| {
                    match &event {
                        Event::PlayerStarted(play_event) => {
                            inner.notify_media_playback(play_event.clone());
                        }
                        Event::PlaybackStateChanged(new_state) => {
                            inner.notify_media_state_changed(new_state.clone())
                        }
                        Event::PlayerStopped(_) => inner.notify_media_stopped(),
                        _ => {}
                    }
                    Some(event)
                }),
                DEFAULT_ORDER,
            );
        } else {
            warn!("Unable to handle control events for PlaybackControls, EventPublisher has not been set");
        }

        instance
    }
}

#[derive(Debug)]
struct InnerPlaybackControls {
    platform: Arc<Box<dyn PlatformData>>,
    callbacks: CoreCallbacks<PlaybackControlEvent>,
}

impl InnerPlaybackControls {
    fn notify_media_playback(&self, event: PlayerStartedEvent) {
        debug!("Notifying system that a new media playback is being started");
        self.platform
            .notify_media_event(MediaNotificationEvent::StateStarting(MediaInfo {
                title: event.title,
                subtitle: event.quality,
                thumb: event.thumbnail,
            }))
    }

    fn notify_media_state_changed(&self, state: PlaybackState) {
        debug!(
            "Notifying system that the media playback state has changed to {}",
            state
        );
        match state {
            PlaybackState::PLAYING => self
                .platform
                .notify_media_event(MediaNotificationEvent::StatePlaying),
            PlaybackState::PAUSED => self
                .platform
                .notify_media_event(MediaNotificationEvent::StatePaused),
            _ => {}
        }
    }

    fn notify_media_stopped(&self) {
        debug!("Notifying system that the media playback has stopped");
        self.platform
            .notify_media_event(MediaNotificationEvent::StateStopped)
    }

    fn register(&self, callback: PlaybackControlCallback) {
        self.callbacks.add_callback(callback);
    }

    fn handle_event(&self, event: PlatformEvent) {
        trace!("Handling platform event {:?}", event);
        match event {
            PlatformEvent::TogglePlaybackState => self
                .callbacks
                .invoke(PlaybackControlEvent::TogglePlaybackState),
            PlatformEvent::ForwardMedia => self.callbacks.invoke(PlaybackControlEvent::Forward),
            PlatformEvent::RewindMedia => self.callbacks.invoke(PlaybackControlEvent::Rewind),
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use crate::core::event::PlayerStoppedEvent;
    use crate::testing::{init_logger, MockDummyPlatformData};

    use super::*;

    #[test]
    fn test_platform_event_toggle_playback() {
        init_logger();
        let (tx, rx) = channel();
        let (tx_ce, rx_ce) = channel();
        let mut platform = MockDummyPlatformData::new();
        platform
            .expect_register()
            .returning(move |callback| tx.send(callback).unwrap());
        let event_publisher = Arc::new(EventPublisher::default());
        let controls = PlaybackControls::builder()
            .platform(Arc::new(Box::new(platform)))
            .event_publisher(event_publisher.clone())
            .build();

        // add a callback to the playback control events
        controls.register(Box::new(move |e| tx_ce.send(e).unwrap()));

        // invoke the callback on the platform
        let callback = rx.recv_timeout(Duration::from_millis(100)).unwrap();
        callback(PlatformEvent::TogglePlaybackState);

        let result = rx_ce.recv_timeout(Duration::from_millis(100)).unwrap();
        match result {
            PlaybackControlEvent::TogglePlaybackState => {}
            _ => panic!("Expected PlaybackControlEvent::TogglePlaybackState"),
        }
    }

    #[test]
    fn test_platform_event_forward() {
        init_logger();
        let (tx, rx) = channel();
        let (tx_ce, rx_ce) = channel();
        let mut platform = MockDummyPlatformData::new();
        platform
            .expect_register()
            .returning(move |callback| tx.send(callback).unwrap());
        let event_publisher = Arc::new(EventPublisher::default());
        let controls = PlaybackControls::builder()
            .platform(Arc::new(Box::new(platform)))
            .event_publisher(event_publisher.clone())
            .build();

        // add a callback to the playback control events
        controls.register(Box::new(move |e| tx_ce.send(e).unwrap()));

        // invoke the callback on the platform
        let callback = rx.recv_timeout(Duration::from_millis(100)).unwrap();
        callback(PlatformEvent::ForwardMedia);

        let result = rx_ce.recv_timeout(Duration::from_millis(100)).unwrap();
        match result {
            PlaybackControlEvent::Forward => {}
            _ => panic!("Expected PlaybackControlEvent::Forward"),
        }
    }

    #[test]
    fn test_on_player_started_event() {
        init_logger();
        let (tx, rx) = channel();
        let mut platform = MockDummyPlatformData::new();
        platform.expect_register().returning(|_| {});
        platform
            .expect_notify_media_event()
            .returning(move |notification: MediaNotificationEvent| tx.send(notification).unwrap());
        let event_publisher = Arc::new(EventPublisher::default());
        let _controls = PlaybackControls::builder()
            .platform(Arc::new(Box::new(platform)))
            .event_publisher(event_publisher.clone())
            .build();

        event_publisher.publish(Event::PlayerStarted(PlayerStartedEvent {
            url: "https://my-url".to_string(),
            title: "Lorem ipsum".to_string(),
            thumbnail: Some("MyThumb".to_string()),
            background: None,
            quality: Some("My showname".to_string()),
            auto_resume_timestamp: None,
            subtitles_enabled: false,
        }));

        let result = rx.recv_timeout(Duration::from_millis(100)).unwrap();
        match result {
            MediaNotificationEvent::StateStarting(info) => assert_eq!(
                info,
                MediaInfo {
                    title: "Lorem ipsum".to_string(),
                    subtitle: Some("My showname".to_string()),
                    thumb: Some("MyThumb".to_string()),
                }
            ),
            _ => panic!("Expected MediaNotificationEvent::PlaybackStarted"),
        }
    }

    #[test]
    fn test_on_player_stopped_event() {
        init_logger();
        let (tx, rx) = channel();
        let mut platform = MockDummyPlatformData::new();
        platform.expect_register().returning(|_| {});
        platform
            .expect_notify_media_event()
            .returning(move |notification: MediaNotificationEvent| tx.send(notification).unwrap());
        let event_publisher = Arc::new(EventPublisher::default());
        let _controls = PlaybackControls::builder()
            .platform(Arc::new(Box::new(platform)))
            .event_publisher(event_publisher.clone())
            .build();

        event_publisher.publish(Event::PlayerStopped(PlayerStoppedEvent {
            url: "http://localhost/my-video.mp4".to_string(),
            media: None,
            time: Some(10000),
            duration: Some(50000),
        }));

        let result = rx.recv_timeout(Duration::from_millis(100)).unwrap();
        assert_eq!(MediaNotificationEvent::StateStopped, result);
    }

    #[test]
    fn test_on_playback_state_changed_event() {
        init_logger();
        let (tx, rx) = channel();
        let mut platform = MockDummyPlatformData::new();
        platform.expect_register().returning(|_| {});
        platform
            .expect_notify_media_event()
            .returning(move |notification: MediaNotificationEvent| tx.send(notification).unwrap());
        let event_publisher = Arc::new(EventPublisher::default());
        let _controls = PlaybackControls::builder()
            .platform(Arc::new(Box::new(platform)))
            .event_publisher(event_publisher.clone())
            .build();

        event_publisher.publish(Event::PlaybackStateChanged(PlaybackState::PLAYING));
        let result = rx.recv_timeout(Duration::from_millis(100)).unwrap();
        assert_eq!(MediaNotificationEvent::StatePlaying, result);

        event_publisher.publish(Event::PlaybackStateChanged(PlaybackState::PAUSED));
        let result = rx.recv_timeout(Duration::from_millis(100)).unwrap();
        assert_eq!(MediaNotificationEvent::StatePaused, result);
    }
}
