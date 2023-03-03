use std::cmp::Ordering;
use std::sync::Arc;

use derive_more::Display;
use log::{debug, error, info, trace};
use reqwest::{Client, ClientBuilder, Response};
use semver::Version;
use tokio::sync::Mutex;
use url::Url;

use crate::core::{CoreCallback, CoreCallbacks, updater};
use crate::core::config::ApplicationConfig;
use crate::core::platform::PlatformData;
use crate::core::updater::{UpdateError, VersionInfo};
use crate::core::updater::UpdateState::CheckingForNewVersion;
use crate::VERSION;

const UPDATE_INFO_FILE: &str = "versions.json";

/// The callback type for update events.
pub type UpdateCallback = CoreCallback<UpdateEvent>;

/// The update events of the updater.
#[derive(Debug, Clone, Display)]
pub enum UpdateEvent {
    /// Invoked when the state of the updater has changed
    #[display(fmt = "Update state changed to {}", _0)]
    StateChanged(UpdateState),
    /// Invoked when a new update is available
    #[display(fmt = "New application update available")]
    UpdateAvailable(VersionInfo),
}

/// The state of the updater
#[repr(i32)]
#[derive(Debug, Clone, Display, PartialEq)]
pub enum UpdateState {
    CheckingForNewVersion = 0,
    UpdateAvailable = 1,
    NoUpdateAvailable = 2,
    Downloading = 3,
    DownloadFinished = 4,
    Installing = 5,
    Error = 6,
}

/// The updater of the application which is responsible of retrieving
/// the latest release information and verifying if an update can be applied.
#[derive(Debug)]
pub struct Updater {
    inner: Arc<InnerUpdater>,
}

impl Updater {
    pub fn new(settings: &Arc<Mutex<ApplicationConfig>>, platform: &Arc<Box<dyn PlatformData>>) -> Self {
        Self::new_with_callbacks(settings, platform, vec![])
    }

    pub fn new_with_callbacks(settings: &Arc<Mutex<ApplicationConfig>>, platform: &Arc<Box<dyn PlatformData>>, callbacks: Vec<UpdateCallback>) -> Self {
        let instance = Self {
            inner: Arc::new(InnerUpdater::new(settings, platform, callbacks))
        };

        instance.start_polling();
        instance
    }

    /// Retrieve the version information from the update channel.
    /// This will return the cached info if present and otherwise poll the channel for the info.
    ///
    /// It returns the version info of the latest release on success, else the [UpdateError].
    pub async fn version_info(&self) -> updater::Result<VersionInfo> {
        self.inner.version_info().await
    }

    /// Poll the [PopcornProperties] for a new version.
    /// This will always query the channel for the latest release information.
    ///
    /// It returns when the action is completed or returns an error when the polling failed.
    pub async fn poll(&self) -> updater::Result<VersionInfo> {
        self.inner.poll().await
    }

    /// Register a new callback for events of the updater.
    pub fn register(&self, callback: UpdateCallback) {
        self.inner.register(callback)
    }

    /// Download the latest update version of the application if available.
    /// The download will do nothing if no new version is available.
    pub async fn download(&self) -> updater::Result<()> {
        self.inner.download().await
    }

    /// Start polling the update channel on a new thread
    fn start_polling(&self) {
        let updater = self.inner.clone();
        self.inner.runtime.spawn(async move {
            updater.poll().await
        });
    }
}

#[derive(Debug)]
struct InnerUpdater {
    settings: Arc<Mutex<ApplicationConfig>>,
    platform: Arc<Box<dyn PlatformData>>,
    /// The client used for polling the information
    client: Client,
    /// The cached version information if available
    cache: Mutex<Option<VersionInfo>>,
    /// The last know state of the updater
    state: Mutex<UpdateState>,
    runtime: tokio::runtime::Runtime,
    /// The event callbacks for the updater
    callbacks: CoreCallbacks<UpdateEvent>,
}

impl InnerUpdater {
    fn new(settings: &Arc<Mutex<ApplicationConfig>>, platform: &Arc<Box<dyn PlatformData>>, callbacks: Vec<UpdateCallback>) -> Self {
        let core_callbacks: CoreCallbacks<UpdateEvent> = Default::default();

        // add the given callbacks to the initial list
        for callback in callbacks.into_iter() {
            core_callbacks.add(callback);
        }

        Self {
            settings: settings.clone(),
            platform: platform.clone(),
            client: ClientBuilder::new()
                .build()
                .unwrap(),
            cache: Mutex::new(None),
            state: Mutex::new(CheckingForNewVersion),
            runtime: tokio::runtime::Runtime::new().unwrap(),
            callbacks: core_callbacks,
        }
    }

    /// Retrieve the version info from the cache or update channel.
    async fn version_info(&self) -> updater::Result<VersionInfo> {
        let mutex = self.cache.lock().await;

        if mutex.is_none() {
            drop(mutex);
            return self.poll().await;
        }

        Ok(mutex.as_ref().unwrap().clone())
    }

    /// Poll the update channel for a new version.
    async fn poll(&self) -> updater::Result<VersionInfo> {
        let settings_mutex = self.settings.lock().await;
        let update_channel = settings_mutex.properties().update_channel();

        self.update_state_async(CheckingForNewVersion).await;
        match Url::parse(update_channel) {
            Ok(mut url) => {
                url = url.join(UPDATE_INFO_FILE).unwrap();
                let response = self.poll_info_from_url(url).await?;
                let version_info = Self::handle_response(response).await?;

                self.update_version_info(&version_info).await;
                Ok(version_info)
            }
            Err(e) => {
                error!("Failed to poll update channel, {}", e);
                Err(UpdateError::InvalidUpdateChannel(update_channel.to_string()))
            }
        }
    }

    async fn update_version_info(&self, version_info: &VersionInfo) {
        let mut mutex = self.cache.lock().await;
        let update_version = Version::parse(version_info.version());

        *mutex = Some(version_info.clone());
        // mutex is not used beyond this point, so release it
        drop(mutex);

        match update_version {
            Ok(version) => {
                let current_version = Self::current_version();

                debug!("Checking current version {} against update channel version {}", current_version, version);
                if version.cmp(&current_version) == Ordering::Greater
                    && self.is_platform_available(version_info) {
                    info!("New version {} is available to be installed", version);
                    self.update_state_async(UpdateState::UpdateAvailable).await;
                    self.callbacks.invoke(UpdateEvent::UpdateAvailable(version_info.clone()))
                } else {
                    info!("Application version {} is up-to-date", VERSION);
                    self.update_state_async(UpdateState::NoUpdateAvailable).await
                }
            }
            Err(e) => {
                error!("Failed to parse update channel version, {}", e);
                self.update_state_async(UpdateState::Error).await
            }
        }
    }

    async fn update_state_async(&self, state: UpdateState) {
        let mut mutex = self.state.lock().await;
        if *mutex == state {
            return; // ignore duplicate state updates
        }

        *mutex = state.clone();
        self.callbacks.invoke(UpdateEvent::StateChanged(state));
    }

    async fn poll_info_from_url(&self, url: Url) -> updater::Result<Response> {
        debug!("Polling version update data from {}", url.as_str());
        self.client.get(url.clone())
            .send()
            .await
            .map_err(|e| {
                error!("Failed to poll update channel, {}", e);
                UpdateError::InvalidUpdateChannel(url.to_string())
            })
    }

    async fn download(&self) -> updater::Result<()> {
        trace!("Starting application update download");
        let current_version = Self::current_version();
        let version_info = self.version_info().await?;
        let channel_version = Version::parse(version_info.version()).unwrap();

        if channel_version.cmp(&current_version) == Ordering::Greater {}

        Ok(())
    }

    fn register(&self, callback: UpdateCallback) {
        self.callbacks.add(callback)
    }

    /// Verify if the current platform update binary is available within the update channel information.
    ///
    /// It returns `true` when the platform is available, else `false`.
    fn is_platform_available(&self, version: &VersionInfo) -> bool {
        version.platforms.contains_key(self.platform_identifier().as_str())
    }

    /// Retrieve the current platform identifier which can be used to get the correct binary from the update channel.
    ///
    /// It returns the identifier as `platform.arch`
    fn platform_identifier(&self) -> String {
        let platform = self.platform.info();
        format!("{}.{}", platform.platform_type.name(), platform.arch)
    }

    async fn handle_response(response: Response) -> updater::Result<VersionInfo> {
        match response.json::<VersionInfo>().await {
            Ok(version) => {
                debug!("Retrieved latest version info {:?}", version);
                Ok(version)
            }
            Err(e) => {
                error!("Failed to parse update info, {}", e);
                Err(UpdateError::Response(e.to_string()))
            }
        }
    }

    fn current_version() -> Version {
        Version::parse(VERSION).expect("expected the current version to be valid")
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use httpmock::Method::GET;
    use httpmock::MockServer;
    use tempfile::tempdir;

    use crate::core::config::PopcornProperties;
    use crate::core::platform::{MockDummyPlatformData, PlatformInfo, PlatformType};
    use crate::core::storage::Storage;
    use crate::core::updater::ChangeLog;
    use crate::testing::init_logger;

    use super::*;

    #[test]
    fn test_poll_version() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (server, settings) = create_server_and_settings(temp_path);
        server.mock(|when, then| {
            when.method(GET)
                .path(format!("/{}", UPDATE_INFO_FILE));
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{
  "version": "1.0.0",
  "platforms": {
    "debian.x86_64": "http://localhost/v1.0.0/popcorn-time_1.0.0.deb"
  },
  "changelog": {
    "features": [
      "lorem ipsum"
    ],
    "bugfixes": [
      "ipsum dolor"
    ]
  }
}"#);
        });
        let mut platform_mock = MockDummyPlatformData::new();
        platform_mock.expect_info()
            .return_const(PlatformInfo {
                platform_type: PlatformType::Linux,
                arch: "x86_64".to_string(),
            });
        let platform = Arc::new(Box::new(platform_mock) as Box<dyn PlatformData>);
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let updater = Updater::new(&settings, &platform);
        let expected_result = VersionInfo {
            version: "1.0.0".to_string(),
            changelog: ChangeLog {
                features: vec!["lorem ipsum".to_string()],
                bugfixes: vec!["ipsum dolor".to_string()],
            },
            platforms: HashMap::from([
                ("debian.x86_64".to_string(), "http://localhost/v1.0.0/popcorn-time_1.0.0.deb".to_string())
            ]),
        };

        let result = runtime.block_on(async {
            updater.version_info().await
        }).expect("expected the poll to succeed");

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_poll_older_version() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (server, settings) = create_server_and_settings(temp_path);
        server.mock(|when, then| {
            when.method(GET)
                .path(format!("/{}", UPDATE_INFO_FILE));
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{
  "version": "0.5.0",
  "platforms": {},
  "changelog": {}
}"#);
        });
        let platform_mock = MockDummyPlatformData::new();
        let platform = Arc::new(Box::new(platform_mock) as Box<dyn PlatformData>);
        let (tx, rx) = channel();
        let _ = Updater::new_with_callbacks(&settings, &platform, vec![Box::new(move |event| {
            tx.send(event).unwrap()
        })]);

        let event = rx.recv_timeout(Duration::from_millis(100)).unwrap();

        match event {
            UpdateEvent::StateChanged(result) => assert_eq!(UpdateState::NoUpdateAvailable, result),
            _ => assert!(false, "expected UpdateEvent::StateChanged")
        }
    }

    #[test]
    fn test_poll_newer_version() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (server, settings) = create_server_and_settings(temp_path);
        server.mock(|when, then| {
            when.method(GET)
                .path(format!("/{}", UPDATE_INFO_FILE));
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{
  "version": "999.0.0",
  "platforms": {
   "debian.x86_64": "http://localhost/v999.0.0/popcorn-time_999.0.0.deb"
  },
  "changelog": {}
}"#);
        });
        let mut platform_mock = MockDummyPlatformData::new();
        platform_mock.expect_info()
            .return_const(PlatformInfo {
                platform_type: PlatformType::Linux,
                arch: "x86_64".to_string(),
            });
        let platform = Arc::new(Box::new(platform_mock) as Box<dyn PlatformData>);
        let (tx, rx) = channel();
        let _ = Updater::new_with_callbacks(&settings, &platform, vec![Box::new(move |event| {
            tx.send(event).unwrap()
        })]);

        let event = rx.recv_timeout(Duration::from_millis(100)).unwrap();

        match event {
            UpdateEvent::StateChanged(result) => assert_eq!(UpdateState::UpdateAvailable, result),
            _ => assert!(false, "expected UpdateEvent::StateChanged")
        }
    }

    fn create_server_and_settings(temp_path: &str) -> (MockServer, Arc<Mutex<ApplicationConfig>>) {
        let server = MockServer::start();
        let update_channel = server.url("");

        (server, Arc::new(Mutex::new(ApplicationConfig {
            storage: Storage::from(temp_path),
            properties: PopcornProperties {
                update_channel,
                providers: Default::default(),
                subtitle: Default::default(),
            },
            settings: Default::default(),
            callbacks: Default::default(),
        })))
    }
}