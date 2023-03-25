use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use derive_more::Display;
use futures::StreamExt;
use log::{debug, error, info, trace, warn};
use reqwest::{Client, ClientBuilder, Response, StatusCode};
use semver::Version;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use url::Url;

use crate::core::{CoreCallback, CoreCallbacks, updater};
use crate::core::config::ApplicationConfig;
use crate::core::platform::PlatformData;
use crate::core::storage::Storage;
use crate::core::updater::{UpdateError, VersionInfo};
use crate::core::updater::UpdateState::CheckingForNewVersion;
use crate::VERSION;

const UPDATE_INFO_FILE: &str = "versions.json";
const UPDATE_DIRECTORY: &str = "updates";

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
#[derive(Debug, Clone, Display, PartialEq)]
pub enum UpdateState {
    CheckingForNewVersion,
    UpdateAvailable,
    NoUpdateAvailable,
    Downloading,
    /// Indicates that the download has finished.
    /// The `String` points to the downloaded file on the system.
    DownloadFinished(String),
    Installing,
    Error,
}

/// The updater of the application which is responsible of retrieving
/// the latest release information and verifying if an update can be applied.
#[derive(Debug)]
pub struct Updater {
    inner: Arc<InnerUpdater>,
}

impl Updater {
    /// Create a builder instance for the updater.
    pub fn builder() -> UpdaterBuilder {
        UpdaterBuilder::default()
    }

    /// Retrieve the version information from the update channel.
    /// This will return the cached info if present and otherwise poll the channel for the info.
    ///
    /// It returns the version info of the latest release on success, else the [UpdateError].
    pub async fn version_info(&self) -> updater::Result<VersionInfo> {
        self.inner.version_info().await
    }

    /// Retrieve an owned instance of the current update state.
    pub fn state(&self) -> UpdateState {
        self.inner.state()
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

    /// Install the downloaded update.
    /// It will return an error when no update is downloaded.
    pub fn install(&self) -> updater::Result<()> {
        self.inner.install(self.inner.clone())
    }

    /// Start polling the update channel on a new thread
    fn start_polling(&self) {
        let updater = self.inner.clone();
        self.inner.runtime.spawn(async move {
            updater.poll().await
        });
    }
}

/// The builder for creating new [Updater] instances.
#[derive(Default)]
pub struct UpdaterBuilder {
    settings: Option<Arc<Mutex<ApplicationConfig>>>,
    insecure: bool,
    platform: Option<Arc<Box<dyn PlatformData>>>,
    storage_path: Option<String>,
    callbacks: Vec<UpdateCallback>,
    runtime: Option<Arc<Runtime>>,
}

impl UpdaterBuilder {
    pub fn settings(mut self, settings: Arc<Mutex<ApplicationConfig>>) -> Self {
        self.settings = Some(settings);
        self
    }

    pub fn insecure(mut self, insecure: bool) -> Self {
        self.insecure = insecure;
        self
    }

    pub fn platform(mut self, platform: Arc<Box<dyn PlatformData>>) -> Self {
        self.platform = Some(platform);
        self
    }

    pub fn storage_path(mut self, storage_path: &str) -> Self {
        self.storage_path = Some(storage_path.to_owned());
        self
    }

    pub fn with_callback(mut self, callback: UpdateCallback) -> Self {
        self.callbacks.push(callback);
        self
    }

    pub fn runtime(mut self, runtime: Arc<Runtime>) -> Self {
        self.runtime = Some(runtime);
        self
    }

    pub fn build(self) -> Updater {
        let instance = Updater {
            inner: Arc::new(InnerUpdater::new(
                self.settings.expect("Settings are not set"),
                self.insecure,
                self.platform.expect("Platform is not set"),
                self.storage_path.expect("Storage path is not set").as_str(),
                self.callbacks,
                self.runtime
                    .or_else(|| Some(Arc::new(Runtime::new().unwrap())))
                    .unwrap()))
        };

        instance.start_polling();
        instance
    }
}

impl Debug for UpdaterBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpdaterBuilder")
            .field("settings", &self.settings)
            .field("insecure", &self.insecure)
            .field("platform", &self.platform)
            .field("storage_path", &self.storage_path)
            .field("runtime", &self.runtime)
            .finish()
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
    runtime: Arc<Runtime>,
    /// The event callbacks for the updater
    callbacks: CoreCallbacks<UpdateEvent>,
    storage_path: PathBuf,
    /// Indicates if an update is being started
    updating: Mutex<bool>,
}

impl InnerUpdater {
    fn new(settings: Arc<Mutex<ApplicationConfig>>, insecure: bool, platform: Arc<Box<dyn PlatformData>>, storage_path: &str, callbacks: Vec<UpdateCallback>, runtime: Arc<Runtime>) -> Self {
        let core_callbacks: CoreCallbacks<UpdateEvent> = Default::default();

        // add the given callbacks to the initial list
        for callback in callbacks.into_iter() {
            core_callbacks.add(callback);
        }

        Self {
            settings,
            platform,
            client: ClientBuilder::new()
                .danger_accept_invalid_certs(insecure.clone())
                .build()
                .unwrap(),
            cache: Mutex::new(None),
            state: Mutex::new(CheckingForNewVersion),
            runtime,
            callbacks: core_callbacks,
            storage_path: PathBuf::from(storage_path),
            updating: Default::default(),
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

    fn state(&self) -> UpdateState {
        let mutex = self.state.blocking_lock();
        mutex.clone()
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
                let version_info = Self::handle_query_response(response).await?;

                self.update_version_info(&version_info).await;
                Ok(version_info)
            }
            Err(e) => {
                error!("Failed to poll update channel, {}", e);
                self.update_state_async(UpdateState::Error).await;
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
                if self.is_update_available(version_info, &version) {
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
        let version_info = self.version_info().await?;
        let channel_version = Version::parse(version_info.version()).unwrap();

        if self.is_update_available(&version_info, &channel_version) {
            let download_link = version_info.platforms.get(self.platform_identifier().as_str()).expect("expected the platform link to have been found");

            self.update_state_async(UpdateState::Downloading).await;
            return match Url::parse(download_link.as_str()) {
                Ok(url) => self.download_and_store(url).await,
                Err(e) => {
                    warn!("Failed to parse update download url, {}" , e);
                    self.update_state_async(UpdateState::Error).await;
                    Err(UpdateError::InvalidDownloadUrl(download_link.clone()))
                }
            };
        }

        Ok(())
    }

    async fn download_and_store(&self, url: Url) -> updater::Result<()> {
        let directory = self.update_directory_path();
        let url_path = PathBuf::from(url.path());
        let filename = url_path.file_name().expect("expected a valid filename").to_str().unwrap();
        let mut file = self.create_update_file(&directory, filename).await?;

        debug!("Downloading update from {:?}", url);
        match self.client.get(url)
            .send()
            .await {
            Ok(response) => {
                let status_code = response.status();

                trace!("Received update download status code {}", status_code);
                if status_code == StatusCode::OK {
                    let mut stream = response.bytes_stream();
                    while let Some(chunk) = stream.next().await {
                        let chunk = chunk.map_err(|e| {
                            error!("Failed to read update chunk, {}", e);
                            UpdateError::DownloadFailed(status_code.to_string(), filename.to_string())
                        })?;

                        tokio::io::copy(&mut chunk.as_ref(), &mut file).await.map_err(|e| {
                            error!("Failed to write update chunk, {}", e);
                            UpdateError::IO("Failed to write chunk to file".to_string())
                        })?;
                    }

                    let filepath_buf = directory.join(filename);
                    let filepath = filepath_buf.to_str().unwrap();
                    info!("Update has been stored in {}", filepath);
                    self.update_state_async(UpdateState::DownloadFinished(filepath.to_string())).await;
                    return Ok(());
                }

                self.update_state_async(UpdateState::Error).await;
                Err(UpdateError::DownloadFailed(status_code.to_string(), filename.to_string()))
            }
            Err(e) => {
                self.update_state_async(UpdateState::Error).await;
                Err(UpdateError::DownloadFailed("UNKNOWN".to_string(), e.to_string()))
            }
        }
    }

    async fn create_update_file(&self, directory: &PathBuf, filename: &str) -> updater::Result<tokio::fs::File> {
        self.create_updates_directory(directory).await?;
        let filepath = directory.join(filename);
        match tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&filepath)
            .await {
            Ok(e) => Ok(e),
            Err(e) => {
                error!("Failed to create update file, {}", e);
                Err(UpdateError::IO(filepath.to_str().unwrap().to_string()))
            }
        }
    }

    async fn create_updates_directory(&self, directory: &PathBuf) -> updater::Result<()> {
        trace!("Creating updates directory {}", directory.to_str().unwrap());
        match tokio::fs::create_dir_all(directory).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to create update directory, {}", e);
                Err(UpdateError::IO("update directory couldn't be created".to_string()))
            }
        }
    }

    fn install(&self, inner: Arc<InnerUpdater>) -> updater::Result<()> {
        trace!("Starting installer");
        let mutex = self.state.blocking_lock();
        match mutex.clone() {
            UpdateState::DownloadFinished(filepath) => {
                debug!("Starting update installation of {}", filepath);
                let runtime = inner.runtime.clone();
                let clone = inner.clone();

                // make sure the closing state knows the application update has started
                // to prevent accidental deletion of the update file
                let mut updating_mutex = self.updating.blocking_lock();
                *updating_mutex = true;
                drop(updating_mutex);

                runtime.spawn(async move {
                    Command::new(filepath)
                        .spawn()
                        .expect("failed to start update");
                    clone.update_state_async(UpdateState::Installing).await;
                });

                Ok(())
            }
            _ => {
                warn!("Unable to start update, update state is {}", *mutex);
                Err(UpdateError::UpdateNotAvailable(mutex.clone()))
            }
        }
    }

    fn register(&self, callback: UpdateCallback) {
        self.callbacks.add(callback)
    }

    /// Verify if an update is available for the current platform.
    ///
    /// It returns `true` when a new version is available for the platform, else `false`.
    fn is_update_available(&self, version_info: &VersionInfo, channel_version: &Version) -> bool {
        let current_version = Self::current_version();

        if channel_version.cmp(&current_version) == Ordering::Greater {
            let platform_identifier = self.platform_identifier();
            if version_info.platforms.contains_key(platform_identifier.as_str()) {
                return true;
            }
            warn!("New version {} available, but no installer found for {}", channel_version, platform_identifier.as_str());
        }

        false
    }

    /// Retrieve the current platform identifier which can be used to get the correct binary from the update channel.
    ///
    /// It returns the identifier as `platform.arch`
    fn platform_identifier(&self) -> String {
        let platform = self.platform.info();
        format!("{}.{}", platform.platform_type.name(), platform.arch)
    }

    async fn handle_query_response(response: Response) -> updater::Result<VersionInfo> {
        let status_code = response.status();

        if status_code == StatusCode::OK {
            response.json::<VersionInfo>().await.map_err(|e| {
                error!("Failed to parse update info, {}", e);
                UpdateError::Response(e.to_string())
            })
        } else {
            Err(UpdateError::Response(format!("received invalid status code {} from update channel", status_code)))
        }
    }

    /// Retrieve the [PathBuf] to the updates directory used by this [InnerUpdater].
    fn update_directory_path(&self) -> PathBuf {
        self.storage_path.join(UPDATE_DIRECTORY)
    }

    fn current_version() -> Version {
        Version::parse(VERSION).expect("expected the current version to be valid")
    }
}

impl Drop for InnerUpdater {
    fn drop(&mut self) {
        let updating = self.updating.blocking_lock();

        // check if an update has been started
        // if not, we try to clean the updates directory
        if !*updating {
            trace!("Starting cleanup of updates directory located at {:?}", self.update_directory_path());
            match Storage::clean_directory(self.update_directory_path()) {
                Ok(_) => info!("Cleaned updates directory located at {:?}", self.update_directory_path()),
                Err(e) => warn!("Failed to clean the updates directory, {}", e)
            }
        } else {
            debug!("Application update running, not cleaning updates directory")
        }
    }
}

#[cfg(test)]
mod test {
    use std::{fs, thread};
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
    use crate::testing::{copy_test_file, init_logger, read_temp_dir_file, read_test_file, test_resource_filepath};

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
        let runtime = Runtime::new().unwrap();
        let updater = Updater::builder()
            .settings(settings)
            .platform(platform)
            .storage_path(temp_path)
            .insecure(false)
            .build();
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
        let _ = Updater::builder()
            .settings(settings)
            .platform(platform)
            .storage_path(temp_path)
            .insecure(false)
            .with_callback(Box::new(move |event| {
                tx.send(event).unwrap()
            }))
            .build();

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
        let _ = Updater::builder()
            .settings(settings)
            .platform(platform)
            .storage_path(temp_path)
            .insecure(false)
            .with_callback(Box::new(move |event| {
                tx.send(event).unwrap()
            }))
            .build();

        let event = rx.recv_timeout(Duration::from_millis(100)).unwrap();

        match event {
            UpdateEvent::StateChanged(result) => assert_eq!(UpdateState::UpdateAvailable, result),
            _ => assert!(false, "expected UpdateEvent::StateChanged")
        }
    }

    #[test]
    fn test_download() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (server, settings) = create_server_and_settings(temp_path);
        let filename = "popcorn-time_99.0.0.deb";
        let url = server.url("/v99.0.0/popcorn-time_99.0.0.deb");
        server.mock(move |when, then| {
            when.method(GET)
                .path(format!("/{}", UPDATE_INFO_FILE));
            then.status(200)
                .header("content-type", "application/json")
                .body(format!(r#"{{
  "version": "99.0.0",
  "platforms": {{
    "debian.x86_64": "{}"
  }},
  "changelog": {{}}
}}"#, url));
        });
        server.mock(move |when, then| {
            when.method(GET)
                .path("/v99.0.0/popcorn-time_99.0.0.deb");
            then.status(200)
                .header("content-type", "application/octet-stream")
                .body_from_file(test_resource_filepath(filename).to_str().unwrap());
        });
        let mut platform_mock = MockDummyPlatformData::new();
        platform_mock.expect_info()
            .return_const(PlatformInfo {
                platform_type: PlatformType::Linux,
                arch: "x86_64".to_string(),
            });
        let platform = Arc::new(Box::new(platform_mock) as Box<dyn PlatformData>);
        let runtime = Runtime::new().unwrap();
        let updater = Updater::builder()
            .settings(settings)
            .platform(platform)
            .storage_path(temp_path)
            .insecure(false)
            .build();
        let expected_result = read_test_file(filename);

        let _ = runtime.block_on(async {
            updater.download().await
        }).expect("expected the download to succeed");
        let result = read_temp_dir_file(&temp_dir, format!("updates/{}", filename).as_str());

        assert_eq!(expected_result, result)
    }

    #[test]
    fn test_download_not_found() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (server, settings) = create_server_and_settings(temp_path);
        let url = server.url("/unknown.deb");
        server.mock(move |when, then| {
            when.method(GET)
                .path(format!("/{}", UPDATE_INFO_FILE));
            then.status(200)
                .header("content-type", "application/json")
                .body(format!(r#"{{"version": "99.0.0",
  "platforms": {{
    "debian.x86_64": "{}"
  }},
  "changelog": {{}} }}"#, url));
        });
        let mut platform_mock = MockDummyPlatformData::new();
        platform_mock.expect_info()
            .return_const(PlatformInfo {
                platform_type: PlatformType::Linux,
                arch: "x86_64".to_string(),
            });
        let platform = Arc::new(Box::new(platform_mock) as Box<dyn PlatformData>);
        let runtime = Runtime::new().unwrap();
        let updater = Updater::builder()
            .settings(settings)
            .platform(platform)
            .storage_path(temp_path)
            .insecure(false)
            .build();

        let result = runtime.block_on(async {
            updater.download().await
        });

        assert!(result.is_err(), "expected the download to return an error");
        match result.err().unwrap() {
            UpdateError::DownloadFailed(status, _) => assert_eq!(StatusCode::NOT_FOUND.to_string(), status),
            _ => assert!(false, "expected UpdateError::DownloadFailed")
        }
    }

    #[test]
    fn test_install_no_update() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (server, settings) = create_server_and_settings(temp_path);
        server.mock(move |when, then| {
            when.method(GET)
                .path(format!("/{}", UPDATE_INFO_FILE));
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"version": "0.0.1",
  "platforms": {},
  "changelog": {}}"#);
        });
        let platform_mock = MockDummyPlatformData::new();
        let platform = Arc::new(Box::new(platform_mock) as Box<dyn PlatformData>);
        let (tx, rx) = channel();
        let updater = Updater::builder()
            .settings(settings)
            .platform(platform)
            .storage_path(temp_path)
            .insecure(false)
            .with_callback(Box::new(move |event| {
                tx.send(event).unwrap()
            }))
            .build();

        rx.recv_timeout(Duration::from_millis(100))
            .expect("expected the state changed event");

        if let Err(result) = updater.install() {
            match result {
                UpdateError::UpdateNotAvailable(state) => assert_eq!(UpdateState::NoUpdateAvailable, state),
                _ => assert!(false, "expected UpdateError::UpdateNotAvailable")
            }
        } else {
            assert!(false, "expected an error to have been returned")
        }
    }

    #[test]
    fn test_clean_updates_directory() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let updates_directory = temp_dir.path().join(UPDATE_DIRECTORY);
        let filename = "popcorn-time_99.0.0.deb";
        let platform_mock = MockDummyPlatformData::new();
        let platform = Arc::new(Box::new(platform_mock) as Box<dyn PlatformData>);
        let settings = Arc::new(Mutex::new(ApplicationConfig {
            storage: Storage::from(temp_path),
            properties: PopcornProperties {
                loggers: Default::default(),
                update_channel: String::new(),
                providers: Default::default(),
                enhancers: Default::default(),
                subtitle: Default::default(),
            },
            settings: Default::default(),
            callbacks: Default::default(),
        }));
        let updater = Updater::builder()
            .settings(settings)
            .platform(platform)
            .storage_path(temp_path)
            .insecure(false)
            .build();
        copy_test_file(updates_directory.to_str().unwrap(), filename, None);

        // wait for the polling to complete
        while updater.state() == CheckingForNewVersion {
            info!("Waiting for update poll to complete");
            thread::sleep(Duration::from_millis(50));
        }

        // drop the updater to start the cleanup
        drop(updater);

        let dir = fs::read_dir(&updates_directory).unwrap();
        let mut num_files = 0;
        for file in dir {
            warn!("Found remaining file {:?}", file);
            num_files += 1;
        }

        assert_eq!(0, num_files);
    }

    fn create_server_and_settings(temp_path: &str) -> (MockServer, Arc<Mutex<ApplicationConfig>>) {
        let server = MockServer::start();
        let update_channel = server.url("");

        (server, Arc::new(Mutex::new(ApplicationConfig {
            storage: Storage::from(temp_path),
            properties: PopcornProperties {
                loggers: Default::default(),
                update_channel,
                providers: Default::default(),
                enhancers: Default::default(),
                subtitle: Default::default(),
            },
            settings: Default::default(),
            callbacks: Default::default(),
        })))
    }
}