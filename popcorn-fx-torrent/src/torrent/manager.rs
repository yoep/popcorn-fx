use async_trait::async_trait;

use popcorn_fx_core::core::torrent;
use popcorn_fx_core::core::torrent::{TorrentInfo, TorrentManager, TorrentManagerCallback, TorrentManagerState};

/// The rusttorrent implementation for the [TorrentManager].
pub struct RTTorrentManager {
    // internal: Arc<TorrentManagerInner>,
}

impl RTTorrentManager {
    pub fn new() -> Self {
        // let wrapper = TorrentManagerInner::new(settings);
        Self {
            // internal: Arc::new(wrapper),
        }
    }

    /// Create a new pointer to the [TorrentManagerInner]
    // fn instance(&self) -> Arc<TorrentManagerInner> {
    //     self.internal.clone()
    // }

    async fn info<'a>(&'a self, url: &'a str) -> torrent::Result<TorrentInfo> {
        // self.internal.info(url).await
        todo!()
    }
}

#[async_trait]
impl TorrentManager for RTTorrentManager {
    fn state(&self) -> TorrentManagerState {
        // self.internal.state()
        todo!()
    }

    fn register(&self, callback: TorrentManagerCallback) {
        // self.internal.register(callback)
        todo!()
    }

    async fn info<'a>(&'a self, url: &'a str) -> torrent::Result<TorrentInfo> {
        // self.internal.info(url).await
        todo!()
    }
}

// /// The internal wrapper around the [RTTorrentManager] data.
// struct TorrentManagerInner {
//     /// The application settings
//     settings: Arc<ApplicationConfig>,
//     /// The state of the manager
//     state: Arc<Mutex<TorrentManagerState>>,
//     /// The session used within this manager
//     // session: Arc<Mutex<Session>>,
//     /// The runtime that is used for async functions
//     runtime: Arc<tokio::runtime::Runtime>,
//     /// The callbacks for this manager
//     callbacks: CoreCallbacks<TorrentManagerEvent>,
// }
//
// impl TorrentManagerInner {
//     fn new(settings: &Arc<ApplicationConfig>) -> Self {
//         Self {
//             settings: settings.clone(),
//             state: Arc::new(Mutex::new(TorrentManagerState::Initializing)),
//             // session: Arc::new(Mutex::new(Session::default())),
//             runtime: Arc::new(tokio::runtime::Runtime::new().expect("expected a new runtime")),
//             callbacks: CoreCallbacks::default(),
//         }
//     }
//
//     fn update_state(&self, new_value: TorrentManagerState) {
//         let mut mutex = self.state.blocking_lock();
//         debug!("Updated torrent manager state to {}", &new_value);
//         *mutex = new_value.clone();
//
//         self.callbacks.invoke(TorrentManagerEvent::StateChanged(new_value.clone()))
//     }
// }
//
// #[async_trait]
// impl TorrentManager for TorrentManagerInner {
//     fn state(&self) -> TorrentManagerState {
//         self.state.blocking_lock().clone()
//     }
//
//     fn register(&self, callback: TorrentManagerCallback) {
//         self.callbacks.add(callback);
//     }
//
//     async fn info<'a>(&'a self, url: &'a str) -> torrent::Result<TorrentInfo> {
//         trace!("Retrieving info of {}", url);
//         todo!()
//     }
// }
//
// impl Debug for TorrentManagerInner {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "settings: {:?}, state: {:?}, callback: {:?}",
//                self.settings, self.state, self.callbacks)
//     }
// }