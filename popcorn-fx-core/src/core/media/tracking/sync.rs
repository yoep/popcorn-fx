use std::result;
use std::sync::Arc;

use derive_more::Display;
use log::{debug, error, info, trace};
use thiserror::Error;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use crate::core::{block_in_place, CallbackHandle};
use crate::core::config::{ApplicationConfig, TrackerSyncState};
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

#[derive(Debug)]
pub struct SyncMediaTracking {
    inner: Arc<InnerSyncMediaTracking>,
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

        instance
    }

    pub fn state(&self) -> SyncState {
        self.inner.state()
    }

    pub fn start_sync(&self) {
        let inner = self.inner.clone();
        self.inner.runtime.spawn(async move {
            Self::handle_sync_result(inner.sync().await)
        });
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
            Arc::new(tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(1)
                .thread_name("tracking")
                .build()
                .expect("expected a new runtime"))
        });

        SyncMediaTracking::new(
            self.config.expect("expected the config to have been set"),
            self.provider.expect("expected the tracking provider to have been set"),
            self.watched_service.expect("expected the watched service to have been set"),
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
            debug!("Unable to start tracking synchronization, synchronizer is in invalid state {}", state);
            return Err(SyncError::InvalidState(state));
        }

        {
            let mut mutex = self.state.lock().await;
            *mutex = SyncState::Syncing;
        }

        trace!("Syncing tracker movies");
        match self.provider.watched_movies().await {
            Ok(_) => {}
            Err(e) => self.handle_error(e)?,
        }

        info!("Media tracker has been synced");
        self.config.user_settings_ref()
            .tracking_mut()
            .update_state(TrackerSyncState::Success);
        Ok(())
    }

    fn handle_error(&self, err: TrackingError) -> Result<()> {
        error!("Failed to synchronize tracking data, {}", err);
        self.config.user_settings_ref()
            .tracking_mut()
            .update_state(TrackerSyncState::Failed);
        Err(SyncError::Failed(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use mockall::predicate;

    use crate::assert_timeout_eq;
    use crate::core::Handle;
    use crate::core::media::MediaIdentifier;
    use crate::core::media::tracking::MockTrackingProvider;
    use crate::core::media::watched::MockWatchedService;
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_drop() {
        init_logger();
        let handle = Handle::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let config = Arc::new(ApplicationConfig::builder()
            .storage(temp_path)
            .build());
        let watched_service = MockWatchedService::new();
        let mut provider = MockTrackingProvider::new();
        provider.expect_add()
            .times(1)
            .return_const(handle.clone());
        provider.expect_remove()
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
        let config = Arc::new(ApplicationConfig::builder()
            .storage(temp_path)
            .build());
        let mut provider = MockTrackingProvider::new();
        provider.expect_add()
            .return_const(Handle::new());
        provider.expect_remove()
            .return_const(());
        provider.expect_watched_movies()
            .returning(|| {
                Ok(Vec::<Box<dyn MediaIdentifier>>::new())
            });
        let watched_service = MockWatchedService::new();
        let sync = SyncMediaTracking::builder()
            .config(config)
            .tracking_provider(Arc::new(Box::new(provider)))
            .watched_service(Arc::new(Box::new(watched_service)))
            .build();

        sync.start_sync();
        assert_timeout_eq!(Duration::from_millis(200), true, sync.inner.config.user_settings().tracking().last_sync().is_some());

        let settings = sync.inner.config.user_settings();
        let result = settings.tracking().last_sync().unwrap();
        assert_eq!(TrackerSyncState::Success, result.state);
    }

    #[test]
    fn test_sync_watched_movies_error() {
        init_logger();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let config = Arc::new(ApplicationConfig::builder()
            .storage(temp_path)
            .build());
        let mut provider = MockTrackingProvider::new();
        provider.expect_add()
            .return_const(Handle::new());
        provider.expect_remove()
            .return_const(());
        provider.expect_watched_movies()
            .returning(|| {
               Err(TrackingError::Retrieval)
            });
        let watched_service = MockWatchedService::new();
        let sync = SyncMediaTracking::builder()
            .config(config)
            .tracking_provider(Arc::new(Box::new(provider)))
            .watched_service(Arc::new(Box::new(watched_service)))
            .build();

        sync.start_sync();
        assert_timeout_eq!(Duration::from_millis(200), true, sync.inner.config.user_settings().tracking().last_sync().is_some());

        let settings = sync.inner.config.user_settings();
        let result = settings.tracking().last_sync().unwrap();
        assert_eq!(TrackerSyncState::Failed, result.state);
    }

    #[test]
    fn test_authorization_state_changed() {
        init_logger();
        let (tx, rx) = channel();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let config = Arc::new(ApplicationConfig::builder()
            .storage(temp_path)
            .build());
        let mut provider = MockTrackingProvider::new();
        provider.expect_add()
            .returning(move |e| {
                tx.send(e).unwrap();
                Handle::new()
            });
        provider.expect_remove()
            .return_const(());
        provider.expect_watched_movies()
            .returning(|| {
                Ok(Vec::<Box<dyn MediaIdentifier>>::new())
            });
        let watched_service = MockWatchedService::new();
        let sync = SyncMediaTracking::builder()
            .config(config)
            .tracking_provider(Arc::new(Box::new(provider)))
            .watched_service(Arc::new(Box::new(watched_service)))
            .build();

        let callback = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        callback(TrackingEvent::AuthorizationStateChanged(true));

        assert_timeout_eq!(Duration::from_millis(200), true, sync.inner.config.user_settings().tracking().last_sync().is_some());

        let settings = sync.inner.config.user_settings();
        let result = settings.tracking().last_sync().unwrap();
        assert_eq!(TrackerSyncState::Success, result.state);
    }
}

