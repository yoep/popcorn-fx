use crate::core::channel::{ChannelReceiver, ChannelSender, Reply};
use crate::core::config::ApplicationConfig;
use crate::core::launcher::LauncherOptions;
use crate::core::platform::PlatformData;
use crate::core::updater::task::UpdateTask;
use crate::core::updater::{Error, Result, VersionInfo};
use crate::VERSION;
use derive_more::Display;
use flate2::read::GzDecoder;
use futures::StreamExt;
use fx_callback::{Callback, MultiThreadedCallback, Subscription};
use itertools::Itertools;
use log::{debug, error, info, trace, warn};
use reqwest::{Client, ClientBuilder, Response, StatusCode};
use semver::Version;
use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::fs::OpenOptions;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tar::Archive;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tokio::{fs, select};
use tokio_util::sync::CancellationToken;
use url::Url;

const UPDATE_INFO_FILE: &str = "versions.json";
const UPDATE_DIRECTORY: &str = "updates";
const RUNTIMES_DIRECTORY: &str = "runtimes";

/// Represents the events that can occur during an update process.
#[derive(Debug, Clone, Display)]
pub enum UpdateEvent {
    /// Indicates that the state of the updater has changed.
    #[display("Update state changed to {}", _0)]
    StateChanged(UpdateState),
    /// Indicates that a new update is available for the application.
    #[display("New application update available")]
    UpdateAvailable(VersionInfo),
    /// Indicates that the update download has progressed.
    #[display("The update download has progressed to {:?}", _0)]
    DownloadProgress(DownloadProgress),
    /// Indicates that the update installation has progressed.
    #[display("The update installation has progressed to {:?}", _0)]
    InstallationProgress(InstallationProgress),
}

/// Represents the state of the updater.
#[derive(Debug, Copy, Clone, Display, PartialEq)]
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
#[derive(Debug, Default, Clone)]
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
    sender: ChannelSender<UpdateCommand>,
    callbacks: MultiThreadedCallback<UpdateEvent>,
    cancellation_token: CancellationToken,
}

impl Updater {
    /// Create a builder instance for the updater.
    pub fn builder() -> UpdaterBuilder {
        UpdaterBuilder::default()
    }

    /// Create a new application updater instance.
    pub fn new(
        settings: ApplicationConfig,
        platform: Arc<dyn PlatformData>,
        data_path: &str,
    ) -> Result<Self> {
        let (sender, command_receiver) = channel!(16);

        let mut inner = InnerUpdater::new(settings, platform, data_path)?;
        let callbacks = inner.callbacks.clone();
        let cancellation_token = inner.cancellation_token.clone();
        tokio::spawn(async move {
            inner.run(command_receiver).await;
        });

        Ok(Self {
            sender,
            callbacks,
            cancellation_token,
        })
    }

    /// Returns the latest application version info, if available.
    ///
    /// This might return the cached info if present, otherwise polls the channel for the info.
    pub async fn version_info(&self) -> Result<VersionInfo> {
        self.sender
            .send(|tx| UpdateCommand::GetVersionInfo {
                force_poll: false,
                response: tx,
            })
            .await
            .await
    }

    /// Returns the state of the updater.
    pub async fn state(&self) -> UpdateState {
        self.sender
            .send(|tx| UpdateCommand::GetState { response: tx })
            .await
            .await
            .unwrap_or(UpdateState::Error)
    }

    /// Poll the [PopcornProperties] for a new version.
    ///
    /// This will always query the channel for the latest release information.
    ///
    /// # Returns
    ///
    /// Returns when the action is completed or returns an error when the polling failed.
    pub async fn poll(&self) -> Result<VersionInfo> {
        self.sender
            .send(|tx| UpdateCommand::GetVersionInfo {
                force_poll: true,
                response: tx,
            })
            .await
            .await
    }

    /// Start downloading the latest versions of the application.
    /// Returns an error when the download could not be started.
    pub async fn download(&self) -> Result<()> {
        match self
            .sender
            .send(|tx| UpdateCommand::StartDownload { response: tx })
            .await
            .await
        {
            Ok(_) => {
                self.sender.fire_and_forget(UpdateCommand::Download).await;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Start installing the downloaded updates.
    /// Returns an error when the installation could not be started.
    pub async fn install(&self) -> Result<()> {
        match self
            .sender
            .send(|tx| UpdateCommand::StartInstall { response: tx })
            .await
            .await
        {
            Ok(_) => {
                self.sender.fire_and_forget(UpdateCommand::Install).await;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Clean the updates directory.
    pub async fn clean(&self) -> Result<()> {
        Ok(self
            .sender
            .send(|tx| UpdateCommand::Clean { response: tx })
            .await
            .await?)
    }

    /// Poll the update channel for new versions.
    ///
    /// If the updater state is [UpdateState::CheckingForNewVersion], then the call will be ignored.
    pub async fn check_for_updates(&self) {
        let state = self.state().await;
        if state == UpdateState::CheckingForNewVersion {
            debug!("Updater is already checking for new version, ignoring check_for_updates");
            return;
        }

        self.sender.fire_and_forget(UpdateCommand::Poll).await;
    }
}

impl Callback<UpdateEvent> for Updater {
    fn subscribe(&self) -> Subscription<UpdateEvent> {
        self.callbacks.subscribe()
    }
}

impl Drop for Updater {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
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
/// use popcorn_fx_core::core::updater::{UpdateEvent, UpdaterBuilder};
///
/// let config = ApplicationConfig::builder().build();
/// let platform = Arc::new(Box::new(MyPlatformData));
///
/// let builder = UpdaterBuilder::default()
///     .settings(config)
///     .platform(platform)
///     .data_path("~/.local/share/popcorn-fx");
///
/// let updater = builder.build();
/// ```
///
/// This example creates an `UpdaterBuilder` instance and sets its properties, including the `ApplicationConfig`, `PlatformData`, storage path, and update callback.
/// It then uses the builder to construct an `Updater` instance, which is returned and can be used to check for and install updates.
#[derive(Default)]
pub struct UpdaterBuilder {
    settings: Option<ApplicationConfig>,
    platform: Option<Arc<dyn PlatformData>>,
    data_path: Option<String>,
}

impl UpdaterBuilder {
    /// Sets the application settings for the updater.
    pub fn settings(mut self, settings: ApplicationConfig) -> Self {
        self.settings = Some(settings);
        self
    }

    /// Sets the platform data for the updater.
    pub fn platform(mut self, platform: Arc<dyn PlatformData>) -> Self {
        self.platform = Some(platform);
        self
    }

    /// Sets the data path for the updater.
    pub fn data_path(mut self, storage_path: &str) -> Self {
        self.data_path = Some(storage_path.to_owned());
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
    pub fn build(self) -> Result<Updater> {
        let settings = self.settings.expect("Settings are not set");
        let platform = self.platform.expect("Platform is not set");
        let data_path = self.data_path.expect("Data path is not set");

        Updater::new(settings, platform, data_path.as_str())
    }
}

impl Debug for UpdaterBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpdaterBuilder")
            .field("settings", &self.settings)
            .field("platform", &self.platform)
            .field("storage_path", &self.data_path)
            .finish()
    }
}

#[derive(Debug)]
enum UpdateCommand {
    /// Returns the current state of the updater.
    GetState {
        response: Reply<UpdateState>,
    },
    /// Returns the polled version of the latest application.
    GetVersionInfo {
        force_poll: bool,
        response: Reply<Result<VersionInfo>>,
    },
    /// Start downloading the latest update version.
    StartDownload {
        response: Reply<Result<()>>,
    },
    /// Start installing the downloaded updates.
    StartInstall {
        response: Reply<Result<()>>,
    },
    /// Clean the updates directory.
    Clean {
        response: Reply<()>,
    },
    Download,
    Install,
    Poll,
}

/// Manages the update process by handling configurations, platform-specific data,
/// state management, callbacks, and update tasks.
#[derive(Debug)]
struct InnerUpdater {
    /// The application configuration.
    settings: ApplicationConfig,
    /// The Operating System specific data used for updates.
    platform: Arc<dyn PlatformData>,
    /// The client used for polling the information
    client: Client,
    /// The cached version information if available
    cache: Option<VersionInfo>,
    /// The last know state of the updater
    state: UpdateState,
    /// The event callbacks for the updater
    callbacks: MultiThreadedCallback<UpdateEvent>,
    data_path: PathBuf,
    download_progress: Mutex<DownloadProgress>,
    tasks: Vec<UpdateTask>,
    launcher_options: LauncherOptions,
    cancellation_token: CancellationToken,
}

impl InnerUpdater {
    fn new(
        settings: ApplicationConfig,
        platform: Arc<dyn PlatformData>,
        data_path: &str,
    ) -> Result<Self> {
        let client = ClientBuilder::new()
            .build()
            .map_err(|e| Error::Io(io::Error::new(io::ErrorKind::Other, e)))?;

        Ok(Self {
            settings,
            platform,
            client,
            cache: None,
            state: UpdateState::NoUpdateAvailable,
            callbacks: MultiThreadedCallback::new(),
            data_path: PathBuf::from(data_path),
            download_progress: Default::default(),
            tasks: Default::default(),
            launcher_options: LauncherOptions::new(data_path),
            cancellation_token: Default::default(),
        })
    }

    async fn run(&mut self, mut command_receiver: ChannelReceiver<UpdateCommand>) {
        // start by cleaning the older versions of the application
        self.clean_data_path().await;

        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(command) = command_receiver.recv() => self.on_command(command).await,
            }
        }

        debug!("Updater main loop ended");
    }

    async fn on_command(&mut self, command: UpdateCommand) {
        match command {
            UpdateCommand::GetState { response } => response.send(self.state.clone()),
            UpdateCommand::GetVersionInfo {
                force_poll,
                response,
            } => response.send(self.version_info(force_poll).await),
            UpdateCommand::StartDownload { response } => response.send(self.start_download().await),
            UpdateCommand::StartInstall { response } => response.send(self.start_install().await),
            UpdateCommand::Clean { response } => {
                response.send(self.clean_data_path().await);
            }
            UpdateCommand::Download => self.download().await,
            UpdateCommand::Install => self.install().await,
            UpdateCommand::Poll => {
                let _ = self.poll().await;
            }
        }
    }

    /// Retrieve the version info from the cache or update channel.
    async fn version_info(&mut self, force_poll: bool) -> Result<VersionInfo> {
        if force_poll {
            return self.poll().await;
        }

        match self.cache.as_ref().cloned() {
            None => self.poll().await,
            Some(e) => Ok(e),
        }
    }

    /// Poll the update channel for a new version.
    async fn poll(&mut self) -> Result<VersionInfo> {
        trace!("Polling for application information on the update channel");
        let properties = self.settings.properties();
        let update_channel = properties.update_channel();

        self.update_state(UpdateState::CheckingForNewVersion);
        trace!("Parsing update channel url {}", update_channel);
        match Url::parse(update_channel) {
            Ok(mut url) => {
                url = url
                    .join(UPDATE_INFO_FILE)
                    .map_err(|e| Error::Io(io::Error::new(io::ErrorKind::InvalidData, e)))?;
                let response = self.poll_info_from_url(url).await?;
                let version_info = Self::handle_query_response(response).await?;

                self.update_version_info(&version_info)
                    .await
                    .map(|_| version_info)
            }
            Err(e) => {
                error!("Failed to poll update channel, {}", e);
                self.update_state(UpdateState::Error);
                Err(Error::InvalidUpdateChannel(update_channel.to_string()))
            }
        }
    }

    async fn update_version_info(&mut self, version_info: &VersionInfo) -> Result<()> {
        self.cache = Some(version_info.clone());
        self.create_update_tasks(version_info).await
    }

    async fn create_update_tasks(&mut self, version_info: &VersionInfo) -> Result<()> {
        let platform_identifier = self.platform_identifier();
        let current_version = Self::current_application_version();
        let application_version =
            Version::parse(version_info.application.version()).map_err(|e| {
                Error::InvalidApplicationVersion(
                    version_info.application.version().to_string(),
                    e.to_string(),
                )
            })?;
        let runtime_version = Version::parse(version_info.runtime.version()).map_err(|e| {
            Error::InvalidRuntimeVersion(version_info.runtime.version().to_string(), e.to_string())
        })?;

        debug!(
            "Checking channel app version {} against local version {}",
            current_version.to_string(),
            application_version.to_string()
        );
        if self
            .is_application_update_available(version_info, &application_version)
            .await
        {
            info!(
                "New application version {} is available",
                application_version
            );
            self.tasks.push(
                UpdateTask::builder()
                    .current_version(current_version)
                    .install_directory(application_version.to_string())
                    .new_version(application_version)
                    .download_link(Self::convert_download_link_to_url(
                        version_info
                            .application
                            .download_link(platform_identifier.as_str()),
                    )?)
                    .build(),
            );
        } else {
            info!("Application version {} is up-to-date", VERSION);
        }

        debug!(
            "Checking channel runtime version {} against local version {}",
            self.launcher_options.runtime_version,
            runtime_version.to_string()
        );
        if self
            .is_runtime_update_available(version_info, &runtime_version)
            .await
        {
            info!("New runtime version {} is available", runtime_version);
            self.tasks.push(
                UpdateTask::builder()
                    .current_version(
                        Version::parse(self.launcher_options.runtime_version.as_str()).map_err(
                            |e| {
                                Error::InvalidRuntimeVersion(
                                    self.launcher_options.runtime_version.clone(),
                                    e.to_string(),
                                )
                            },
                        )?,
                    )
                    .new_version(runtime_version)
                    .download_link(Self::convert_download_link_to_url(
                        version_info
                            .runtime
                            .download_link(platform_identifier.as_str()),
                    )?)
                    .install_directory(RUNTIMES_DIRECTORY.to_string())
                    .build(),
            );
        }

        if self.tasks.len() > 0 {
            debug!(
                "A total of {} update tasks have been created",
                self.tasks.len()
            );
            self.update_state(UpdateState::UpdateAvailable);
            self.callbacks
                .invoke(UpdateEvent::UpdateAvailable(version_info.clone()));
        } else {
            self.update_state(UpdateState::NoUpdateAvailable);
        }

        Ok(())
    }

    fn update_state(&mut self, new_state: UpdateState) {
        if self.state == new_state {
            return; // ignore duplicate state updates
        }

        debug!("Changing update state to {}", new_state);
        self.state = new_state;
        self.callbacks.invoke(UpdateEvent::StateChanged(new_state));
    }

    async fn poll_info_from_url(&self, url: Url) -> Result<Response> {
        debug!("Polling update information from {}", url.as_str());
        self.client.get(url.clone()).send().await.map_err(|e| {
            error!("Failed to poll update channel, {}", e);
            Error::InvalidUpdateChannel(url.to_string())
        })
    }

    async fn start_download(&mut self) -> Result<()> {
        if self.state != UpdateState::UpdateAvailable {
            return Err(Error::UpdateNotAvailable(self.state));
        }

        self.update_state(UpdateState::Downloading);
        *self.download_progress.lock().await = DownloadProgress::default();

        let update_directory = self.update_directory_path();
        self.create_updates_directory(&update_directory).await?;
        Ok(())
    }

    async fn download(&mut self) {
        let tasks = std::mem::take(&mut self.tasks);
        let futures = tasks
            .into_iter()
            .map(|task| {
                trace!("Starting download task of {}", task.download_link);
                self.download_update_task(task)
            })
            .collect_vec();

        let tasks: Result<Vec<_>> = futures::future::join_all(futures)
            .await
            .into_iter()
            .collect();
        match tasks {
            Ok(tasks) => {
                self.tasks = tasks;
                self.update_state(UpdateState::DownloadFinished);
            }
            Err(err) => {
                error!("Updater failed to download update, {}", err);
                self.update_state(UpdateState::Error);
            }
        }
    }

    async fn download_update_task(&self, mut task: UpdateTask) -> Result<UpdateTask> {
        let directory = self.update_directory_path();
        let url_path = PathBuf::from(task.download_link.path());
        let filename = url_path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| Error::InvalidDownloadUrl(task.download_link.to_string()))?;
        let (mut file, archive_path) = self.create_update_file(&directory, filename).await?;

        debug!(
            "Downloading update patch from {}",
            task.download_link.as_str()
        );
        let result = async {
            let response = self
                .client
                .get(task.download_link.as_ref())
                .send()
                .await
                .map_err(|e| {
                    trace!(
                        "Received an error for {}, error: {}",
                        task.download_link.as_str(),
                        e
                    );
                    Error::DownloadFailed("UNKNOWN".to_string(), filename.to_string())
                })?;

            let status_code = response.status();

            trace!(
                "Received update download status code {} for {}",
                status_code,
                task.download_link.as_str()
            );
            if status_code != StatusCode::OK {
                return Err(Error::DownloadFailed(
                    status_code.to_string(),
                    filename.to_string(),
                ));
            }

            let total_size = response.content_length().unwrap_or(0);
            let mut stream = response.bytes_stream();

            self.update_download_progress(Some(total_size), None).await;
            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(|e| {
                    error!("Failed to read update chunk, {}", e);
                    Error::DownloadFailed(status_code.to_string(), filename.to_string())
                })?;

                file.write_all(&chunk).await.map_err(Error::Io)?;
                self.update_download_progress(None, Some(chunk.len() as u64))
                    .await;
            }

            file.flush().await.map_err(Error::Io)?;
            Ok(())
        }
        .await;

        if let Err(err) = result {
            if let Err(clean_err) = tokio::fs::remove_file(&archive_path).await {
                debug!(
                    "Unable to clean partial archive at {:?}: {}",
                    archive_path, clean_err
                );
            }
            return Err(err);
        }

        task.set_archive_location(archive_path)?;
        Ok(task)
    }

    async fn create_update_file(
        &self,
        directory: &PathBuf,
        filename: &str,
    ) -> Result<(tokio::fs::File, PathBuf)> {
        let mut candidate_index = 0usize;
        loop {
            let candidate_name = if candidate_index == 0 {
                filename.to_string()
            } else {
                format!("{}.{}", filename, candidate_index)
            };
            let filepath = directory.join(candidate_name);

            match tokio::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&filepath)
                .await
            {
                Ok(file) => return Ok((file, filepath)),
                Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
                    candidate_index += 1;
                }
                Err(err) => {
                    error!("Failed to create update file, {}", err);
                    return Err(Error::Io(err));
                }
            }
        }
    }

    async fn update_download_progress(
        &self,
        total_size: Option<u64>,
        downloaded_size: Option<u64>,
    ) {
        trace!(
            "Updating download progression with downloaded: {:?} and total: {:?}",
            downloaded_size,
            total_size
        );
        let mut download_progress = self.download_progress.lock().await;

        if let Some(total_size) = total_size {
            download_progress.total_size += total_size;
        }
        if let Some(downloaded_size) = downloaded_size {
            download_progress.downloaded += downloaded_size;
        }

        self.callbacks
            .invoke(UpdateEvent::DownloadProgress(download_progress.clone()));
    }

    async fn create_updates_directory(&self, directory: &PathBuf) -> Result<()> {
        trace!("Creating updates directory {:?}", directory);
        match tokio::fs::create_dir_all(directory).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to create update directory, {}", e);
                Err(Error::Io(e))
            }
        }
    }

    async fn start_install(&mut self) -> Result<()> {
        if self.state != UpdateState::DownloadFinished {
            warn!("Unable to start update, update state is {}", self.state);
            return Err(Error::UpdateNotAvailable(self.state));
        }

        self.update_state(UpdateState::Installing);
        Ok(())
    }

    async fn install(&mut self) {
        debug!(
            "Starting update installation from {:?}",
            self.update_directory_path()
        );

        self.update_state(UpdateState::Installing);
        match self.execute_installation().await {
            Ok(_) => {
                info!("Update installation finished, restart required");
                self.update_state(UpdateState::InstallationFinished);
            }
            Err(e) => {
                error!("Update installation failed, {}", e);
                self.update_state(UpdateState::Error);
            }
        }
    }

    async fn execute_installation(&mut self) -> Result<()> {
        let tasks: Vec<&UpdateTask> = self
            .tasks
            .iter()
            .filter(|e| e.archive_location().is_some())
            .collect();
        let destination = &self.data_path;
        let total_tasks = tasks.len();
        let mut index = 0;

        trace!("Installing a total of {} tasks", total_tasks);
        for task in tasks {
            let destination = destination.join(task.install_directory());
            let file = OpenOptions::new()
                .read(true)
                .open(
                    task.archive_location()
                        .expect("expected archive location to be present"),
                )
                .map_err(|e| Error::Io(e))?;
            let gz = GzDecoder::new(file);
            let mut archive = Archive::new(gz);

            debug!(
                "Extracting archive {:?} to {:?}",
                task.archive_location().unwrap(),
                destination
            );
            archive
                .unpack(destination)
                .map_err(|e| Error::ExtractionFailed(e.to_string()))?;
            index += 1;
            info!("Installation task {} of {} completed", index, total_tasks);
        }

        trace!("Updating launcher options");
        let info = self.version_info(false).await?;
        let mut launcher_options = self.launcher_options.clone();

        launcher_options.version = info.application.version;
        launcher_options.runtime_version = info.runtime.version;
        launcher_options
            .write(self.data_path.join(LauncherOptions::filename()))
            .map_err(|e| Error::Io(io::Error::new(io::ErrorKind::Other, e)))?;
        debug!("Launcher options have been updated");

        Ok(())
    }

    /// Verify if an application update is available for the current platform.
    ///
    /// It returns `true` when a new version is available for the platform, else `false`.
    async fn is_application_update_available(
        &self,
        version_info: &VersionInfo,
        channel_version: &Version,
    ) -> bool {
        let current_version = Self::current_application_version();

        if channel_version.cmp(&current_version) == Ordering::Greater {
            let platform_identifier = self.platform_identifier();
            if let Some(url) = version_info
                .application
                .download_link(platform_identifier.as_str())
            {
                trace!("Verifying if application download link exists for {}", url);
                return match self.client.head(url.as_str()).send().await {
                    Ok(response) => {
                        if response.status().is_success() || response.status() == StatusCode::FOUND
                        {
                            debug!("Application download link is available at {}", url);
                            true
                        } else {
                            warn!(
                                "Application download link is unavailable, status {}",
                                response.status()
                            );
                            false
                        }
                    }
                    Err(e) => {
                        warn!("Application download link is unavailable, {}", e);
                        false
                    }
                };
            }
            warn!(
                "New version {} available, but no installer found for {}",
                channel_version,
                platform_identifier.as_str()
            );
        }

        false
    }

    /// Verify if a runtime update is available for the current platform.
    ///
    /// It returns `true` when a new runtime version is available for the platform, else `false`.
    async fn is_runtime_update_available(
        &self,
        version_info: &VersionInfo,
        runtime_version: &Version,
    ) -> bool {
        let current_runtime_version =
            Version::parse(self.launcher_options.runtime_version.as_str()).unwrap();

        if runtime_version.cmp(&current_runtime_version) == Ordering::Greater {
            let platform_identifier = self.platform_identifier();
            if let Some(url) = version_info
                .runtime
                .platforms
                .get(platform_identifier.as_str())
            {
                trace!("Verifying if runtime download link exists for {}", url);
                return match self.client.head(url.as_str()).send().await {
                    Ok(response) => {
                        if response.status().is_success() || response.status() == StatusCode::FOUND
                        {
                            debug!("Runtime download link is available at {}", url);
                            true
                        } else {
                            warn!(
                                "Runtime download link is unavailable (status {}), {}",
                                response.status(),
                                url.as_str()
                            );
                            false
                        }
                    }
                    Err(e) => {
                        warn!("Runtime download link is unavailable, {}", e);
                        false
                    }
                };
            }
            warn!(
                "New runtime version {} available, but no runtime update found for {}",
                runtime_version,
                platform_identifier.as_str()
            );
        }

        false
    }

    /// Clean older installation versions of the application in the current data path.
    async fn clean_data_path(&self) {
        let options = &self.launcher_options;

        // clean old application versions
        match self
            .clean_installation_dir(&self.data_path, options.version.as_str())
            .await
        {
            Ok(_) => {
                debug!("Old application versions have been removed");
            }
            Err(e) => {
                error!("Updater failed to clean old application versions, {}", e);
            }
        }

        // clean old runtime versions
        match self
            .clean_installation_dir(
                self.data_path.join(RUNTIMES_DIRECTORY),
                options.runtime_version.as_str(),
            )
            .await
        {
            Ok(_) => {
                debug!("Old runtime versions have been removed");
            }
            Err(e) => {
                error!("Updater failed to clean old runtime versions, {}", e);
            }
        }
    }

    async fn clean_installation_dir<P: AsRef<Path>>(
        &self,
        path: P,
        expected_version: &str,
    ) -> Result<()> {
        let path = path.as_ref();
        let launcher_filename = PathBuf::from(LauncherOptions::filename())
            .file_stem()
            .and_then(|e| e.to_str())
            .map(String::from)
            .ok_or(Error::Io(io::Error::new(
                io::ErrorKind::Other,
                "launcher filename is invalid".to_string(),
            )))?;

        let mut entries = fs::read_dir(path).await?;
        while let Ok(Some(entry)) = entries.next_entry().await {
            if !entry.file_type().await?.is_dir() {
                continue;
            }

            let name = entry.file_name().into_string().map_err(|_| {
                Error::Io(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "failed to read filename",
                ))
            })?;
            if name.starts_with(RUNTIMES_DIRECTORY) || name.starts_with(launcher_filename.as_str())
            {
                continue;
            }

            if name != expected_version {
                trace!(
                    "Updater is removing old version {} from {:?}",
                    name,
                    entry.path()
                );
                tokio::fs::remove_dir_all(entry.path()).await?;
                debug!("Updater removed {:?}", entry.path());
            }
        }
        Ok(())
    }

    /// Retrieve the current platform identifier which can be used to get the correct binary from the update channel.
    ///
    /// It returns the identifier as `platform.arch`
    fn platform_identifier(&self) -> String {
        let platform = self.platform.info();
        format!("{}.{}", platform.platform_type.name(), platform.arch)
    }

    async fn handle_query_response(response: Response) -> Result<VersionInfo> {
        let status_code = response.status();

        if status_code == StatusCode::OK {
            response.json::<VersionInfo>().await.map_err(|e| {
                error!("Failed to parse update info, {}", e);
                Error::Response(e.to_string())
            })
        } else {
            Err(Error::Response(format!(
                "received invalid status code {} from update channel",
                status_code
            )))
        }
    }

    /// Retrieve the [PathBuf] to the updates directory used by this [InnerUpdater].
    fn update_directory_path(&self) -> PathBuf {
        self.data_path.join(UPDATE_DIRECTORY)
    }

    fn convert_download_link_to_url(link: Option<&String>) -> Result<Url> {
        match link {
            None => Err(Error::PlatformUpdateUnavailable),
            Some(e) => Url::parse(e.as_str()).map_err(|e| {
                warn!("Download link is invalid for {:?}", link);
                Error::InvalidDownloadUrl(e.to_string())
            }),
        }
    }

    fn current_application_version() -> Version {
        Version::parse(VERSION).expect("expected the current version to be valid")
    }
}

#[cfg(test)]
mod test {
    use crate::core::config::PopcornProperties;
    use crate::core::platform::{PlatformInfo, PlatformType};
    use crate::core::updater::PatchInfo;
    use crate::testing::{
        copy_test_file, read_temp_dir_file_as_bytes, read_temp_dir_file_as_string,
        read_test_file_to_bytes, read_test_file_to_string, test_resource_filepath,
        MockDummyPlatformData,
    };
    use crate::{assert_timeout, assert_timeout_eq, recv_timeout};
    use httpmock::Method::{GET, HEAD};
    use httpmock::MockServer;
    use std::collections::HashMap;
    use std::fs;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::sync::mpsc::unbounded_channel;

    use super::*;

    #[tokio::test]
    async fn test_poll_version() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (server, settings) = create_server_and_settings(temp_path);
        server.mock(|when, then| {
            when.method(GET).path(format!("/{}", UPDATE_INFO_FILE));
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{
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
}"#,
                );
        });
        let platform = default_platform_info();
        let updater = Updater::builder()
            .settings(settings)
            .platform(platform)
            .data_path(temp_path)
            .build()
            .unwrap();
        let expected_result = VersionInfo {
            application: PatchInfo {
                version: "1.0.0".to_string(),
                platforms: HashMap::from([(
                    "debian.x86_64".to_string(),
                    "http://localhost/v1.0.0/popcorn-time_1.0.0.deb".to_string(),
                )]),
            },
            runtime: PatchInfo {
                version: "17.0.6".to_string(),
                platforms: HashMap::from([(
                    "debian.x86_64".to_string(),
                    "http://localhost/runtime_debian_x86_64.tar.gz".to_string(),
                )]),
            },
        };

        let result = updater
            .version_info()
            .await
            .expect("expected the poll to succeed");

        assert_eq!(expected_result, result)
    }

    #[tokio::test]
    async fn test_poll_download_link_unavailable() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (server, settings) = create_server_and_settings(temp_path);
        server.mock(|when, then| {
            when.method(GET).path(format!("/{}", UPDATE_INFO_FILE));
            then.status(200)
                .header("content-type", "application/json")
                .body(format!(
                    r#"{{
  "application": {{
    "version": "999.0.0",
    "platforms": {{
        "debian.x86_64": "{}"
    }}
  }},
  "runtime": {{
    "version": "2.0.0",
    "platforms": {{}}
  }}
}}"#,
                    server.url("/v999.0.0/popcorn-time_999.0.0.deb")
                ));
        });
        let platform = default_platform_info();
        let updater = Updater::builder()
            .settings(settings)
            .platform(platform)
            .data_path(temp_path)
            .build()
            .unwrap();

        assert_timeout_eq!(
            Duration::from_millis(500),
            UpdateState::NoUpdateAvailable,
            updater.state().await
        );
    }

    #[tokio::test]
    async fn test_install_no_update() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (server, settings) = create_server_and_settings(temp_path);
        no_update_response(&server);
        let platform = default_platform_info();
        let updater = Updater::builder()
            .settings(settings)
            .platform(platform)
            .data_path(temp_path)
            .build()
            .unwrap();

        // poll the latest version info
        let result = updater.poll().await.expect("expected the poll to succeed");
        assert_eq!(
            "0.0.5",
            result.application.version.as_str(),
            "expected the application version to match"
        );

        // check the current state of the updater
        let result = updater.state().await;
        assert_eq!(UpdateState::NoUpdateAvailable, result);

        // try to install a non-existing update
        let result = updater.install().await;
        match result {
            Err(Error::UpdateNotAvailable(state)) => {
                assert_eq!(UpdateState::NoUpdateAvailable, state);
            }
            _ => assert!(
                false,
                "expected Err(Error::UpdateNotAvailable), but got {:?}",
                result
            ),
        }
    }

    #[tokio::test]
    async fn test_install_update_application() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let application_patch_filepath = temp_dir.path().join("99.0.0").join("test.txt");
        let (server, settings) = create_server_and_settings(temp_path);
        let application_patch_url = server.url("/application.tar.gz");
        server.mock(move |when, then| {
            when.method(GET).path(format!("/{}", UPDATE_INFO_FILE));
            then.status(200)
                .header("content-type", "application/json")
                .body(format!(
                    r#"{{
  "application": {{
    "version": "99.0.0",
    "platforms": {{
        "debian.x86_64": "{}"
    }}
  }},
  "runtime": {{
    "version": "1.0.0",
    "platforms": {{}}
  }}
 }}"#,
                    application_patch_url
                ));
        });
        server.mock(|when, then| {
            when.method(HEAD).path("/application.tar.gz");
            then.status(302);
        });
        server.mock(|when, then| {
            when.method(GET).path("/application.tar.gz");
            then.status(200).body_from_file(
                test_resource_filepath("application.tar.gz")
                    .to_str()
                    .unwrap(),
            );
        });
        let platform = default_platform_info();
        let updater = Updater::builder()
            .settings(settings)
            .platform(platform)
            .data_path(temp_path)
            .build()
            .unwrap();

        // poll the latest version info
        let result = updater.poll().await.expect("expected the poll to succeed");
        assert_eq!(
            "99.0.0",
            result.application.version.as_str(),
            "expected the application version to match"
        );

        // download the update
        if let Err(err) = updater.download().await {
            assert!(false, "expected the download to succeed, {}", err);
        }

        // install the update
        if let Err(err) = updater.install().await {
            assert!(false, "expected the installation to succeed, {}", err);
        }

        // wait for the installation to complete
        assert_timeout_eq!(
            Duration::from_millis(200),
            UpdateState::InstallationFinished,
            updater.state().await
        );

        // verify if the patch file exists
        assert!(
            application_patch_filepath.exists(),
            "expected application patch file {:?} to exist",
            application_patch_filepath
        );
    }

    #[tokio::test]
    async fn test_install_update() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let runtime_patch_filepath = temp_dir.path().join("runtimes").join("runtime.txt");
        let (server, settings) = create_server_and_settings(temp_path);
        let runtime_patch_url = server.url("/runtime.tar.gz");
        server.mock(move |when, then| {
            when.method(GET).path(format!("/{}", UPDATE_INFO_FILE));
            then.status(200)
                .header("content-type", "application/json")
                .body(format!(
                    r#"{{
  "application": {{
    "version": "1.0.0",
    "platforms": {{}}
  }},
  "runtime": {{
    "version": "99.0.0",
    "platforms": {{
        "debian.x86_64": "{}"
    }}
  }}
 }}"#,
                    runtime_patch_url
                ));
        });
        server.mock(|when, then| {
            when.method(HEAD).path("/runtime.tar.gz");
            then.status(302);
        });
        server.mock(|when, then| {
            when.method(GET).path("/runtime.tar.gz");
            then.status(200)
                .body_from_file(test_resource_filepath("runtime.tar.gz").to_str().unwrap());
        });
        let platform = default_platform_info();
        let updater = Updater::builder()
            .settings(settings)
            .platform(platform)
            .data_path(temp_path)
            .build()
            .unwrap();

        // poll for updates
        let result = updater.poll().await.expect("expected the poll to succeed");
        assert_eq!(
            "99.0.0",
            result.runtime.version.as_str(),
            "expected the application version to match"
        );

        // check the state of the updater
        let result = updater.state().await;
        assert_eq!(UpdateState::UpdateAvailable, result);

        // subscribe to the updater events
        let (tx, mut rx) = unbounded_channel();
        let mut receiver = updater.subscribe();
        tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                if let UpdateEvent::StateChanged(state) = &*event {
                    let _ = tx.send(state.clone());
                }
            }
        });

        // start downloading the update
        updater
            .download()
            .await
            .expect("expected the download to succeed");

        // wait for the download to finish
        let state =
            timeout!(rx.recv(), Duration::from_millis(200)).expect("expected a state update");
        assert_eq!(UpdateState::Downloading, state); // the first state event is the download being started
        let state =
            timeout!(rx.recv(), Duration::from_millis(200)).expect("expected a state update");
        assert_eq!(UpdateState::DownloadFinished, state);

        // install the update
        updater
            .install()
            .await
            .expect("expected the installation to succeed");

        // wait for the installation to complete
        assert_timeout_eq!(
            Duration::from_millis(200),
            UpdateState::InstallationFinished,
            updater.state().await
        );

        // verify if the patch file exists
        assert!(
            runtime_patch_filepath.exists(),
            "expected runtime patch file {:?} to exist",
            runtime_patch_filepath
        );
    }

    #[tokio::test]
    async fn test_cleanup_data_path() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let updates_directory = temp_dir.path().join(UPDATE_DIRECTORY);
        let filename = "popcorn-time_99.0.0.deb";
        let platform_mock = MockDummyPlatformData::new();
        let settings = ApplicationConfig::builder()
            .storage(temp_path)
            .properties(PopcornProperties {
                loggers: Default::default(),
                update_channel: String::new(),
                providers: Default::default(),
                enhancers: Default::default(),
                subtitle: Default::default(),
                tracking: Default::default(),
            })
            .build();
        copy_test_file(updates_directory.to_str().unwrap(), filename, None);

        // create the updater instance
        // it should run the cleanup cycle as first operation within the task loop
        let _updater = Updater::builder()
            .settings(settings)
            .platform(Arc::new(platform_mock))
            .data_path(temp_path)
            .build()
            .unwrap();

        assert_timeout!(
            Duration::from_millis(500),
            updates_directory
                .read_dir()
                .ok()
                .and_then(|mut e| e.next())
                .is_none(),
            "expected the updates directory to have been cleaned"
        );
    }

    #[tokio::test]
    async fn test_check_for_updates() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (tx, mut rx) = unbounded_channel();
        let (server, settings) = create_server_and_settings(temp_path);
        let mut first_mock = server.mock(move |when, then| {
            when.method(GET).path(format!("/{}", UPDATE_INFO_FILE));
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{
  "application": {
    "version": "0.0.1",
    "platforms": {}
  },
  "runtime": {
    "version": "0.0.1",
    "platforms": {}
  }
}"#,
                );
        });
        let platform = default_platform_info();
        let updater = Updater::builder()
            .settings(settings)
            .platform(platform)
            .data_path(temp_path)
            .build()
            .unwrap();

        let mut receiver = updater.subscribe();
        tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                if let UpdateEvent::StateChanged(state) = &*event {
                    let _ = tx.send(state.clone());
                }
            }
        });

        updater.check_for_updates().await;

        // wait for the updating check to start
        let result =
            timeout!(rx.recv(), Duration::from_millis(200)).expect("expected a state change event");
        assert_eq!(UpdateState::CheckingForNewVersion, result);

        // wait for the update check state change result
        let result =
            timeout!(rx.recv(), Duration::from_millis(200)).expect("expected a state change event");
        assert_eq!(UpdateState::NoUpdateAvailable, result);
        first_mock.delete();
        server.mock(|when, then| {
            when.method(GET).path(format!("/{}", UPDATE_INFO_FILE));
            then.status(200)
                .header("content-type", "application/json")
                .body(format!(
                    r#"{{
  "application": {{
    "version": "999.0.0",
    "platforms": {{
        "debian.x86_64": "{}"
    }}
  }},
  "runtime": {{
    "version": "30.0.1",
    "platforms": {{
        "debian.x86_64": "{}"
    }}
  }}
 }}"#,
                    server.url("/app-update"),
                    server.url("/runtime-update")
                ));
        });
        server.mock(move |when, then| {
            when.method(HEAD).path("/app-update");
            then.status(302);
        });
        server.mock(move |when, then| {
            when.method(HEAD).path("/runtime-update");
            then.status(302);
        });

        updater.check_for_updates().await;

        let result = recv_timeout!(&mut rx, Duration::from_millis(200));
        assert_eq!(UpdateState::CheckingForNewVersion, result);
        let result = recv_timeout!(&mut rx, Duration::from_millis(200));
        assert_eq!(UpdateState::UpdateAvailable, result);
    }

    #[tokio::test]
    async fn test_builder_callback() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (tx, mut rx) = unbounded_channel();
        let (server, settings) = create_server_and_settings(temp_path);
        no_update_response(&server);
        let platform = default_platform_info();
        let updater = Updater::builder()
            .settings(settings)
            .platform(platform)
            .data_path(temp_path)
            .build()
            .unwrap();

        let mut receiver = updater.subscribe();
        tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                if let UpdateEvent::StateChanged(_) = &*event {
                    let _ = tx.send((*event).clone());
                }
            }
        });

        updater.check_for_updates().await;

        let event = recv_timeout!(&mut rx, Duration::from_millis(300));
        match event {
            UpdateEvent::StateChanged(_) => {}
            _ => assert!(false, "expected UpdateEvent::StateChanged event"),
        }
    }

    #[tokio::test]
    async fn test_register_callback() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (tx, mut rx) = unbounded_channel();
        let (server, settings) = create_server_and_settings(temp_path);
        no_update_response(&server);
        let platform = default_platform_info();
        let updater = Updater::builder()
            .settings(settings)
            .platform(platform)
            .data_path(temp_path)
            .build()
            .unwrap();

        let mut receiver = updater.subscribe();
        tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                if let UpdateEvent::StateChanged(_) = &*event {
                    tx.send((*event).clone()).unwrap()
                }
            }
        });

        updater.check_for_updates().await;

        let event = recv_timeout!(&mut rx, Duration::from_millis(300));
        match event {
            UpdateEvent::StateChanged(_) => {}
            _ => assert!(false, "expected UpdateEvent::StateChanged event"),
        }
    }

    mod poll {
        use super::*;

        #[tokio::test]
        async fn test_newer_version() {
            init_logger!();
            let temp_dir = tempdir().unwrap();
            let temp_path = temp_dir.path().to_str().unwrap();
            let (server, settings) = create_server_and_settings(temp_path);
            server.mock(|when, then| {
                when.method(GET).path(format!("/{}", UPDATE_INFO_FILE));
                then.status(200)
                    .header("content-type", "application/json")
                    .body(format!(
                        r#"{{
  "application": {{
    "version": "999.0.0",
    "platforms": {{
        "debian.x86_64": "{}"
    }}
  }},
  "runtime": {{
    "version": "1.0.0",
    "platforms": {{}}
  }}
}}"#,
                        server.url("/v999.0.0/popcorn-time_999.0.0.deb")
                    ));
            });
            server.mock(|when, then| {
                when.method(HEAD).path("/v999.0.0/popcorn-time_999.0.0.deb");
                then.status(200);
            });
            let platform = default_platform_info();
            let updater = Updater::builder()
                .settings(settings)
                .platform(platform)
                .data_path(temp_path)
                .build()
                .unwrap();

            let (tx, mut rx) = unbounded_channel();
            let mut receiver = updater.subscribe();
            tokio::spawn(async move {
                while let Ok(event) = receiver.recv().await {
                    if let UpdateEvent::StateChanged(state) = &*event {
                        let _ = tx.send(*state);
                    }
                }
            });

            // poll the latest version info
            let result = updater
                .poll()
                .await
                .expect("expected the version info to have been polled");
            assert_eq!("999.0.0", result.application.version.as_str());

            // check that the returned state is update available
            let result = updater.state().await;
            assert_eq!(UpdateState::UpdateAvailable, result);

            // check that the initial state event was CheckingForNewVersion
            let result = timeout!(rx.recv(), Duration::from_millis(100))
                .expect("expected the state to be updated");
            assert_eq!(UpdateState::CheckingForNewVersion, result);

            // check that the state event is UpdateAvailable
            let result = timeout!(rx.recv(), Duration::from_millis(100))
                .expect("expected the state to be updated");
            assert_eq!(UpdateState::UpdateAvailable, result);
        }

        #[tokio::test]
        async fn test_older_version() {
            init_logger!();
            let temp_dir = tempdir().unwrap();
            let temp_path = temp_dir.path().to_str().unwrap();
            let (server, settings) = create_server_and_settings(temp_path);
            server.mock(|when, then| {
                when.method(GET).path(format!("/{}", UPDATE_INFO_FILE));
                then.status(200)
                    .header("content-type", "application/json")
                    .body(
                        r#"{
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
}"#,
                    );
            });
            let platform = default_platform_info();
            let updater = Updater::builder()
                .settings(settings)
                .platform(platform)
                .data_path(temp_path)
                .build()
                .unwrap();

            let result = updater
                .poll()
                .await
                .expect("expected the version info to have been polled");
            assert_eq!(
                "0.5.0",
                result.application.version.as_str(),
                "expected the application version to match"
            );

            let result = updater.state().await;
            assert_eq!(UpdateState::NoUpdateAvailable, result);
        }

        #[tokio::test]
        async fn test_invalid_application_version() {
            init_logger!();
            let temp_dir = tempdir().unwrap();
            let temp_path = temp_dir.path().to_str().unwrap();
            let (server, settings) = create_server_and_settings(temp_path);
            server.mock(move |when, then| {
                when.method(GET).path("/versions.json");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(r#"{
  "application": {
    "version": "lorem",
    "platforms": {
      "debian.x86_64": "https://github.com/yoep/popcorn-fx/releases/download/v1.0.0/patch_app_1.0.0_debian_x86_64.tar.gz"
    }
  },
  "runtime": {
    "version": "1.0.0",
    "platforms": {
      "debian.x86_64": "https://github.com/yoep/popcorn-fx/releases/download/v1.0.0/patch_runtime_21.0.3_debian_x86_64.tar.gz"
    }
  }
}"#);
            });
            let platform = default_platform_info();
            let updater = Updater::builder()
                .settings(settings)
                .platform(platform)
                .data_path(temp_path)
                .build()
                .unwrap();

            let result = updater.poll().await;
            match result {
                Err(Error::InvalidApplicationVersion(value, _)) => {
                    assert_eq!("lorem", value, "expected the invalid value to match");
                }
                _ => assert!(
                    false,
                    "expected Err(Error::InvalidApplicationVersion), but got {:?}",
                    result
                ),
            }
        }

        #[tokio::test]
        async fn test_invalid_runtime_version() {
            init_logger!();
            let temp_dir = tempdir().unwrap();
            let temp_path = temp_dir.path().to_str().unwrap();
            let (server, settings) = create_server_and_settings(temp_path);
            server.mock(move |when, then| {
                when.method(GET).path("/versions.json");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(r#"{
  "application": {
    "version": "1.0.0",
    "platforms": {
      "debian.x86_64": "https://github.com/yoep/popcorn-fx/releases/download/v1.0.0/patch_app_1.0.0_debian_x86_64.tar.gz"
    }
  },
  "runtime": {
    "version": "FooBar",
    "platforms": {
      "debian.x86_64": "https://github.com/yoep/popcorn-fx/releases/download/v1.0.0/patch_runtime_21.0.3_debian_x86_64.tar.gz"
    }
  }
}"#);
            });
            let platform = default_platform_info();
            let updater = Updater::builder()
                .settings(settings)
                .platform(platform)
                .data_path(temp_path)
                .build()
                .unwrap();

            let result = updater.poll().await;
            match result {
                Err(Error::InvalidRuntimeVersion(value, _)) => {
                    assert_eq!("FooBar", value, "expected the invalid value to match");
                }
                _ => assert!(
                    false,
                    "expected Err(Error::InvalidRuntimeVersion), but got {:?}",
                    result
                ),
            }
        }
    }

    mod download {
        use super::*;

        /// Download the available update and wait for the expected state.
        macro_rules! download_and_wait {
            ($updater:expr) => {{
                download_and_wait!(
                    $updater,
                    crate::core::updater::UpdateState::DownloadFinished
                )
            }};
            ($updater:expr, $state:expr) => {{
                use crate::core::updater::{UpdateState, Updater};

                let updater: &Updater = $updater;
                let expected_state: UpdateState = $state;

                // subscribe to the updater events
                let (tx, mut rx) = unbounded_channel();
                let mut receiver = updater.subscribe();
                tokio::spawn(async move {
                    while let Ok(event) = receiver.recv().await {
                        if let UpdateEvent::StateChanged(state) = &*event {
                            let _ = tx.send(state.clone());
                        }
                    }
                });

                // start downloading the update
                let _ = updater
                    .download()
                    .await
                    .expect("expected the download to start");

                // wait for the download to finish
                let result = timeout!(rx.recv(), Duration::from_millis(100))
                    .expect("expected the download to start");
                assert_eq!(UpdateState::Downloading, result);
                let result = timeout!(rx.recv(), Duration::from_millis(500))
                    .expect("expected the download to finish");
                assert_eq!(expected_state, result);
            }};
        }

        #[tokio::test]
        async fn test_application() {
            init_logger!();
            let temp_dir = tempdir().unwrap();
            let temp_path = temp_dir.path().to_str().unwrap();
            let (server, settings) = create_server_and_settings(temp_path);
            let filename = "popcorn-time_99.0.0.deb";
            let app_url = server.url("/v99.0.0/popcorn-time_99.0.0.deb");
            server.mock(move |when, then| {
                when.method(GET).path(format!("/{}", UPDATE_INFO_FILE));
                then.status(200)
                    .header("content-type", "application/json")
                    .body(format!(
                        r#"{{
  "application": {{
    "version": "99.0.0",
    "platforms": {{
        "debian.x86_64": "{}"
    }}
  }},
  "runtime": {{
    "version": "1.0.0",
    "platforms": {{}}
  }}
}}"#,
                        app_url
                    ));
            });
            server.mock(|when, then| {
                when.method(HEAD).path("/v99.0.0/popcorn-time_99.0.0.deb");
                then.status(302);
            });
            server.mock(move |when, then| {
                when.method(GET).path("/v99.0.0/popcorn-time_99.0.0.deb");
                then.status(200)
                    .header("content-type", "application/octet-stream")
                    .body_from_file(test_resource_filepath(filename).to_str().unwrap());
            });
            let platform = default_platform_info();
            let updater = Updater::builder()
                .settings(settings)
                .platform(platform)
                .data_path(temp_path)
                .build()
                .unwrap();
            let expected_result = read_test_file_to_string(filename);

            // poll the latest version info
            let result = updater.poll().await.expect("expected the poll to succeed");
            assert_eq!(
                "99.0.0",
                result.application.version.as_str(),
                "expected the application version to match"
            );

            // check the state of the updater
            let result = updater.state().await;
            assert_eq!(UpdateState::UpdateAvailable, result);

            // start downloading the update
            download_and_wait!(&updater);

            // verify that the update has been downloaded to the expected path
            let result =
                read_temp_dir_file_as_string(&temp_dir, format!("updates/{}", filename).as_str());
            assert_eq!(expected_result, result)
        }

        #[tokio::test]
        async fn test_runtime() {
            init_logger!();
            let temp_dir = tempdir().unwrap();
            let temp_path = temp_dir.path().to_str().unwrap();
            let (server, settings) = create_server_and_settings(temp_path);
            let filename = "runtime.tar.gz";
            let runtime_url = server.url("/v100.0.0/runtime.tar.gz");
            server.mock(move |when, then| {
                when.method(GET).path(format!("/{}", UPDATE_INFO_FILE));
                then.status(200)
                    .header("content-type", "application/json")
                    .body(format!(
                        r#"{{
  "application": {{
    "version": "1.0.0",
    "platforms": {{}}
  }},
  "runtime": {{
    "version": "100.0.0",
    "platforms": {{
        "debian.x86_64": "{}"
    }}
  }}
}}"#,
                        runtime_url
                    ));
            });
            server.mock(move |when, then| {
                when.method(HEAD).path("/v100.0.0/runtime.tar.gz");
                then.status(302);
            });
            server.mock(move |when, then| {
                when.method(GET).path("/v100.0.0/runtime.tar.gz");
                then.status(200)
                    .header("content-type", "application/octet-stream")
                    .body_from_file(test_resource_filepath(filename).to_str().unwrap());
            });
            let platform = default_platform_info();
            let updater = Updater::builder()
                .settings(settings)
                .platform(platform)
                .data_path(temp_path)
                .build()
                .unwrap();
            let expected_result = read_test_file_to_bytes(filename);

            // poll the latest version info
            let result = updater.poll().await.expect("expected the poll to succeed");
            assert_eq!(
                "100.0.0",
                result.runtime.version.as_str(),
                "expected the application version to match"
            );

            // check the state of the updater
            let result = updater.state().await;
            assert_eq!(UpdateState::UpdateAvailable, result);

            // start downloading the update
            download_and_wait!(&updater);

            // check that the update has been downloaded to the expected path
            let result =
                read_temp_dir_file_as_bytes(&temp_dir, format!("updates/{}", filename).as_str());
            assert_eq!(expected_result, result)
        }

        #[tokio::test]
        async fn test_not_found() {
            init_logger!();
            let temp_dir = tempdir().unwrap();
            let temp_path = temp_dir.path().to_str().unwrap();
            let (server, settings) = create_server_and_settings(temp_path);
            let url = server.url("/unknown.deb");
            server.mock(move |when, then| {
                when.method(GET).path(format!("/{}", UPDATE_INFO_FILE));
                then.status(200)
                    .header("content-type", "application/json")
                    .body(format!(
                        r#"{{
  "application": {{
    "version": "99.0.0",
    "platforms": {{
        "debian.x86_64": "{}"
    }}
  }},
  "runtime": {{
    "version": "17.0.0",
    "platforms": {{}}
  }} }}"#,
                        url
                    ));
            });
            server.mock(move |when, then| {
                when.method(HEAD).path("/unknown.deb");
                then.status(302);
            });
            let platform = default_platform_info();
            let updater = Updater::builder()
                .settings(settings)
                .platform(platform)
                .data_path(temp_path)
                .build()
                .unwrap();

            // poll the latest version info
            let result = updater.poll().await.expect("expected the poll to succeed");
            assert_eq!(
                "99.0.0",
                result.application.version.as_str(),
                "expected the application version to match"
            );

            // try to download the update
            download_and_wait!(&updater, UpdateState::Error);
        }

        #[tokio::test]
        async fn test_multiple_tasks_with_same_filename() {
            init_logger!();
            let temp_dir = tempdir().unwrap();
            let temp_path = temp_dir.path().to_str().unwrap();
            let (server, settings) = create_server_and_settings(temp_path);
            let app_url = server.url("/v99.0.0/shared.bin");
            let runtime_url = server.url("/v100.0.0/shared.bin");
            let app_body = "application-payload";
            let runtime_body = "runtime-payload";
            server.mock(move |when, then| {
                when.method(GET).path(format!("/{}", UPDATE_INFO_FILE));
                then.status(200)
                    .header("content-type", "application/json")
                    .body(format!(
                        r#"{{
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
}}"#,
                        app_url, runtime_url
                    ));
            });
            server.mock(move |when, then| {
                when.method(HEAD).path("/v99.0.0/shared.bin");
                then.status(302);
            });
            server.mock(move |when, then| {
                when.method(HEAD).path("/v100.0.0/shared.bin");
                then.status(302);
            });
            server.mock(move |when, then| {
                when.method(GET).path("/v99.0.0/shared.bin");
                then.status(200).body(app_body);
            });
            server.mock(move |when, then| {
                when.method(GET).path("/v100.0.0/shared.bin");
                then.status(200).body(runtime_body);
            });
            let platform = default_platform_info();
            let updater = Updater::builder()
                .settings(settings)
                .platform(platform)
                .data_path(temp_path)
                .build()
                .unwrap();

            // poll the latest version info
            let _ = updater.poll().await.expect("expected the poll to succeed");
            let result = updater.state().await;
            assert_eq!(UpdateState::UpdateAvailable, result);

            // download the update
            download_and_wait!(&updater);

            let updates_path = temp_dir.path().join(UPDATE_DIRECTORY);
            let files = fs::read_dir(updates_path)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect_vec();
            assert_eq!(
                2,
                files.len(),
                "expected both tasks to store separate archives"
            );

            let mut contents = files
                .iter()
                .map(|path| String::from_utf8(fs::read(path).unwrap()).unwrap())
                .collect_vec();
            contents.sort();
            assert_eq!(
                vec![app_body.to_string(), runtime_body.to_string()],
                contents
            );
        }
    }

    fn default_platform_info() -> Arc<dyn PlatformData> {
        let mut platform_mock = MockDummyPlatformData::new();
        platform_mock.expect_info().returning(|| PlatformInfo {
            platform_type: PlatformType::Linux,
            arch: "x86_64".to_string(),
        });

        Arc::new(platform_mock)
    }

    fn no_update_response(server: &MockServer) {
        server.mock(move |when, then| {
            when.method(GET).path(format!("/{}", UPDATE_INFO_FILE));
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{
  "application": {
    "version": "0.0.5",
    "platforms": {}
  },
  "runtime": {
    "version": "0.2.1",
    "platforms": {}
  }
 }"#,
                )
                .delay(Duration::from_millis(100));
        });
    }

    fn create_server_and_settings(temp_path: &str) -> (MockServer, ApplicationConfig) {
        let server = MockServer::start();
        let update_channel = server.url("");

        (
            server,
            ApplicationConfig::builder()
                .storage(temp_path)
                .properties(PopcornProperties {
                    loggers: Default::default(),
                    update_channel,
                    providers: Default::default(),
                    enhancers: Default::default(),
                    subtitle: Default::default(),
                    tracking: Default::default(),
                })
                .build(),
        )
    }
}
