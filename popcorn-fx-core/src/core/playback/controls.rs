use crate::core::event::{
    Event, EventCallback, EventHandler, EventPublisher, PlayerStartedEvent, DEFAULT_ORDER,
};
use crate::core::platform::{PlatformData, PlatformEvent};
use crate::core::playback::{
    MediaInfo, MediaNotificationEvent, PlaybackControlEvent, PlaybackState,
};
use fx_callback::{Callback, MultiThreadedCallback, Subscriber, Subscription};
use log::{debug, trace, warn};
use std::sync::Arc;
use tokio::select;
use tokio_util::sync::CancellationToken;

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
///     .event_publisher(EventPublisher::new())
///     .build();
/// ```
#[derive(Debug, Clone)]
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
    ///     .event_publisher(EventPublisher::new())
    ///     .build();
    /// ```
    pub fn builder() -> PlaybackControlsBuilder {
        PlaybackControlsBuilder::default()
    }
}

impl Callback<PlaybackControlEvent> for PlaybackControls {
    fn subscribe(&self) -> Subscription<PlaybackControlEvent> {
        self.inner.callbacks.subscribe()
    }

    fn subscribe_with(&self, subscriber: Subscriber<PlaybackControlEvent>) {
        self.inner.callbacks.subscribe_with(subscriber);
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
///     .event_publisher(EventPublisher::new())
///     .build();
/// ```
#[derive(Default)]
pub struct PlaybackControlsBuilder {
    platform: Option<Arc<Box<dyn PlatformData>>>,
    event_publisher: Option<EventPublisher>,
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
    pub fn event_publisher(mut self, event_publisher: EventPublisher) -> Self {
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
                callbacks: MultiThreadedCallback::new(),
                cancellation_token: Default::default(),
            }),
        };

        let inner = instance.inner.clone();
        let mut receiver = instance.inner.platform.subscribe();
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                inner.handle_platform_event(event);
            }
        });

        let inner = instance.inner.clone();
        if let Some(event_publisher) = self.event_publisher {
            let callback = event_publisher
                .subscribe(DEFAULT_ORDER)
                .expect("expected to receive a callback");
            tokio::spawn(async move {
                inner.start(callback).await;
            });
        } else {
            warn!("Unable to handle control events for PlaybackControls, EventPublisher has not been set");
        }

        instance
    }
}

#[derive(Debug)]
struct InnerPlaybackControls {
    platform: Arc<Box<dyn PlatformData>>,
    callbacks: MultiThreadedCallback<PlaybackControlEvent>,
    cancellation_token: CancellationToken,
}

impl InnerPlaybackControls {
    async fn start(&self, mut event_receiver: EventCallback) {
        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(handler) = event_receiver.recv() => self.handle_event(handler).await,
            }
        }
        debug!("Playback controls main loop ended");
    }

    async fn handle_event(&self, mut handler: EventHandler) {
        if let Some(event) = handler.event_ref() {
            match event {
                Event::PlayerStarted(play_event) => {
                    self.notify_media_playback(play_event.clone());
                }
                Event::PlaybackStateChanged(new_state) => {
                    self.notify_media_state_changed(new_state.clone())
                }
                Event::PlayerStopped(_) => self.notify_media_stopped(),
                _ => {}
            }
        }
        handler.next();
    }

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

    fn handle_platform_event(&self, event: Arc<PlatformEvent>) {
        trace!("Handling platform event {:?}", event);
        match &*event {
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
    use super::*;
    use crate::core::event::PlayerStoppedEvent;
    use crate::testing::MockDummyPlatformData;
    use crate::{init_logger, recv_timeout};
    use std::sync::mpsc::channel;
    use std::time::Duration;
    use tokio::sync::mpsc::unbounded_channel;
    use tokio::sync::oneshot;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_platform_event_toggle_playback() {
        init_logger!();
        let (tx, rx) = unbounded_channel();
        let mut platform = MockDummyPlatformData::new();
        platform.expect_subscribe().return_once(move || rx);
        let event_publisher = EventPublisher::default();
        let controls = PlaybackControls::builder()
            .platform(Arc::new(Box::new(platform)))
            .event_publisher(event_publisher.clone())
            .build();

        // invoke the callback on the platform
        let mut receiver = controls.subscribe();
        tx.send(Arc::new(PlatformEvent::TogglePlaybackState))
            .unwrap();

        let result = recv_timeout!(&mut receiver, Duration::from_millis(500));
        match &*result {
            PlaybackControlEvent::TogglePlaybackState => {}
            _ => assert!(
                false,
                "expected PlaybackControlEvent::TogglePlaybackState, but got {:?}",
                result
            ),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_platform_event_forward() {
        init_logger!();
        let (tx, rx) = unbounded_channel();
        let mut platform = MockDummyPlatformData::new();
        platform.expect_subscribe().return_once(move || rx);
        let event_publisher = EventPublisher::default();
        let controls = PlaybackControls::builder()
            .platform(Arc::new(Box::new(platform)))
            .event_publisher(event_publisher.clone())
            .build();

        // invoke the callback on the platform
        let mut receiver = controls.subscribe();
        tx.send(Arc::new(PlatformEvent::ForwardMedia)).unwrap();

        let result = recv_timeout!(&mut receiver, Duration::from_millis(500));
        match &*result {
            PlaybackControlEvent::Forward => {}
            _ => assert!(
                false,
                "expected PlaybackControlEvent::Forward, but got {:?}",
                result
            ),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_on_player_started_event() {
        init_logger!();
        let (tx, rx) = oneshot::channel();
        let mut platform = MockDummyPlatformData::new();
        platform.expect_notify_media_event().return_once(
            move |notification: MediaNotificationEvent| tx.send(notification).unwrap(),
        );
        platform.expect_subscribe().returning(|| {
            let (_, rx) = unbounded_channel();
            rx
        });
        let event_publisher = EventPublisher::default();
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

        let result = timeout(Duration::from_millis(500), rx)
            .await
            .expect("timed-out waiting for notification event")
            .unwrap();
        match result {
            MediaNotificationEvent::StateStarting(info) => assert_eq!(
                info,
                MediaInfo {
                    title: "Lorem ipsum".to_string(),
                    subtitle: Some("My showname".to_string()),
                    thumb: Some("MyThumb".to_string()),
                }
            ),
            _ => assert!(
                false,
                "Expected MediaNotificationEvent::PlaybackStarted, but got {:?}",
                result
            ),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_on_player_stopped_event() {
        init_logger!();
        let (tx, rx) = channel();
        let mut platform = MockDummyPlatformData::new();
        platform
            .expect_notify_media_event()
            .returning(move |notification: MediaNotificationEvent| tx.send(notification).unwrap());
        platform.expect_subscribe().returning(|| {
            let (_, rx) = unbounded_channel();
            rx
        });
        let event_publisher = EventPublisher::default();
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_on_playback_state_changed_event() {
        init_logger!();
        let (tx, rx) = channel();
        let mut platform = MockDummyPlatformData::new();
        platform
            .expect_notify_media_event()
            .returning(move |notification: MediaNotificationEvent| tx.send(notification).unwrap());
        platform.expect_subscribe().returning(|| {
            let (_, rx) = unbounded_channel();
            rx
        });
        let event_publisher = EventPublisher::default();
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
