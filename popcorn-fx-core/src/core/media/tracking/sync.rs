use crate::core::config::{ApplicationConfig, MediaTrackingSyncState};
use crate::core::media::tracking::{TrackingError, TrackingEvent, TrackingProvider};
use crate::core::media::watched::WatchedService;
use derive_more::Display;
use fx_callback::Subscription;
use log::{debug, error, info, trace};
use std::result;
use std::sync::Arc;
use thiserror::Error;
use tokio::select;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

/// Represents the state of synchronization.
#[derive(Debug, Display, Clone, PartialEq)]
pub enum SyncState {
    #[display("idle")]
    Idle,
    #[display("syncing")]
    Syncing,
}

/// Represents errors that can occur during synchronization.
#[derive(Debug, Clone, Error)]
pub enum SyncError {
    /// The synchronization is in an invalid state.
    #[error("sync is in invalid state {0}")]
    InvalidState(SyncState),
    /// The media tracker has not been authorized.
    #[error("media tracker has not been authorized")]
    MediaTrackerNotAuthorized,
    /// Failed to synchronize the media tracker.
    #[error("failed to sync media tracker, {0}")]
    Failed(String),
}

/// Alias for `Result` with `SyncError`.
pub type Result<T> = result::Result<T, SyncError>;

/// Represents synchronized media tracking.
#[derive(Debug)]
pub struct SyncMediaTracking {
    /// The inner actual synchronizer.
    inner: Arc<InnerSyncMediaTracking>,
}

impl SyncMediaTracking {
    pub fn builder() -> SyncMediaTrackingBuilder {
        SyncMediaTrackingBuilder::builder()
    }

    pub fn new(
        config: ApplicationConfig,
        provider: Arc<dyn TrackingProvider>,
        watched_service: Arc<dyn WatchedService>,
    ) -> Self {
        let (command_sender, command_receiver) = unbounded_channel();
        let inner = Arc::new(InnerSyncMediaTracking {
            config,
            provider,
            watched_service,
            state: Mutex::new(SyncState::Idle),
            command_sender,
            cancellation_token: Default::default(),
        });

        let event_receiver = inner.provider.subscribe();
        let inner_main = inner.clone();
        tokio::spawn(async move {
            inner_main.start(event_receiver, command_receiver).await;
        });

        Self { inner }
    }

    pub async fn state(&self) -> SyncState {
        self.inner.state().await
    }

    pub fn start_sync(&self) {
        self.inner.send_command(SyncMediaTrackingCommand::SyncAll);
    }

    pub async fn sync(&self) -> Result<()> {
        self.inner.sync().await
    }
}

impl Drop for SyncMediaTracking {
    fn drop(&mut self) {
        trace!("Dropping {:?}", self);
        self.inner.cancellation_token.cancel();
    }
}

/// Builder for constructing `SyncMediaTracking` instances.
#[derive(Debug, Default)]
pub struct SyncMediaTrackingBuilder {
    config: Option<ApplicationConfig>,
    provider: Option<Arc<dyn TrackingProvider>>,
    watched_service: Option<Arc<dyn WatchedService>>,
}

impl SyncMediaTrackingBuilder {
    /// Creates a new `SyncMediaTrackingBuilder`.
    pub fn builder() -> Self {
        Self::default()
    }

    /// Set the application config for the builder.
    pub fn config(mut self, config: ApplicationConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Sets the tracking provider for the builder.
    pub fn tracking_provider(mut self, tracking_provider: Arc<dyn TrackingProvider>) -> Self {
        self.provider = Some(tracking_provider);
        self
    }

    /// Sets the watched service for the builder.
    pub fn watched_service(mut self, watched_service: Arc<dyn WatchedService>) -> Self {
        self.watched_service = Some(watched_service);
        self
    }

    /// Builds the `SyncMediaTracking` instance.
    pub fn build(self) -> SyncMediaTracking {
        SyncMediaTracking::new(
            self.config.expect("expected the config to have been set"),
            self.provider
                .expect("expected the tracking provider to have been set"),
            self.watched_service
                .expect("expected the watched service to have been set"),
        )
    }
}

#[derive(Debug, PartialEq)]
enum SyncMediaTrackingCommand {
    SyncAll,
    SyncMovies,
}

#[derive(Debug)]
struct InnerSyncMediaTracking {
    config: ApplicationConfig,
    provider: Arc<dyn TrackingProvider>,
    watched_service: Arc<dyn WatchedService>,
    state: Mutex<SyncState>,
    command_sender: UnboundedSender<SyncMediaTrackingCommand>,
    cancellation_token: CancellationToken,
}

impl InnerSyncMediaTracking {
    async fn start(
        &self,
        mut tracking_receiver: Subscription<TrackingEvent>,
        mut command_receiver: UnboundedReceiver<SyncMediaTrackingCommand>,
    ) {
        if self.provider.is_authorized().await {
            debug!(
                "Tracking provider has been authorized, starting automatic startup synchronization"
            );
            Self::handle_sync_result(self.sync().await)
        }

        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(event) = tracking_receiver.recv() => self.handle_event(&*event).await,
                Some(command) = command_receiver.recv() => self.handle_command(command).await
            }
        }

        debug!("Sync media tracking main loop ended");
    }

    async fn handle_event(&self, event: &TrackingEvent) {
        if let TrackingEvent::AuthorizationStateChanged(state) = event {
            trace!("Received authorization state changed to {}", state);
            if *state {
                Self::handle_sync_result(self.sync().await)
            }
        }
    }

    async fn handle_command(&self, command: SyncMediaTrackingCommand) {
        match command {
            SyncMediaTrackingCommand::SyncAll => Self::handle_sync_result(self.sync().await),
            SyncMediaTrackingCommand::SyncMovies => {
                Self::handle_sync_result(self.sync_movies().await)
            }
        }
    }

    async fn state(&self) -> SyncState {
        (*self.state.lock().await).clone()
    }

    async fn sync(&self) -> Result<()> {
        trace!("Syncing media tracking data");
        let state: SyncState;
        {
            let mutex = self.state.lock().await;
            state = mutex.clone();
        }

        if state != SyncState::Idle {
            debug!(
                "Unable to start tracking synchronization, synchronizer is in invalid state {}",
                state
            );
            return Err(SyncError::InvalidState(state));
        }

        {
            let mut mutex = self.state.lock().await;
            *mutex = SyncState::Syncing;
        }

        self.sync_movies().await?;

        info!("Media tracker has been synchronized");
        self.config
            .update_tracker_state(MediaTrackingSyncState::Success)
            .await;
        self.update_state_to_idle().await;
        Ok(())
    }

    async fn sync_movies(&self) -> Result<()> {
        trace!("Retrieving locally watched movies");
        match self.watched_service.watched_movies() {
            Ok(watched_movies) => {
                trace!("Syncing movies from tracker");
                match self.provider.watched_movies().await {
                    Ok(tracker_movies) => {
                        let mut synced_items = 0;
                        for movie in tracker_movies {
                            if !watched_movies.contains(&movie.imdb_id().to_string()) {
                                if let Err(e) = self.watched_service.add(movie) {
                                    error!("Failed to add watched movie, {}", e);
                                } else {
                                    synced_items += 1;
                                }
                            }
                        }
                        debug!("Synced a total of {} movies to local DB", synced_items);
                    }
                    Err(e) => self.handle_error(e).await?,
                }

                trace!("Syncing movies to tracker");
                match self.watched_service.watched_movies() {
                    Ok(movies) => match self.provider.add_watched_movies(movies).await {
                        Ok(_) => debug!("Remote tracker has been updated with watched movies"),
                        Err(e) => self.handle_error(e).await?,
                    },
                    Err(e) => error!("Failed to retrieve watched movies, {}", e),
                }
            }
            Err(e) => {
                error!("Unable to sync movies, {}", e);
            }
        }
        Ok(())
    }

    async fn handle_error(&self, err: TrackingError) -> Result<()> {
        error!("Failed to synchronize tracking data, {}", err);
        self.update_state_to_idle().await;
        self.config
            .update_tracker_state(MediaTrackingSyncState::Failed)
            .await;
        Err(SyncError::Failed(err.to_string()))
    }

    async fn update_state_to_idle(&self) {
        let mut mutex = self.state.lock().await;
        *mutex = SyncState::Idle;
    }

    fn send_command(&self, command: SyncMediaTrackingCommand) {
        if let Err(e) = self.command_sender.send(command) {
            debug!("Sync media tracking failed to send command, {}", e);
        }
    }

    fn handle_sync_result(result: Result<()>) {
        match result {
            Ok(_) => info!("Tracking synchronization completed"),
            Err(e) => error!("Tracking synchronization failed, {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::core::media::tracking::MockTrackingProvider;
    use crate::core::media::watched::test::MockWatchedService;
    use crate::core::media::{MediaIdentifier, MockMediaIdentifier};
    use crate::{assert_timeout_eq, init_logger};

    use fx_callback::{Callback, MultiThreadedCallback};
    use std::time::Duration;

    #[tokio::test]
    async fn test_new_is_authorized() {
        init_logger!();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let config = ApplicationConfig::builder().storage(temp_path).build();
        let callbacks = MultiThreadedCallback::new();
        let callback_receiver = callbacks.subscribe();
        let mut provider = MockTrackingProvider::new();
        provider.expect_is_authorized().times(1).return_const(true);
        provider.expect_add_watched_movies().return_const(Ok(()));
        provider
            .expect_watched_movies()
            .times(1)
            .returning(move || {
                let mut movie = MockMediaIdentifier::new();
                movie.expect_imdb_id().return_const("tt000123".to_string());
                Ok(vec![Box::new(movie)])
            });
        provider
            .expect_subscribe()
            .times(1)
            .return_once(move || callback_receiver);
        let mut watched_service = MockWatchedService::new();
        watched_service
            .expect_watched_movies()
            .return_const(Ok(vec![]));
        watched_service.expect_add().return_const(Ok(()));
        let sync = SyncMediaTracking::builder()
            .config(config)
            .tracking_provider(Arc::new(provider))
            .watched_service(Arc::new(watched_service))
            .build();

        assert_timeout_eq!(
            Duration::from_millis(200),
            true,
            sync.inner
                .config
                .user_settings_ref(|e| e.tracking().last_sync().is_some())
                .await
        );

        let settings = sync.inner.config.user_settings().await;
        let result = settings.tracking().last_sync().unwrap();
        assert_eq!(&MediaTrackingSyncState::Success, &result.state);
    }

    #[tokio::test]
    async fn test_drop() {
        init_logger!();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let config = ApplicationConfig::builder().storage(temp_path).build();
        let watched_service = MockWatchedService::new();
        let callbacks = MultiThreadedCallback::new();
        let callback_receiver = callbacks.subscribe();
        let mut provider = MockTrackingProvider::new();
        provider
            .expect_subscribe()
            .times(1)
            .return_once(move || callback_receiver);

        let sync = SyncMediaTracking::builder()
            .config(config)
            .tracking_provider(Arc::new(provider))
            .watched_service(Arc::new(watched_service))
            .build();

        drop(sync);

        // expect_remove will panic if it has not been invoked with the handle
    }

    #[tokio::test]
    async fn test_start_sync() {
        init_logger!();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let config = ApplicationConfig::builder().storage(temp_path).build();
        let callbacks = MultiThreadedCallback::new();
        let callback_receiver = callbacks.subscribe();
        let mut provider = MockTrackingProvider::new();
        provider.expect_is_authorized().return_const(false);
        provider.expect_add_watched_movies().return_const(Ok(()));
        provider
            .expect_watched_movies()
            .returning(|| Ok(Vec::<Box<dyn MediaIdentifier>>::new()));
        provider
            .expect_subscribe()
            .times(1)
            .return_once(move || callback_receiver);
        let mut watched_service = MockWatchedService::new();
        watched_service
            .expect_watched_movies()
            .return_const(Ok(vec![]));
        let sync = SyncMediaTracking::builder()
            .config(config)
            .tracking_provider(Arc::new(provider))
            .watched_service(Arc::new(watched_service))
            .build();

        sync.start_sync();
        assert_timeout_eq!(
            Duration::from_millis(200),
            true,
            sync.inner
                .config
                .user_settings_ref(|e| e.tracking().last_sync().is_some())
                .await
        );

        let settings = sync.inner.config.user_settings().await;
        let result = settings.tracking().last_sync().unwrap();
        assert_eq!(MediaTrackingSyncState::Success, result.state);
    }

    #[tokio::test]
    async fn test_sync_watched_movies_error() {
        init_logger!();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let config = ApplicationConfig::builder().storage(temp_path).build();
        let callbacks = MultiThreadedCallback::new();
        let callback_receiver = callbacks.subscribe();
        let mut provider = MockTrackingProvider::new();
        provider.expect_is_authorized().return_const(false);
        provider.expect_add_watched_movies().return_const(Ok(()));
        provider
            .expect_watched_movies()
            .returning(|| Err(TrackingError::Request));
        provider
            .expect_subscribe()
            .times(1)
            .return_once(move || callback_receiver);
        let mut watched_service = MockWatchedService::new();
        watched_service
            .expect_watched_movies()
            .return_const(Ok(vec![]));
        let sync = SyncMediaTracking::builder()
            .config(config)
            .tracking_provider(Arc::new(provider))
            .watched_service(Arc::new(watched_service))
            .build();

        sync.start_sync();
        assert_timeout_eq!(
            Duration::from_millis(200),
            true,
            sync.inner
                .config
                .user_settings_ref(|e| e.tracking().last_sync().is_some())
                .await
        );

        let settings = sync.inner.config.user_settings().await;
        let result = settings.tracking().last_sync().unwrap();
        assert_eq!(MediaTrackingSyncState::Failed, result.state);
    }

    #[tokio::test]
    async fn test_authorization_state_changed() {
        init_logger!();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let config = ApplicationConfig::builder().storage(temp_path).build();
        let callbacks = MultiThreadedCallback::<TrackingEvent>::new();
        let callback_receiver = callbacks.subscribe();
        let mut provider = MockTrackingProvider::new();
        provider.expect_is_authorized().return_const(false);
        provider.expect_add_watched_movies().return_const(Ok(()));
        provider
            .expect_watched_movies()
            .returning(|| Ok(Vec::<Box<dyn MediaIdentifier>>::new()));
        provider
            .expect_subscribe()
            .times(1)
            .return_once(move || callback_receiver);
        let mut watched_service = MockWatchedService::new();
        watched_service
            .expect_watched_movies()
            .return_const(Ok(vec![]));
        let sync = SyncMediaTracking::builder()
            .config(config)
            .tracking_provider(Arc::new(provider))
            .watched_service(Arc::new(watched_service))
            .build();

        callbacks.invoke(TrackingEvent::AuthorizationStateChanged(true));

        assert_timeout_eq!(
            Duration::from_millis(200),
            true,
            sync.inner
                .config
                .user_settings_ref(|e| e.tracking().last_sync().is_some())
                .await
        );

        let result = sync
            .inner
            .config
            .user_settings_ref(|e| e.tracking().last_sync().unwrap().clone())
            .await;
        assert_eq!(MediaTrackingSyncState::Success, result.state);
    }
}
