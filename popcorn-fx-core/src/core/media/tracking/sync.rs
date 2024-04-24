use std::result;
use std::sync::Arc;

use derive_more::Display;
use log::{debug, error, info, trace};
use thiserror::Error;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use crate::core::{block_in_place, CallbackHandle};
use crate::core::config::{ApplicationConfig, MediaTrackingSyncState};
use crate::core::media::tracking::{TrackingError, TrackingEvent, TrackingProvider};
use crate::core::media::watched::WatchedService;

/// Represents the state of synchronization.
#[derive(Debug, Display, Clone, PartialEq)]
pub enum SyncState {
    #[display(fmt = "idle")]
    Idle,
    #[display(fmt = "syncing")]
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
    /// Optional callback handle.
    callback_handle: Option<CallbackHandle>,
}

impl SyncMediaTracking {
    pub fn builder() -> SyncMediaTrackingBuilder {
        SyncMediaTrackingBuilder::builder()
    }

    pub fn new(
        config: Arc<ApplicationConfig>,
        provider: Arc<Box<dyn TrackingProvider>>,
        watched_service: Arc<Box<dyn WatchedService>>,
        runtime: Arc<Runtime>,
    ) -> Self {
        let mut instance = Self {
            inner: Arc::new(InnerSyncMediaTracking {
                config,
                provider,
                watched_service,
                state: Mutex::new(SyncState::Idle),
                runtime,
            }),
            callback_handle: None,
        };

        let event_instance = instance.inner.clone();
        let event_runtime = instance.inner.runtime.clone();
        instance.callback_handle = Some(instance.inner.provider.add(Box::new(move |event| {
            if let TrackingEvent::AuthorizationStateChanged(state) = event {
                trace!("Received authorization state changed to {}", state);
                if state {
                    let runtime_instance = event_instance.clone();
                    event_runtime.spawn(async move {
                        Self::handle_sync_result(runtime_instance.sync().await)
                    });
                }
            }
        })));
        let auto_sync_instance = instance.inner.clone();
        instance.inner.runtime.spawn(async move {
            if auto_sync_instance.provider.is_authorized() {
                debug!("Tracking provider has been authorized, starting automatic startup synchronization");
                Self::handle_sync_result(auto_sync_instance.sync().await)
            }
        });

        instance
    }

    pub fn state(&self) -> SyncState {
        self.inner.state()
    }

    pub fn start_sync(&self) {
        let inner = self.inner.clone();
        self.inner
            .runtime
            .spawn(async move { Self::handle_sync_result(inner.sync().await) });
    }

    pub async fn sync(&self) -> Result<()> {
        self.inner.sync().await
    }

    fn handle_sync_result(result: Result<()>) {
        match result {
            Ok(_) => info!("Tracking synchronization completed"),
            Err(e) => error!("Tracking synchronization failed, {}", e),
        }
    }
}

impl Drop for SyncMediaTracking {
    fn drop(&mut self) {
        trace!("Dropping {:?}", self);
        if let Some(handle) = self.callback_handle {
            debug!("Removing tracking provider callback handle {}", handle);
            self.inner.provider.remove(handle);
        }
    }
}

/// Builder for constructing `SyncMediaTracking` instances.
#[derive(Debug, Default)]
pub struct SyncMediaTrackingBuilder {
    config: Option<Arc<ApplicationConfig>>,
    provider: Option<Arc<Box<dyn TrackingProvider>>>,
    watched_service: Option<Arc<Box<dyn WatchedService>>>,
    runtime: Option<Arc<Runtime>>,
}

impl SyncMediaTrackingBuilder {
    /// Creates a new `SyncMediaTrackingBuilder`.
    pub fn builder() -> Self {
        Self::default()
    }

    /// Set the application config for the builder.
    pub fn config(mut self, config: Arc<ApplicationConfig>) -> Self {
        self.config = Some(config);
        self
    }

    /// Sets the tracking provider for the builder.
    pub fn tracking_provider(mut self, tracking_provider: Arc<Box<dyn TrackingProvider>>) -> Self {
        self.provider = Some(tracking_provider);
        self
    }

    /// Sets the watched service for the builder.
    pub fn watched_service(mut self, watched_service: Arc<Box<dyn WatchedService>>) -> Self {
        self.watched_service = Some(watched_service);
        self
    }

    /// Sets the runtime for the builder.
    pub fn runtime(mut self, runtime: Arc<Runtime>) -> Self {
        self.runtime = Some(runtime);
        self
    }

    /// Builds the `SyncMediaTracking` instance.
    pub fn build(self) -> SyncMediaTracking {
        let runtime = self.runtime.unwrap_or_else(|| {
            Arc::new(
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .worker_threads(1)
                    .thread_name("tracking")
                    .build()
                    .expect("expected a new runtime"),
            )
        });

        SyncMediaTracking::new(
            self.config.expect("expected the config to have been set"),
            self.provider
                .expect("expected the tracking provider to have been set"),
            self.watched_service
                .expect("expected the watched service to have been set"),
            runtime,
        )
    }
}

#[derive(Debug)]
struct InnerSyncMediaTracking {
    config: Arc<ApplicationConfig>,
    provider: Arc<Box<dyn TrackingProvider>>,
    watched_service: Arc<Box<dyn WatchedService>>,
    state: Mutex<SyncState>,
    runtime: Arc<Runtime>,
}

impl InnerSyncMediaTracking {
    fn state(&self) -> SyncState {
        let mutex = block_in_place(self.state.lock());
        mutex.clone()
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
            .user_settings_ref()
            .tracking_mut()
            .update_state(MediaTrackingSyncState::Success);
        self.config.save_async().await;
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
            .user_settings_ref()
            .tracking_mut()
            .update_state(MediaTrackingSyncState::Failed);
        self.config.save_async().await;
        Err(SyncError::Failed(err.to_string()))
    }

    async fn update_state_to_idle(&self) {
        let mut mutex = self.state.lock().await;
        *mutex = SyncState::Idle;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use mockall::predicate;

    use crate::assert_timeout_eq;
    use crate::core::Handle;
    use crate::core::media::{MediaIdentifier, MockMediaIdentifier};
    use crate::core::media::tracking::MockTrackingProvider;
    use crate::core::media::watched::MockWatchedService;
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_new_is_authorized() {
        init_logger();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let config = Arc::new(ApplicationConfig::builder().storage(temp_path).build());
        let mut provider = MockTrackingProvider::new();
        provider.expect_is_authorized().times(1).return_const(true);
        provider.expect_add().times(1).return_const(Handle::new());
        provider.expect_remove().times(1).return_const(());
        provider.expect_add_watched_movies().return_const(Ok(()));
        provider
            .expect_watched_movies()
            .times(1)
            .returning(move || {
                let mut movie = MockMediaIdentifier::new();
                movie.expect_imdb_id().return_const("tt000123".to_string());
                Ok(vec![Box::new(movie)])
            });
        let mut watched_service = MockWatchedService::new();
        watched_service
            .expect_watched_movies()
            .return_const(Ok(vec![]));
        watched_service.expect_add().return_const(Ok(()));
        let sync = SyncMediaTracking::builder()
            .config(config)
            .tracking_provider(Arc::new(Box::new(provider)))
            .watched_service(Arc::new(Box::new(watched_service)))
            .build();

        assert_timeout_eq!(
            Duration::from_millis(200),
            true,
            sync.inner
                .config
                .user_settings()
                .tracking()
                .last_sync()
                .is_some()
        );

        let settings = sync.inner.config.user_settings();
        let result = settings.tracking().last_sync().unwrap();
        assert_eq!(&MediaTrackingSyncState::Success, &result.state);
    }

    #[test]
    fn test_drop() {
        init_logger();
        let handle = Handle::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let config = Arc::new(ApplicationConfig::builder().storage(temp_path).build());
        let watched_service = MockWatchedService::new();
        let mut provider = MockTrackingProvider::new();
        provider.expect_add().times(1).return_const(handle.clone());
        provider
            .expect_remove()
            .times(1)
            .with(predicate::eq(handle))
            .return_const(());

        let sync = SyncMediaTracking::builder()
            .config(config)
            .tracking_provider(Arc::new(Box::new(provider)))
            .watched_service(Arc::new(Box::new(watched_service)))
            .build();

        drop(sync);

        // expect_remove will panic if it has not been invoked with the handle
    }

    #[test]
    fn test_start_sync() {
        init_logger();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let config = Arc::new(ApplicationConfig::builder().storage(temp_path).build());
        let mut provider = MockTrackingProvider::new();
        provider.expect_is_authorized().return_const(false);
        provider.expect_add().return_const(Handle::new());
        provider.expect_remove().return_const(());
        provider.expect_add_watched_movies().return_const(Ok(()));
        provider
            .expect_watched_movies()
            .returning(|| Ok(Vec::<Box<dyn MediaIdentifier>>::new()));
        let mut watched_service = MockWatchedService::new();
        watched_service
            .expect_watched_movies()
            .return_const(Ok(vec![]));
        let sync = SyncMediaTracking::builder()
            .config(config)
            .tracking_provider(Arc::new(Box::new(provider)))
            .watched_service(Arc::new(Box::new(watched_service)))
            .build();

        sync.start_sync();
        assert_timeout_eq!(
            Duration::from_millis(200),
            true,
            sync.inner
                .config
                .user_settings()
                .tracking()
                .last_sync()
                .is_some()
        );

        let settings = sync.inner.config.user_settings();
        let result = settings.tracking().last_sync().unwrap();
        assert_eq!(MediaTrackingSyncState::Success, result.state);
    }

    #[test]
    fn test_sync_watched_movies_error() {
        init_logger();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let config = Arc::new(ApplicationConfig::builder().storage(temp_path).build());
        let mut provider = MockTrackingProvider::new();
        provider.expect_is_authorized().return_const(false);
        provider.expect_add().return_const(Handle::new());
        provider.expect_remove().return_const(());
        provider.expect_add_watched_movies().return_const(Ok(()));
        provider
            .expect_watched_movies()
            .returning(|| Err(TrackingError::Request));
        let mut watched_service = MockWatchedService::new();
        watched_service
            .expect_watched_movies()
            .return_const(Ok(vec![]));
        let sync = SyncMediaTracking::builder()
            .config(config)
            .tracking_provider(Arc::new(Box::new(provider)))
            .watched_service(Arc::new(Box::new(watched_service)))
            .build();

        sync.start_sync();
        assert_timeout_eq!(
            Duration::from_millis(200),
            true,
            sync.inner
                .config
                .user_settings()
                .tracking()
                .last_sync()
                .is_some()
        );

        let settings = sync.inner.config.user_settings();
        let result = settings.tracking().last_sync().unwrap();
        assert_eq!(MediaTrackingSyncState::Failed, result.state);
    }

    #[test]
    fn test_authorization_state_changed() {
        init_logger();
        let (tx, rx) = channel();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let config = Arc::new(ApplicationConfig::builder().storage(temp_path).build());
        let mut provider = MockTrackingProvider::new();
        provider.expect_is_authorized().return_const(false);
        provider.expect_add().returning(move |e| {
            tx.send(e).unwrap();
            Handle::new()
        });
        provider.expect_remove().return_const(());
        provider.expect_add_watched_movies().return_const(Ok(()));
        provider
            .expect_watched_movies()
            .returning(|| Ok(Vec::<Box<dyn MediaIdentifier>>::new()));
        let mut watched_service = MockWatchedService::new();
        watched_service
            .expect_watched_movies()
            .return_const(Ok(vec![]));
        let sync = SyncMediaTracking::builder()
            .config(config)
            .tracking_provider(Arc::new(Box::new(provider)))
            .watched_service(Arc::new(Box::new(watched_service)))
            .build();

        let callback = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        callback(TrackingEvent::AuthorizationStateChanged(true));

        assert_timeout_eq!(
            Duration::from_millis(200),
            true,
            sync.inner
                .config
                .user_settings()
                .tracking()
                .last_sync()
                .is_some()
        );

        let settings = sync.inner.config.user_settings();
        let result = settings.tracking().last_sync().unwrap();
        assert_eq!(MediaTrackingSyncState::Success, result.state);
    }
}
