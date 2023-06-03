use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::sync::Arc;

use derive_more::Display;
use flate2::read::GzDecoder;
use futures::StreamExt;
use log::{debug, error, info, trace, warn};
use reqwest::{Client, ClientBuilder, Response, StatusCode};
use semver::Version;
use tar::Archive;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use url::Url;

use popcorn_fx_common::LauncherOptions;

use crate::core::{CoreCallback, CoreCallbacks, updater};
use crate::core::config::ApplicationConfig;
use crate::core::platform::PlatformData;
use crate::core::storage::Storage;
use crate::core::updater::{UpdateError, VersionInfo};
use crate::core::updater::task::UpdateTask;
use crate::VERSION;

const UPDATE_INFO_FILE: &str = "versions.json";
const UPDATE_DIRECTORY: &str = "updates";
const RUNTIMES_DIRECTORY: &str = "runtimes";

/// A type representing a callback function that can handle update events.
pub type UpdateCallback = CoreCallback<UpdateEvent>;

/// Represents the events that can occur during an update process.
#[derive(Debug, Clone, Display)]
pub enum UpdateEvent {
    /// Indicates that the state of the updater has changed.
    #[display(fmt = "Update state changed to {}", _0)]
    StateChanged(UpdateState),
    /// Indicates that a new update is available for the application.
    #[display(fmt = "New application update available")]
    UpdateAvailable(VersionInfo),
    /// Indicates that the update download has progressed.
    #[display(fmt = "The update download has progressed to {:?}", _0)]
    DownloadProgress(DownloadProgress),
    /// Indicates that the update installation has progressed.
    #[display(fmt = "The update installation has progressed to {:?}", _0)]
    InstallationProgress(InstallationProgress),
}

/// Represents the state of the updater.
#[derive(Debug, Clone, Display, PartialEq)]
pub enum UpdateState {
    /// The updater is currently checking for a new version.
    CheckingForNewVersion,
    /// A new update is available for the application.
    UpdateAvailable,
    /// The updater has found that there is no update available.
    NoUpdateAvailable,
    /// The updater is currently downloading the update.
    Downloading,
    /// The download has finished and the update is ready to be installed.
    DownloadFinished,
    /// The updater is currently installing the update.
    Installing,
    /// The installation has finished and a restart is required.
    InstallationFinished,
    /// The updater has encountered an error.
    Error,
}

/// Represents the current progress of an update being downloaded.
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// The total size of the download in bytes.
    pub total_size: u64,
    /// The total downloaded size of the update in bytes.
    pub downloaded: u64,
}

/// Represents the current progress of an update being installed.
#[derive(Debug, Clone)]
pub struct InstallationProgress {
    /// The current installation task.
    pub task: u16,
    /// The total number of tasks executed during the installation.
    pub total_tasks: u16,
}

/// The updater of the application which is responsible for retrieving
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
    ///
    /// This will return the cached info if present and otherwise poll the channel for the info.
    ///
    /// # Returns
    ///
    /// The version info of the latest release on success, else the [UpdateError].
    pub async fn version_info(&self) -> updater::Result<VersionInfo> {
        self.inner.version_info().await
    }

    /// Retrieve an owned instance of the current update state.
    ///
    /// # Returns
    ///
    /// The current update state.
    pub fn state(&self) -> UpdateState {
        self.inner.state()
    }

    /// Poll the [PopcornProperties] for a new version.
    ///
    /// This will always query the channel for the latest release information.
    ///
    /// # Returns
    ///
    /// Returns when the action is completed or returns an error when the polling failed.
    pub async fn poll(&self) -> updater::Result<VersionInfo> {
        self.inner.poll().await
    }

    /// Register a new callback for events of the updater.
    ///
    /// # Arguments
    ///
    /// * `callback` - the callback to be registered.
    pub fn register(&self, callback: UpdateCallback) {
        self.inner.register(callback)
    }

    /// Download the latest update version of the application if available.
    ///
    /// The download will do nothing if no new version is available.
    ///
    /// # Returns
    ///
    /// An error if the download failed.
    pub async fn download(&self) -> updater::Result<()> {
        self.inner.download()
            .await
            .map_err(|e| {
                warn!("Failed to download update, {}", e);
                e
            })
    }

    /// Install the downloaded update.
    ///
    /// # Returns
    ///
    /// An error when the update installation failed to start.
    pub fn install(&self) -> updater::Result<()> {
        self.inner.install(self.inner.clone())
    }

    /// Poll the update channel for new versions.
    ///
    /// If the updater state is [UpdateState::CheckingForNewVersion], then the call will be ignored.
    pub fn check_for_updates(&self) {
        if self.inner.state() == UpdateState::CheckingForNewVersion {
            debug!("Updater is already checking for new version, ignoring check_for_updates");
            return;
        }

        self.start_polling()
    }

    /// Start polling the update channel on a new thread.
    fn start_polling(&self) {
        let updater = self.inner.clone();
        self.inner.runtime.spawn(async move {
            updater.poll().await
        });
    }
}

/// The builder for creating new [Updater] instances.
///
/// Use this builder to customize and construct [Updater] instances.
///# Example
///
///```no_run
/// use std::sync::{Arc};
/// use tokio::sync::Mutex;
/// use popcorn_fx_core::core::config::ApplicationConfig;
/// use popcorn_fx_core::core::updater::{UpdateCallback, UpdateEvent, UpdaterBuilder};
///
/// let config = Arc::new(Mutex::new(ApplicationConfig::default()));
/// let platform = Arc::new(Box::new(MyPlatformData));
///
/// let callback = UpdateCallback::new(|event| {
///     if let UpdateEvent::UpdateAvailable(version) = event {
///         println!("New update available: {}", version);
///     }
/// });
///
/// let builder = UpdaterBuilder::default()
///     .settings(config)
///     .platform(platform)
///     .data_path("~/.local/share/popcorn-fx")
///     .with_callback(callback);
///
/// let updater = builder.build();
/// ```
///
/// This example creates an `UpdaterBuilder` instance and sets its properties, including the `ApplicationConfig`, `PlatformData`, storage path, and update callback.
/// It then uses the builder to construct an `Updater` instance, which is returned and can be used to check for and install updates.
#[derive(Default)]
pub struct UpdaterBuilder {
    settings: Option<Arc<Mutex<ApplicationConfig>>>,
    insecure: bool,
    platform: Option<Arc<Box<dyn PlatformData>>>,
    data_path: Option<String>,
    callbacks: Vec<UpdateCallback>,
    runtime: Option<Arc<Runtime>>,
}

impl UpdaterBuilder {
    /// Sets the application settings for the updater.
    pub fn settings(mut self, settings: Arc<Mutex<ApplicationConfig>>) -> Self {
        self.settings = Some(settings);
        self
    }

    /// Sets whether the updater should use insecure connections to download updates.
    pub fn insecure(mut self, insecure: bool) -> Self {
        self.insecure = insecure;
        self
    }

    /// Sets the platform data for the updater.
    pub fn platform(mut self, platform: Arc<Box<dyn PlatformData>>) -> Self {
        self.platform = Some(platform);
        self
    }

    /// Sets the data path for the updater.
    pub fn data_path(mut self, storage_path: &str) -> Self {
        self.data_path = Some(storage_path.to_owned());
        self
    }

    /// Adds an update callback to the updater.
    pub fn with_callback(mut self, callback: UpdateCallback) -> Self {
        self.callbacks.push(callback);
        self
    }

    /// Sets the Tokio runtime for the updater.
    pub fn runtime(mut self, runtime: Arc<Runtime>) -> Self {
        self.runtime = Some(runtime);
        self
    }

    /// Constructs a new updater and starts polling the update channel.
    ///
    /// This method constructs a new `Updater` instance using the settings, platform, storage path, and callbacks configured
    /// with the builder's methods. If any of these properties have not been set, this method will panic.
    ///
    /// Additionally, this method starts the updater's polling loop, which checks for updates on a regular basis.
    ///
    /// # Panics
    ///
    /// This method will panic if any of the following required properties have not been set on the builder:
    /// - `settings`
    /// - `platform`
    /// - `data_path`
    pub fn build(self) -> Updater {
        let instance = Updater {
            inner: Arc::new(InnerUpdater::new(
                self.settings.expect("Settings are not set"),
                self.insecure,
                self.platform.expect("Platform is not set"),
                self.data_path.expect("Data path is not set").as_str(),
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
            .field("storage_path", &self.data_path)
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
    data_path: PathBuf,
    download_progress: Mutex<Option<DownloadProgress>>,
    tasks: Mutex<Vec<UpdateTask>>,
    launcher_options: LauncherOptions,
}

impl InnerUpdater {
    fn new(settings: Arc<Mutex<ApplicationConfig>>, insecure: bool, platform: Arc<Box<dyn PlatformData>>, data_path: &str, callbacks: Vec<UpdateCallback>, runtime: Arc<Runtime>) -> Self {
        let core_callbacks: CoreCallbacks<UpdateEvent> = Default::default();

        // add the given callbacks to the initial list
        for callback in callbacks.into_iter() {
            core_callbacks.add(callback);
        }

        Self {
            settings,
            platform,
            client: ClientBuilder::new()
                .danger_accept_invalid_certs(insecure)
                .build()
                .unwrap(),
            cache: Mutex::new(None),
            state: Mutex::new(UpdateState::CheckingForNewVersion),
            runtime,
            callbacks: core_callbacks,
            data_path: PathBuf::from(data_path),
            download_progress: Default::default(),
            tasks: Default::default(),
            launcher_options: LauncherOptions::new(data_path),
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
        trace!("Polling for application information on the update channel");
        let settings_mutex = self.settings.lock().await;
        let update_channel = settings_mutex.properties().update_channel();

        self.update_state_async(UpdateState::CheckingForNewVersion).await;
        trace!("Parsing update channel url {}", update_channel);
        match Url::parse(update_channel) {
            Ok(mut url) => {
                url = url.join(UPDATE_INFO_FILE).unwrap();
                let response = self.poll_info_from_url(url).await?;
                let version_info = Self::handle_query_response(response).await?;

                self.update_version_info(&version_info).await
                    .map(|_| version_info)
            }
            Err(e) => {
                error!("Failed to poll update channel, {}", e);
                self.update_state_async(UpdateState::Error).await;
                Err(UpdateError::InvalidUpdateChannel(update_channel.to_string()))
            }
        }
    }

    async fn update_version_info(&self, version_info: &VersionInfo) -> updater::Result<()> {
        let mut info_mutex = self.cache.lock().await;

        *info_mutex = Some(version_info.clone());
        // mutex is not used beyond this point, so release it
        drop(info_mutex);

        self.create_update_tasks(version_info).await
    }

    async fn create_update_tasks(&self, version_info: &VersionInfo) -> updater::Result<()> {
        let platform_identifier = self.platform_identifier();
        let current_version = Self::current_application_version();
        let application_version = Version::parse(version_info.application.version())
            .map_err(|e| UpdateError::InvalidApplicationVersion(version_info.application.version().to_string(), e.to_string()))?;
        let runtime_version = Version::parse(version_info.runtime.version())
            .map_err(|e| UpdateError::InvalidRuntimeVersion(version_info.runtime.version().to_string(), e.to_string()))?;
        let mut tasks_mutex = self.tasks.lock().await;

        debug!("Checking channel app version {} against local version {}", current_version.to_string(), application_version.to_string());
        if self.is_application_update_available(version_info, &application_version) {
            info!("New application version {} is available", application_version);
            tasks_mutex.push(UpdateTask::builder()
                .current_version(current_version)
                .install_directory(application_version.to_string())
                .new_version(application_version)
                .download_link(Self::convert_download_link_to_url(version_info.application.download_link(platform_identifier.as_str()))?)
                .build());
        } else {
            info!("Application version {} is up-to-date", VERSION);
        }

        debug!("Checking channel runtime version {} against local version {}", self.launcher_options.runtime_version, runtime_version.to_string());
        if self.is_runtime_update_available(version_info, &runtime_version) {
            info!("New runtime version {} is available", runtime_version);
            tasks_mutex.push(UpdateTask::builder()
                .current_version(Version::parse(self.launcher_options.runtime_version.as_str())
                    .map_err(|e| UpdateError::InvalidRuntimeVersion(self.launcher_options.runtime_version.clone(), e.to_string()))?)
                .new_version(runtime_version)
                .download_link(Self::convert_download_link_to_url(version_info.runtime.download_link(platform_identifier.as_str()))?)
                .install_directory(RUNTIMES_DIRECTORY.to_string())
                .build());
        }

        if tasks_mutex.len() > 0 {
            debug!("A total of {} update tasks have been created", tasks_mutex.len());
            self.update_state_async(UpdateState::UpdateAvailable).await;
            self.callbacks.invoke(UpdateEvent::UpdateAvailable(version_info.clone()));
        } else {
            self.update_state_async(UpdateState::NoUpdateAvailable).await;
        }

        Ok(())
    }

    async fn update_state_async(&self, state: UpdateState) {
        let mut mutex = self.state.lock().await;
        if *mutex == state {
            return; // ignore duplicate state updates
        }

        debug!("Changing update state to {}", state);
        *mutex = state.clone();
        self.callbacks.invoke(UpdateEvent::StateChanged(state));
    }

    async fn poll_info_from_url(&self, url: Url) -> updater::Result<Response> {
        debug!("Polling update information from {}", url.as_str());
        self.client.get(url.clone())
            .send()
            .await
            .map_err(|e| {
                error!("Failed to poll update channel, {}", e);
                UpdateError::InvalidUpdateChannel(url.to_string())
            })
    }

    async fn download(&self) -> updater::Result<()> {
        // check the state of the updater
        let current_state = self.state.lock().await;
        if *current_state != UpdateState::UpdateAvailable {
            return Err(UpdateError::UpdateNotAvailable(current_state.clone()));
        }
        drop(current_state);

        trace!("Starting update download process");
        let mut tasks_mutex = self.tasks.lock().await;
        let mut futures = vec![];

        for task in tasks_mutex.iter_mut() {
            trace!("Starting download task of {}", task.download_link);
            futures.push(self.download_update_task(task));
        }

        self.update_state_async(UpdateState::Downloading).await;
        let results: Vec<updater::Result<()>> = futures::future::join_all(futures).await;

        for result in results {
            result?;
        }

        self.update_state_async(UpdateState::DownloadFinished).await;
        Ok(())
    }

    async fn download_update_task(&self, task: &mut UpdateTask) -> updater::Result<()> {
        let directory = self.update_directory_path();
        let url_path = PathBuf::from(task.download_link.path());
        let filename = url_path.file_name().expect("expected a valid filename").to_str().unwrap();
        let mut file = self.create_update_file(&directory, filename).await?;

        debug!("Downloading update patch from {}", task.download_link.as_str());
        match self.client.get(task.download_link.as_ref())
            .send()
            .await {
            Ok(response) => {
                let status_code = response.status();

                trace!("Received update download status code {} for {}", status_code, task.download_link.as_str());
                if status_code == StatusCode::OK {
                    let total_size = response.content_length().unwrap_or(0);
                    let mut stream = response.bytes_stream();

                    self.update_download_progress(Some(total_size), None).await;
                    while let Some(chunk) = stream.next().await {
                        let chunk = chunk.map_err(|e| {
                            error!("Failed to read update chunk, {}", e);
                            UpdateError::DownloadFailed(status_code.to_string(), filename.to_string())
                        })?;

                        tokio::io::copy(&mut chunk.as_ref(), &mut file).await.map_err(|e| {
                            error!("Failed to write update chunk, {}", e);
                            UpdateError::IO("Failed to write chunk to file".to_string())
                        })?;

                        self.update_download_progress(None, Some(chunk.len() as u64)).await;
                    }

                    task.set_archive_location(directory.join(filename))?;
                    return Ok(());
                }

                self.update_state_async(UpdateState::Error).await;
                Err(UpdateError::DownloadFailed(status_code.to_string(), filename.to_string()))
            }
            Err(e) => {
                trace!("Received an error for {}, error: {}", task.download_link.as_str(), e.to_string());
                self.update_state_async(UpdateState::Error).await;
                Err(UpdateError::DownloadFailed("UNKNOWN".to_string(), e.to_string()))
            }
        }
    }

    async fn update_download_progress(&self, total_size: Option<u64>, downloaded_size: Option<u64>) {
        trace!("Updating download progression with downloaded: {:?} and total: {:?}", downloaded_size, total_size);
        let mut mutex = self.download_progress.lock().await;

        if mutex.is_none() {
            *mutex = Some(DownloadProgress {
                total_size: 0,
                downloaded: 0,
            })
        }

        if let Some(total_size) = total_size {
            mutex.as_mut().unwrap().total_size += total_size;
        }
        if let Some(downloaded_size) = downloaded_size {
            mutex.as_mut().unwrap().downloaded += downloaded_size;
        }

        let progress = mutex.as_ref().unwrap().clone();
        trace!("Dropping download progression lock");
        drop(mutex);

        self.callbacks.invoke(UpdateEvent::DownloadProgress(progress));
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

        if let UpdateState::DownloadFinished = *mutex {
            debug!("Starting update installation from {:?}", self.update_directory_path());
            let runtime = inner.runtime.clone();

            runtime.spawn(async move {
                match Self::execute_installation(inner.clone()).await {
                    Ok(_) => {
                        info!("Update installation finished, restart required");
                        inner.update_state_async(UpdateState::InstallationFinished).await;
                    }
                    Err(e) => {
                        error!("Update installation failed, {}", e);
                        inner.update_state_async(UpdateState::Error).await;
                    }
                }
            });

            Ok(())
        } else {
            warn!("Unable to start update, update state is {}", *mutex);
            Err(UpdateError::UpdateNotAvailable(mutex.clone()))
        }
    }

    async fn execute_installation(updater: Arc<InnerUpdater>) -> updater::Result<()> {
        let tasks_mutex = updater.tasks.lock().await;
        let tasks: Vec<&UpdateTask> = tasks_mutex.iter()
            .filter(|e| e.archive_location().is_some())
            .collect();
        let destination = &updater.data_path;
        let total_tasks = tasks.len();
        let mut index = 0;
        updater.update_state_async(UpdateState::Installing).await;

        trace!("Installing a total of {} tasks", total_tasks);
        for task in tasks {
            let destination = destination.join(task.install_directory());
            let file = OpenOptions::new()
                .read(true)
                .open(task.archive_location().expect("expected archive location to be present"))
                .map_err(|e| UpdateError::IO(e.to_string()))?;
            let gz = GzDecoder::new(file);
            let mut archive = Archive::new(gz);

            debug!("Extracting archive {:?} to {:?}", task.archive_location().unwrap(), destination);
            archive.unpack(destination)
                .map_err(|e| UpdateError::ExtractionFailed(e.to_string()))?;
            index += 1;
            info!("Installation task {} of {} completed", index, total_tasks);
        }

        trace!("Updating launcher options");
        let info = updater.version_info().await?;
        let mut launcher_options = updater.launcher_options.clone();

        launcher_options.version = info.application.version;
        launcher_options.runtime_version = info.runtime.version;
        launcher_options.write(updater.data_path.join(LauncherOptions::filename()))
            .map_err(|e| UpdateError::IO(e.to_string()))?;
        debug!("Launcher options have been updated");

        Ok(())
    }

    fn register(&self, callback: UpdateCallback) {
        self.callbacks.add(callback)
    }

    /// Verify if an application update is available for the current platform.
    ///
    /// It returns `true` when a new version is available for the platform, else `false`.
    fn is_application_update_available(&self, version_info: &VersionInfo, channel_version: &Version) -> bool {
        let current_version = Self::current_application_version();

        if channel_version.cmp(&current_version) == Ordering::Greater {
            let platform_identifier = self.platform_identifier();
            if version_info.application.download_link(platform_identifier.as_str()).is_some() {
                return true;
            }
            warn!("New version {} available, but no installer found for {}", channel_version, platform_identifier.as_str());
        }

        false
    }

    /// Verify if a runtime update is available for the current platform.
    ///
    /// It returns `true` when a new runtime version is available for the platform, else `false`.
    fn is_runtime_update_available(&self, version_info: &VersionInfo, runtime_version: &Version) -> bool {
        let current_runtime_version = Version::parse(self.launcher_options.runtime_version.as_str()).unwrap();

        if runtime_version.cmp(&current_runtime_version) == Ordering::Greater {
            let platform_identifier = self.platform_identifier();
            if version_info.runtime.platforms.contains_key(platform_identifier.as_str()) {
                return true;
            }
            warn!("New runtime version {} available, but no runtime update found for {}", runtime_version, platform_identifier.as_str());
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
        self.data_path.join(UPDATE_DIRECTORY)
    }

    fn convert_download_link_to_url(link: Option<&String>) -> updater::Result<Url> {
        match link {
            None => Err(UpdateError::PlatformUpdateUnavailable),
            Some(e) => Url::parse(e.as_str())
                .map_err(|e| {
                    warn!("Download link is invalid for {:?}", link);
                    UpdateError::InvalidDownloadUrl(e.to_string())
                })
        }
    }

    fn current_application_version() -> Version {
        Version::parse(VERSION).expect("expected the current version to be valid")
    }
}

impl Drop for InnerUpdater {
    fn drop(&mut self) {
        trace!("Starting cleanup of update directory located at {:?}", self.update_directory_path());
        match Storage::clean_directory(self.update_directory_path()) {
            Ok(_) => info!("Cleaned updates directory located at {:?}", self.update_directory_path()),
            Err(e) => warn!("Failed to clean the updates directory, {}", e)
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

    use crate::{assert_timeout, assert_timeout_eq};
    use crate::core::config::PopcornProperties;
    use crate::core::platform::{MockDummyPlatformData, PlatformInfo, PlatformType};
    use crate::core::storage::Storage;
    use crate::core::updater::PatchInfo;
    use crate::testing::{copy_test_file, init_logger, read_temp_dir_file_as_string, read_test_file_to_string, test_resource_filepath};

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
  "version": "deprecated",
  "application": {
    "version": "1.0.0",
    "platforms": {
        "debian.x86_64": "http://localhost/v1.0.0/popcorn-time_1.0.0.deb"
    }
  },
  "runtime": {
    "version": "17.0.6",
    "platforms": {
      "debian.x86_64": "http://localhost/runtime_debian_x86_64.tar.gz"
    }
  }
}"#);
        });
        let platform = default_platform_info();
        let runtime = Runtime::new().unwrap();
        let updater = Updater::builder()
            .settings(settings)
            .platform(platform)
            .data_path(temp_path)
            .insecure(false)
            .build();
        let expected_result = VersionInfo {
            application: PatchInfo {
                version: "1.0.0".to_string(),
                platforms: HashMap::from([
                    ("debian.x86_64".to_string(), "http://localhost/v1.0.0/popcorn-time_1.0.0.deb".to_string())
                ]),
            },
            runtime: PatchInfo {
                version: "17.0.6".to_string(),
                platforms: HashMap::from([
                    ("debian.x86_64".to_string(), "http://localhost/runtime_debian_x86_64.tar.gz".to_string())
                ]),
            },
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
  "application": {
    "version": "0.5.0",
    "platforms": {}
  },
  "runtime": {
    "version": "8.0.12",
    "platforms": {
      "debian.x86_64": "http://localhost/runtime.tar.gz"
    }
  }
}"#);
        });
        let platform = default_platform_info();
        let (tx, rx) = channel();
        let _updater = Updater::builder()
            .settings(settings)
            .platform(platform)
            .data_path(temp_path)
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
  "application": {
    "version": "999.0.0",
    "platforms": {
        "debian.x86_64": "http://localhost/v999.0.0/popcorn-time_999.0.0.deb"
    }
  },
  "runtime": {
    "version": "17.0.0",
    "platforms": {}
  }
}"#);
        });
        let platform = default_platform_info();
        let updater = Updater::builder()
            .settings(settings)
            .platform(platform)
            .data_path(temp_path)
            .insecure(false)
            .build();

        assert_timeout_eq!(Duration::from_millis(200), UpdateState::UpdateAvailable, updater.state());
    }

    #[test]
    fn test_download() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (server, settings) = create_server_and_settings(temp_path);
        let filename = "popcorn-time_99.0.0.deb";
        let app_url = server.url("/v99.0.0/popcorn-time_99.0.0.deb");
        let runtime_url = server.url("/v100.0.0/runtime.zip");
        server.mock(move |when, then| {
            when.method(GET)
                .path(format!("/{}", UPDATE_INFO_FILE));
            then.status(200)
                .header("content-type", "application/json")
                .body(format!(r#"{{
  "application": {{
    "version": "99.0.0",
    "platforms": {{
        "debian.x86_64": "{}"
    }}
  }},
  "runtime": {{
    "version": "100.0.0",
    "platforms": {{
        "debian.x86_64": "{}"
    }}
  }}
}}"#, app_url, runtime_url));
        });
        server.mock(move |when, then| {
            when.method(GET)
                .path("/v99.0.0/popcorn-time_99.0.0.deb");
            then.status(200)
                .header("content-type", "application/octet-stream")
                .body_from_file(test_resource_filepath(filename).to_str().unwrap());
        });
        server.mock(move |when, then| {
            when.method(GET)
                .path("/v100.0.0/runtime.zip");
            then.status(200)
                .header("content-type", "application/octet-stream")
                .body_from_file(test_resource_filepath(filename).to_str().unwrap());
        });
        let platform = default_platform_info();
        let runtime = Runtime::new().unwrap();
        let updater = Updater::builder()
            .settings(settings)
            .platform(platform)
            .data_path(temp_path)
            .insecure(false)
            .build();
        let expected_result = read_test_file_to_string(filename);

        // wait for state update available
        assert_timeout_eq!(Duration::from_millis(200), UpdateState::UpdateAvailable, updater.state());

        let _ = runtime.block_on(async {
            updater.download().await
        }).expect("expected the download to succeed");
        let result = read_temp_dir_file_as_string(&temp_dir, format!("updates/{}", filename).as_str());

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
                .body(format!(r#"{{
  "application": {{
    "version": "99.0.0",
    "platforms": {{
        "debian.x86_64": "{}"
    }}
  }},
  "runtime": {{
    "version": "17.0.0",
    "platforms": {{}}
  }} }}"#, url));
        });
        let platform = default_platform_info();
        let runtime = Runtime::new().unwrap();
        let updater = Updater::builder()
            .settings(settings)
            .platform(platform)
            .data_path(temp_path)
            .insecure(false)
            .build();

        // wait for state update available
        assert_timeout_eq!(Duration::from_millis(200), UpdateState::UpdateAvailable, updater.state());

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
        no_update_response(&server);
        let platform = default_platform_info();
        let (tx, rx) = channel();
        let updater = Updater::builder()
            .settings(settings)
            .platform(platform)
            .data_path(temp_path)
            .insecure(false)
            .with_callback(Box::new(move |event| {
                tx.send(event).unwrap()
            }))
            .build();

        rx.recv_timeout(Duration::from_millis(300))
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
    fn test_install_update() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let application_patch_filepath = temp_dir.path().join("99.0.0").join("test.txt");
        let runtime_patch_filepath = temp_dir.path().join("runtimes").join("runtime.txt");
        let (server, settings) = create_server_and_settings(temp_path);
        let application_patch_url = server.url("/application.tar.gz");
        let runtime_patch_url = server.url("/runtime.tar.gz");
        server.mock(move |when, then| {
            when.method(GET)
                .path(format!("/{}", UPDATE_INFO_FILE));
            then.status(200)
                .header("content-type", "application/json")
                .body(format!(r#"{{
  "application": {{
    "version": "99.0.0",
    "platforms": {{
        "debian.x86_64": "{}"
    }}
  }},
  "runtime": {{
    "version": "99.0.0",
    "platforms": {{
        "debian.x86_64": "{}"
    }}
  }}
 }}"#, application_patch_url, runtime_patch_url));
        });
        server.mock(|when, then| {
            when.method(GET)
                .path("/application.tar.gz");
            then.status(200)
                .body_from_file(test_resource_filepath("application.tar.gz").to_str().unwrap());
        });
        server.mock(|when, then| {
            when.method(GET)
                .path("/runtime.tar.gz");
            then.status(200)
                .body_from_file(test_resource_filepath("runtime.tar.gz").to_str().unwrap());
        });
        let platform = default_platform_info();
        let updater = Updater::builder()
            .settings(settings)
            .platform(platform)
            .data_path(temp_path)
            .insecure(false)
            .build();
        let runtime = Runtime::new().unwrap();

        // wait for the UpdateAvailable state
        assert_timeout_eq!(Duration::from_millis(200), UpdateState::UpdateAvailable, updater.state());

        // download the update
        if let Err(err) = runtime.block_on(updater.download()) {
            assert!(false, "expected the download to succeed, {}", err);
        }

        // install the update
        if let Err(err) = updater.install() {
            assert!(false, "expected the installation to succeed, {}", err);
        }

        // wait for the installation to complete
        assert_timeout_eq!(Duration::from_millis(200), UpdateState::InstallationFinished, updater.state());

        // verify if the patch file exists
        assert!(application_patch_filepath.exists(), "expected application patch file {:?} to exist", application_patch_filepath);
        assert!(runtime_patch_filepath.exists(), "expected runtime patch file {:?} to exist", runtime_patch_filepath);
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
            .data_path(temp_path)
            .insecure(false)
            .build();
        copy_test_file(updates_directory.to_str().unwrap(), filename, None);

        // wait for the polling to complete
        while updater.state() == UpdateState::CheckingForNewVersion {
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

    #[test]
    fn test_check_for_updates() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (server, settings) = create_server_and_settings(temp_path);
        let mut first_mock = server.mock(move |when, then| {
            when.method(GET)
                .path(format!("/{}", UPDATE_INFO_FILE));
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{
  "application": {
    "version": "0.0.1",
    "platforms": {}
  },
  "runtime": {
    "version": "0.0.1",
    "platforms": {}
  }
}"#);
        });
        let platform = default_platform_info();
        let updater = Updater::builder()
            .settings(settings)
            .platform(platform)
            .data_path(temp_path)
            .insecure(false)
            .build();

        assert_timeout!(Duration::from_millis(300), updater.state() != UpdateState::CheckingForNewVersion);
        assert_eq!(UpdateState::NoUpdateAvailable, updater.state());
        first_mock.delete();
        server.mock(move |when, then| {
            when.method(GET)
                .path(format!("/{}", UPDATE_INFO_FILE));
            then.status(200)
                .delay(Duration::from_millis(500))
                .header("content-type", "application/json")
                .body(r#"{
  "application": {
    "version": "999.0.0",
    "platforms": {
        "debian.x86_64": "http://localhost:9090/app"
    }
  },
  "runtime": {
    "version": "30.0.1",
    "platforms": {
        "debian.x86_64": "http://localhost:9090/runtime"
    }
  }
 }"#);
        });

        updater.check_for_updates();
        assert_timeout_eq!(Duration::from_millis(200), updater.state(), UpdateState::CheckingForNewVersion);
        assert_timeout_eq!(Duration::from_millis(500), updater.state(), UpdateState::UpdateAvailable);
    }

    #[tokio::test]
    async fn test_update_version_info_invalid_application_version() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = create_simple_settings(temp_path);
        let platform = default_platform_info();
        let updater = Updater::builder()
            .settings(settings)
            .platform(platform)
            .data_path(temp_path)
            .insecure(false)
            .build();

        let result = updater.inner.update_version_info(&VersionInfo {
            application: PatchInfo {
                version: "lorem".to_string(),
                platforms: Default::default(),
            },
            runtime: PatchInfo {
                version: "ipsum".to_string(),
                platforms: Default::default(),
            },
        }).await;

        if let Err(err) = result {
            match err {
                UpdateError::InvalidApplicationVersion(_, _) => {}
                _ => assert!(false, "expected UpdateError::InvalidApplicationVersion")
            }
        } else {
            assert!(false, "expected an error to be returned")
        }
    }

    #[test]
    fn test_builder_callback() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (tx, rx) = channel();
        let (server, settings) = create_server_and_settings(temp_path);
        no_update_response(&server);
        let platform = default_platform_info();
        let _updater = Updater::builder()
            .settings(settings)
            .platform(platform)
            .with_callback(Box::new(move |event| {
                match event {
                    UpdateEvent::StateChanged(_) => tx.send(event).unwrap(),
                    _ => {}
                }
            }))
            .data_path(temp_path)
            .insecure(false)
            .build();

        let event = rx.recv_timeout(Duration::from_millis(300)).unwrap();

        match event {
            UpdateEvent::StateChanged(_) => {}
            _ => assert!(false, "expected UpdateEvent::StateChanged event")
        }
    }

    #[test]
    fn test_register_callback() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (tx, rx) = channel();
        let (server, settings) = create_server_and_settings(temp_path);
        no_update_response(&server);
        let platform = default_platform_info();
        let updater = Updater::builder()
            .settings(settings)
            .platform(platform)
            .data_path(temp_path)
            .insecure(false)
            .build();

        updater.register(Box::new(move |event| {
            match event {
                UpdateEvent::StateChanged(_) => tx.send(event).unwrap(),
                _ => {}
            }
        }));

        let event = rx.recv_timeout(Duration::from_millis(300)).unwrap();

        match event {
            UpdateEvent::StateChanged(_) => {}
            _ => assert!(false, "expected UpdateEvent::StateChanged event")
        }
    }

    #[test]
    fn test_updater_builder_debug() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let builder = UpdaterBuilder::default()
            .settings(create_simple_settings(temp_path))
            .platform(default_platform_info())
            .data_path(temp_path)
            .insecure(false)
            .runtime(Arc::new(Runtime::new().unwrap()));

        let debug_output = format!("{:?}", builder);

        assert!(debug_output.contains("UpdaterBuilder"));
        assert!(debug_output.contains("settings: Some"));
        assert!(debug_output.contains("insecure: false"));
        assert!(debug_output.contains("platform: Some"));
        assert!(debug_output.contains("storage_path: Some"));
        assert!(debug_output.contains("runtime: Some"));
    }

    fn default_platform_info() -> Arc<Box<dyn PlatformData>> {
        let mut platform_mock = MockDummyPlatformData::new();
        platform_mock.expect_info()
            .returning(|| PlatformInfo {
                platform_type: PlatformType::Linux,
                arch: "x86_64".to_string(),
            });
        let platform = Arc::new(Box::new(platform_mock) as Box<dyn PlatformData>);
        platform
    }

    fn no_update_response(server: &MockServer) {
        server.mock(move |when, then| {
            when.method(GET)
                .path(format!("/{}", UPDATE_INFO_FILE));
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{
  "application": {
    "version": "0.0.5",
    "platforms": {}
  },
  "runtime": {
    "version": "0.2.1",
    "platforms": {}
  }
 }"#)
                .delay(Duration::from_millis(100));
        });
    }

    fn create_simple_settings(temp_path: &str) -> Arc<Mutex<ApplicationConfig>> {
        Arc::new(Mutex::new(ApplicationConfig {
            storage: Storage::from(temp_path),
            properties: PopcornProperties {
                loggers: Default::default(),
                update_channel: "http://localhost:8080/update.json".to_string(),
                providers: Default::default(),
                enhancers: Default::default(),
                subtitle: Default::default(),
            },
            settings: Default::default(),
            callbacks: Default::default(),
        }))
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