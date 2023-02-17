use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use log::{debug, trace};
use tokio::sync::Mutex;

use popcorn_fx_core::core::{CoreCallbacks, torrent};
use popcorn_fx_core::core::config::Application;
use popcorn_fx_core::core::torrent::{TorrentInfo, TorrentManager, TorrentManagerCallback, TorrentManagerEvent, TorrentManagerState};

/// The rusttorrent implementation for the [TorrentManager].
pub struct RTTorrentManager {
    internal: Arc<TorrentManagerInner>,
}

impl RTTorrentManager {
    pub fn new(settings: &Arc<Application>) -> Self {
        let wrapper = TorrentManagerInner::new(settings);
        Self {
            internal: Arc::new(wrapper),
        }
    }

    /// Create a new pointer to the [TorrentManagerInner]
    fn instance(&self) -> Arc<TorrentManagerInner> {
        self.internal.clone()
    }

    async fn info<'a>(&'a self, url: &'a str) -> torrent::Result<TorrentInfo> {
        self.internal.info(url).await
    }
}

#[async_trait]
impl TorrentManager for RTTorrentManager {
    fn state(&self) -> TorrentManagerState {
        self.internal.state()
    }

    fn register(&self, callback: TorrentManagerCallback) {
        self.internal.register(callback)
    }

    async fn info<'a>(&'a self, url: &'a str) -> torrent::Result<TorrentInfo> {
        self.internal.info(url).await
    }
}

/// The internal wrapper around the [RTTorrentManager] data.
struct TorrentManagerInner {
    /// The application settings
    settings: Arc<Application>,
    /// The state of the manager
    state: Arc<Mutex<TorrentManagerState>>,
    /// The session used within this manager
    // session: Arc<Mutex<Session>>,
    /// The runtime that is used for async functions
    runtime: Arc<tokio::runtime::Runtime>,
    /// The callbacks for this manager
    callbacks: CoreCallbacks<TorrentManagerEvent>,
}

impl TorrentManagerInner {
    fn new(settings: &Arc<Application>) -> Self {
        Self {
            settings: settings.clone(),
            state: Arc::new(Mutex::new(TorrentManagerState::Initializing)),
            // session: Arc::new(Mutex::new(Session::default())),
            runtime: Arc::new(tokio::runtime::Runtime::new().expect("expected a new runtime")),
            callbacks: CoreCallbacks::default(),
        }
    }

    fn update_state(&self, new_value: TorrentManagerState) {
        let mut mutex = self.state.blocking_lock();
        debug!("Updated torrent manager state to {}", &new_value);
        *mutex = new_value.clone();

        self.callbacks.invoke(TorrentManagerEvent::StateChanged(new_value.clone()))
    }
}

#[async_trait]
impl TorrentManager for TorrentManagerInner {
    fn state(&self) -> TorrentManagerState {
        self.state.blocking_lock().clone()
    }

    fn register(&self, callback: TorrentManagerCallback) {
        self.callbacks.add(callback);
    }

    async fn info<'a>(&'a self, url: &'a str) -> torrent::Result<TorrentInfo> {
        trace!("Retrieving info of {}", url);
        todo!()
    }
}

impl Debug for TorrentManagerInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "settings: {:?}, state: {:?}, callback: {:?}",
               self.settings, self.state, self.callbacks)
    }
}

#[cfg(test)]
mod test {
    use std::thread;
    use std::time::Instant;

    use tempfile::tempdir;

    use popcorn_fx_core::core::config::{PopcornProperties, PopcornSettings, ServerSettings, SubtitleSettings, TorrentSettings, UiSettings};
    use popcorn_fx_core::testing::init_logger;

    use super::*;

    // #[test]
    fn test_state() {
        init_logger();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = create_settings(temp_path);
        let manager = RTTorrentManager::new(&settings);
        let start = Instant::now();

        while manager.state() == TorrentManagerState::Initializing
            && start.elapsed() < Duration::from_secs(5) {
            thread::sleep(Duration::from_millis(25));
        }

        let state = manager.state();

        assert_eq!(TorrentManagerState::Running, state)
    }

    fn create_settings(temp_path: &str) -> Arc<Application> {
        Arc::new(Application::new(
            PopcornProperties::default(),
            PopcornSettings::new(
                SubtitleSettings::default(),
                UiSettings::default(),
                ServerSettings::default(),
                TorrentSettings::new(
                    temp_path,
                    false,
                ),
            ),
        ))
    }
}