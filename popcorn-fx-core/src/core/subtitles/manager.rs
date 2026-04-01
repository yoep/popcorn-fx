use crate::core::channel::{ChannelReceiver, ChannelSender, Reply};
use crate::core::config::ApplicationConfig;
use crate::core::media::{Episode, MovieDetails, ShowDetails};
use crate::core::storage::Storage;
use crate::core::subtitles;
use crate::core::subtitles::language::SubtitleLanguage;
use crate::core::subtitles::matcher::SubtitleMatcher;
use crate::core::subtitles::model::{Subtitle, SubtitleInfo, SubtitleType};
use crate::core::subtitles::parsers::Parser;
use crate::core::subtitles::{Result, SubtitleError, SubtitleProvider};
use derive_more::Display;
use fx_callback::{Callback, MultiThreadedCallback, Subscription};
use log::{debug, error, info, trace};
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::select;
use tokio_util::sync::CancellationToken;

/// The callback to listen on events of the subtitle manager.
pub type SubtitleCallback = Subscription<SubtitleEvent>;

/// Represents events related to subtitles.
#[derive(Debug, Clone, Display)]
pub enum SubtitleEvent {
    #[display("subtitle preference changed to {}", _0)]
    PreferenceChanged(SubtitlePreference),
}

/// Represents user preferences for subtitles.
#[derive(Debug, Clone, Display, PartialEq)]
pub enum SubtitlePreference {
    /// Specifies a preferred subtitle language.
    #[display("preferred language {}", _0)]
    Language(SubtitleLanguage),
    /// Indicates subtitles are disabled.
    #[display("disabled")]
    Disabled,
}

/// The subtitle manager manages subtitles for media item playbacks.
#[derive(Debug, Clone)]
pub struct SubtitleManager {
    sender: ChannelSender<SubtitleManagerCommand>,
    callbacks: MultiThreadedCallback<SubtitleEvent>,
    cancellation_token: CancellationToken,
}

impl SubtitleManager {
    /// Create a new subtitle manager builder.
    pub fn builder() -> SubtitleManagerBuilder {
        SubtitleManagerBuilder::new()
    }

    /// Create a new subtitle manager instance.
    pub fn new(
        settings: ApplicationConfig,
        providers: Box<dyn SubtitleProvider>,
        parsers: HashMap<SubtitleType, Box<dyn Parser>>,
    ) -> Self {
        let (sender, receiver) = channel!(128);
        let mut inner = InnerSubtitleManager::new(settings, Arc::from(providers), parsers);
        let callbacks = inner.callbacks.clone();
        let cancellation_token = inner.cancellation_token.clone();

        // spawn the main loop on a separate task
        tokio::spawn(async move {
            inner.run(receiver).await;
        });

        Self {
            sender,
            callbacks,
            cancellation_token,
        }
    }

    /// Returns the default subtitles options.
    pub fn default_subtitle_options() -> Vec<SubtitleInfo> {
        vec![SubtitleInfo::none(), SubtitleInfo::custom()]
    }

    /// Returns the subtitle preference.
    pub async fn preference(&self) -> SubtitlePreference {
        self.sender
            .send(|tx| SubtitleManagerCommand::GetPreference { response: tx })
            .await
            .await
            .unwrap_or(SubtitlePreference::Disabled)
    }

    /// Update the active subtitle preference.
    pub async fn update_preference(&self, preference: SubtitlePreference) {
        let _ = self
            .sender
            .send(|tx| SubtitleManagerCommand::UpdatePreference {
                preference,
                response: tx,
            })
            .await
            .await;
    }

    /// Select a subtitle from the available subtitles list.
    /// The default [SubtitleInfo::none] is returned when none of the available subtitles match
    /// the current subtitle preference.
    pub async fn select_or_default(&self, subtitles: &[SubtitleInfo]) -> SubtitleInfo {
        self.sender
            .send(|tx| SubtitleManagerCommand::SelectOrDefault {
                subtitles: subtitles.to_vec(),
                response: tx,
            })
            .await
            .await
            .unwrap_or(SubtitleInfo::none())
    }

    /// Convert the given [Subtitle] into the given output type.
    pub async fn convert(&self, subtitle: Subtitle, output_type: SubtitleType) -> Result<String> {
        self.sender
            .send(|tx| SubtitleManagerCommand::Convert {
                subtitle,
                output_type,
                response: tx,
            })
            .await
            .await
    }

    /// Returns the available movie subtitles for the given media item.
    pub async fn movie_subtitles(&self, media: &MovieDetails) -> Result<Vec<SubtitleInfo>> {
        let provider = self
            .sender
            .send(|tx| SubtitleManagerCommand::GetProvider { response: tx })
            .await
            .await?;
        provider.movie_subtitles(media).await
    }

    /// Returns the available subtitles for the given media file.
    pub async fn file_subtitles(&self, filename: &str) -> subtitles::Result<Vec<SubtitleInfo>> {
        let provider = self
            .sender
            .send(|tx| SubtitleManagerCommand::GetProvider { response: tx })
            .await
            .await?;
        provider.file_subtitles(filename).await
    }

    /// Returns the available episode subtitles for the given media item.
    pub async fn episode_subtitles(
        &self,
        media: &ShowDetails,
        episode: &Episode,
    ) -> Result<Vec<SubtitleInfo>> {
        let provider = self
            .sender
            .send(|tx| SubtitleManagerCommand::GetProvider { response: tx })
            .await
            .await?;
        provider.episode_subtitles(media, episode).await
    }

    /// Try to download the given subtitle info.
    /// It returns the parsed downloaded subtitle.
    pub async fn download(
        &self,
        subtitle_info: &SubtitleInfo,
        matcher: &SubtitleMatcher,
    ) -> Result<Subtitle> {
        self.sender
            .send(|tx| SubtitleManagerCommand::Download {
                subtitle: subtitle_info.clone(),
                matcher: matcher.clone(),
                response: tx,
            })
            .await
            .await
    }

    /// Reset the selected subtitle preference.
    pub async fn reset(&self) {
        let _ = self
            .sender
            .send(|tx| SubtitleManagerCommand::Reset { response: tx })
            .await
            .await;
    }

    /// Execute a cleanup cycle of the subtitles.
    pub async fn cleanup(&self) {
        let _ = self
            .sender
            .send(|tx| SubtitleManagerCommand::DoCleanup { response: tx })
            .await
            .await;
    }
}

impl Callback<SubtitleEvent> for SubtitleManager {
    fn subscribe(&self) -> Subscription<SubtitleEvent> {
        self.callbacks.subscribe()
    }
}

impl Drop for SubtitleManager {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}

/// The builder for the subtitle manager.
#[derive(Debug, Default)]
pub struct SubtitleManagerBuilder {
    settings: Option<ApplicationConfig>,
    provider: Option<Box<dyn SubtitleProvider>>,
    parsers: HashMap<SubtitleType, Box<dyn Parser>>,
}

impl SubtitleManagerBuilder {
    /// Create a new subtitle manager builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the settings of the application.
    pub fn settings(&mut self, settings: ApplicationConfig) -> &mut Self {
        self.settings = Some(settings);
        self
    }

    /// Set the subtitle provider.
    pub fn provider<P>(&mut self, provider: P) -> &mut Self
    where
        P: SubtitleProvider + 'static,
    {
        self.provider = Some(Box::new(provider));
        self
    }

    /// Add the given subtitle parser.
    pub fn with_parser<P>(&mut self, subtitle_type: SubtitleType, parser: P) -> &mut Self
    where
        P: Parser + 'static,
    {
        self.parsers.insert(subtitle_type, Box::new(parser));
        self
    }

    /// Set the subtitle parsers.
    /// This overrides any existing parsers.
    pub fn set_parsers(&mut self, parsers: HashMap<SubtitleType, Box<dyn Parser>>) -> &mut Self {
        self.parsers = parsers;
        self
    }

    /// Build the subtitle manager.
    ///
    /// # Panics
    ///
    /// Panics if the settings or subtitle provider have not been set.
    pub fn build(&mut self) -> SubtitleManager {
        let settings = self.settings.take().expect("expected settings to be set");
        let provider = self
            .provider
            .take()
            .expect("expected subtitle provider to be set");

        SubtitleManager::new(settings, provider, self.parsers.drain().collect())
    }
}

#[derive(Debug)]
enum SubtitleManagerCommand {
    /// Get the active subtitle preference.
    GetPreference { response: Reply<SubtitlePreference> },
    /// Update the subtitle preference.
    UpdatePreference {
        preference: SubtitlePreference,
        response: Reply<()>,
    },
    /// Select the subtitle from the given list based on preference or configured defaults.
    SelectOrDefault {
        subtitles: Vec<SubtitleInfo>,
        response: Reply<SubtitleInfo>,
    },
    /// Get the subtitle provider.
    GetProvider {
        response: Reply<Arc<dyn SubtitleProvider>>,
    },
    /// Download the subtitle.
    Download {
        subtitle: SubtitleInfo,
        matcher: SubtitleMatcher,
        response: Reply<Result<Subtitle>>,
    },
    /// Convert the given subtitle into the given output type.
    Convert {
        subtitle: Subtitle,
        output_type: SubtitleType,
        response: Reply<Result<String>>,
    },
    /// Reset the subtitle preference to the default.
    Reset { response: Reply<()> },
    /// Clean up the subtitle directory.
    DoCleanup { response: Reply<()> },
}

#[derive(Debug)]
struct InnerSubtitleManager {
    /// The known info of the selected subtitle if applicable.
    preference: SubtitlePreference,
    /// The application settings.
    settings: ApplicationConfig,
    /// The subtitle provider for the manager.
    provider: Arc<dyn SubtitleProvider>,
    /// The subtitle parsers of the manager.
    parsers: HashMap<SubtitleType, Box<dyn Parser>>,
    /// Callbacks for handling subtitle events.
    callbacks: MultiThreadedCallback<SubtitleEvent>,
    cancellation_token: CancellationToken,
}

impl InnerSubtitleManager {
    fn new(
        settings: ApplicationConfig,
        provider: Arc<dyn SubtitleProvider>,
        parsers: HashMap<SubtitleType, Box<dyn Parser>>,
    ) -> Self {
        Self {
            preference: SubtitlePreference::Disabled,
            settings,
            provider,
            parsers,
            callbacks: MultiThreadedCallback::new(),
            cancellation_token: Default::default(),
        }
    }

    /// Run the main loop of the subtitle manager.
    async fn run(&mut self, mut command_receiver: ChannelReceiver<SubtitleManagerCommand>) {
        // update the preference to the default value
        self.preference = Self::default_preference(&self.settings).await;

        loop {
            select! {
                _ = self.cancellation_token.cancelled() => break,
                Some(command) = command_receiver.recv() => self.on_command(command).await,
            }
        }

        self.on_close().await;
        debug!("Subtitle manager main loop ended");
    }

    /// Handle the given command.
    async fn on_command(&mut self, command: SubtitleManagerCommand) {
        match command {
            SubtitleManagerCommand::GetPreference { response } => {
                response.send(self.preference.clone())
            }
            SubtitleManagerCommand::GetProvider { response } => {
                response.send(self.provider.clone())
            }
            SubtitleManagerCommand::UpdatePreference {
                preference,
                response,
            } => {
                self.update_preference(preference);
                response.send(());
            }
            SubtitleManagerCommand::SelectOrDefault {
                subtitles,
                response,
            } => {
                response.send(self.select_or_default(&subtitles).await);
            }
            SubtitleManagerCommand::Download {
                subtitle,
                matcher,
                response,
            } => response.send(self.download(&subtitle, &matcher).await),
            SubtitleManagerCommand::Convert {
                subtitle,
                output_type,
                response,
            } => response.send(self.convert(subtitle, output_type)),
            SubtitleManagerCommand::Reset { response } => response.send(self.reset().await),
            SubtitleManagerCommand::DoCleanup { response } => response.send(self.cleanup().await),
        }
    }

    /// Find the subtitle for the default configured subtitle language.
    /// This uses the [SubtitleSettings::default_subtitle] setting.
    async fn find_for_default_subtitle_language(
        &self,
        subtitles: &[SubtitleInfo],
    ) -> Option<SubtitleInfo> {
        let subtitle_language = self
            .settings
            .user_settings_ref(|e| e.subtitle().default_subtitle().clone())
            .await;

        subtitles
            .iter()
            .find(|e| e.language() == &subtitle_language)
            .map(|e| e.clone())
    }

    /// Find the subtitle for the interface language.
    /// This uses the [UiSettings::default_language] setting.
    async fn find_for_interface_language(
        &self,
        subtitles: &[SubtitleInfo],
    ) -> Option<SubtitleInfo> {
        let settings = self.settings.user_settings().await;
        let language = settings.ui().default_language();

        subtitles
            .iter()
            .find(|e| &e.language().code() == language)
            .map(|e| e.clone())
    }

    fn update_preference(&mut self, preference: SubtitlePreference) {
        self.preference = preference;
    }

    async fn select_or_default(&self, subtitles: &[SubtitleInfo]) -> SubtitleInfo {
        trace!("Selecting subtitle out of {:?}", subtitles);
        let language = match &self.preference {
            SubtitlePreference::Language(language) => language,
            SubtitlePreference::Disabled => return SubtitleInfo::none(),
        };
        let subtitle = match subtitles.iter().find(|e| e.language() == language) {
            Some(info) => Some(info.clone()),
            None => {
                trace!("Subtitle preference language {} not found, using default subtitle language instead", language);
                match self.find_for_default_subtitle_language(subtitles).await {
                    None => self.find_for_interface_language(subtitles).await,
                    Some(subtitle) => Some(subtitle),
                }
            }
        };

        debug!("Selected subtitle {:?}", &subtitle);
        subtitle.unwrap_or(SubtitleInfo::none())
    }

    /// Try to download the given subtitle through the subtitle provider.
    ///
    /// If the subtitle was downloaded, try to parse it.
    async fn download(
        &self,
        subtitle: &SubtitleInfo,
        matcher: &SubtitleMatcher,
    ) -> Result<Subtitle> {
        let path = self.provider.download(subtitle, matcher).await?;
        self.parse(path.as_path(), Some(subtitle)).await
    }

    async fn parse(&self, path: &Path, info: Option<&SubtitleInfo>) -> Result<Subtitle> {
        trace!("Parsing subtitle file {}", path.to_string_lossy());
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_string())
            .ok_or_else(|| {
                SubtitleError::ParseFileError(
                    format!("{}", path.to_string_lossy()),
                    "file has no extension".to_string(),
                )
            })?;
        let subtitle_type = SubtitleType::from_extension(&extension)
            .map_err(|err| SubtitleError::ParseFileError(format!("{:?}", path), err.to_string()))?;
        let parser = self
            .parsers
            .get(&subtitle_type)
            .ok_or_else(|| SubtitleError::TypeNotSupported(subtitle_type))?;

        let file = OpenOptions::new().read(true).open(path).await?;
        let subtitle = parser.parse_file(file).await?;

        info!("Parsed subtitle file {}", path.to_string_lossy());
        Ok(Subtitle::new(
            subtitle,
            info.map(|e| e.clone()),
            path.to_string_lossy().to_string(),
        ))
    }

    fn convert(&self, subtitle: Subtitle, output_type: SubtitleType) -> Result<String> {
        match self.parsers.get(&output_type) {
            None => Err(SubtitleError::TypeNotSupported(output_type)),
            Some(parser) => {
                debug!(
                    "Converting subtitle to raw format of {} for {}",
                    &output_type, subtitle
                );
                match parser.convert(subtitle.cues()) {
                    Err(err) => {
                        error!("Subtitle parsing to raw {} failed, {}", &output_type, err);
                        Err(SubtitleError::ConversionFailed(
                            output_type.clone(),
                            err.to_string(),
                        ))
                    }
                    Ok(e) => {
                        debug!(
                            "Converted subtitle {:?} to raw {}",
                            &subtitle.file(),
                            &output_type
                        );
                        Ok(e)
                    }
                }
            }
        }
    }

    /// Reset the player to its default state for the next media playback.
    async fn reset(&mut self) {
        let preference = Self::default_preference(&self.settings).await;
        self.update_preference(preference);
        info!("Subtitle has been reset for next media playback")
    }

    /// Clean up the subtitle directory by removing all files.
    async fn cleanup(&mut self) {
        let path = self
            .settings
            .user_settings_ref(|e| e.subtitle_settings.directory())
            .await;
        let absolute_path = path.to_str().expect("expected a valid path");

        debug!("Cleaning subtitle directory {}", absolute_path);
        if let Err(e) = Storage::clean_directory(path.as_path()) {
            error!("Failed to clean subtitle directory, {}", e);
        } else {
            info!("Subtitle directory {} has been cleaned", absolute_path);
        }
    }

    async fn on_close(&mut self) {
        let auto_cleaning_enabled = self
            .settings
            .user_settings_ref(|e| e.subtitle().auto_cleaning_enabled)
            .await;
        if auto_cleaning_enabled {
            self.cleanup().await
        } else {
            trace!("Skipping subtitle directory cleaning")
        }
    }

    async fn default_preference(settings: &ApplicationConfig) -> SubtitlePreference {
        let preferred_language = settings
            .user_settings_ref(|e| e.subtitle_settings.default_subtitle.clone())
            .await;
        SubtitlePreference::Language(preferred_language)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::assert_timeout;
    use crate::core::config::{UiScale, UiSettings};
    use crate::core::media::Category;
    use crate::core::subtitles::MockSubtitleProvider;
    use crate::testing::copy_test_file;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_update_preference_language() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let preference = SubtitlePreference::Language(SubtitleLanguage::Dutch);
        let manager = subtitle_manager!(settings!(temp_path));

        manager.update_preference(preference.clone()).await;

        let result = manager.preference().await;
        assert_eq!(preference, result)
    }

    #[tokio::test]
    async fn test_update_preference_disabled() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let preference = SubtitlePreference::Disabled;
        let manager = subtitle_manager!(settings!(temp_path));

        manager.update_preference(preference.clone()).await;

        let result = manager.preference().await;
        assert_eq!(preference, result)
    }

    #[tokio::test]
    async fn test_reset() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let preference = SubtitlePreference::Language(SubtitleLanguage::Bulgarian);
        let manager = subtitle_manager!(settings!(temp_path));

        manager.update_preference(preference.clone()).await;
        manager.reset().await;

        let result = manager.preference().await;
        assert_eq!(
            SubtitlePreference::Language(SubtitleLanguage::English),
            result
        )
    }

    #[tokio::test]
    async fn test_select_or_default_select_for_default_subtitle_language() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let manager = subtitle_manager!(settings!(temp_path, true));
        let subtitle_info = SubtitleInfo::builder()
            .imdb_id("lorem")
            .language(SubtitleLanguage::English)
            .build();
        let subtitles: Vec<SubtitleInfo> = vec![subtitle_info.clone()];

        let result = manager.select_or_default(&subtitles).await;

        assert_eq!(subtitle_info, result)
    }

    #[tokio::test]
    async fn test_select_or_default_select_for_interface_language() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let settings = settings!(temp_path, true);
        settings
            .update_ui(UiSettings {
                default_language: "fr".to_string(),
                ui_scale: UiScale::new(1.0).unwrap(),
                start_screen: Category::Movies,
                maximized: false,
                native_window_enabled: false,
            })
            .await;
        let manager = subtitle_manager!(settings);
        let subtitle_info = SubtitleInfo::builder()
            .imdb_id("ipsum")
            .language(SubtitleLanguage::French)
            .build();
        let subtitles: Vec<SubtitleInfo> = vec![subtitle_info.clone()];

        let result = manager.select_or_default(&subtitles).await;

        assert_eq!(subtitle_info, result)
    }

    #[tokio::test]
    async fn test_drop_cleanup_subtitles() {
        init_logger!();
        let temp_dir = tempdir().expect("expected a tempt dir to be created");
        let temp_path = temp_dir.path().to_str().unwrap();
        let manager = subtitle_manager!(settings!(temp_path, true));
        let filepath = copy_test_file(temp_path, "example.srt", None);

        drop(manager);

        assert_timeout!(
            Duration::from_millis(250),
            !PathBuf::from(filepath.as_str()).exists(),
            "expected the file to have been removed"
        );
        assert_eq!(
            true,
            PathBuf::from(temp_path).exists(),
            "expected the subtitle directory to not have been removed"
        );
    }

    mod convert {
        use super::*;
        use crate::core::subtitles::cue::{StyledText, SubtitleCue, SubtitleLine};
        use crate::core::subtitles::parsers::{SrtParser, VttParser};
        use crate::testing::read_test_file_to_string;

        #[tokio::test]
        async fn test_convert_srt() {
            init_logger!();
            let temp_dir = tempdir().expect("expected a tempt dir to be created");
            let temp_path = temp_dir.path().to_str().unwrap();
            let subtitle = create_subtitle();
            let expected_result =
                read_test_file_to_string("example-conversion.srt").replace("\r\n", "\n");
            let settings = settings!(temp_path, true);
            let manager = SubtitleManager::builder()
                .settings(settings)
                .provider(MockSubtitleProvider::new())
                .with_parser(SubtitleType::Srt, SrtParser::default())
                .build();

            let result = manager
                .convert(subtitle, SubtitleType::Srt)
                .await
                .expect("expected the conversion to succeed");

            assert_eq!(expected_result, result);
        }

        #[tokio::test]
        async fn test_convert_vtt() {
            init_logger!();
            let temp_dir = tempdir().expect("expected a tempt dir to be created");
            let temp_path = temp_dir.path().to_str().unwrap();
            let subtitle = create_subtitle();
            let expected_result =
                read_test_file_to_string("example-conversion.vtt").replace("\r\n", "\n");
            let settings = settings!(temp_path, true);
            let manager = SubtitleManager::builder()
                .settings(settings)
                .provider(MockSubtitleProvider::new())
                .with_parser(SubtitleType::Vtt, VttParser::default())
                .build();

            let result = manager
                .convert(subtitle, SubtitleType::Vtt)
                .await
                .expect("expected the conversion to succeed");

            assert_eq!(expected_result, result);
        }

        fn create_subtitle() -> Subtitle {
            Subtitle::new(
                vec![SubtitleCue::new(
                    "1".to_string(),
                    45000,
                    46890,
                    vec![SubtitleLine::new(vec![StyledText::new(
                        "lorem".to_string(),
                        false,
                        false,
                        true,
                    )])],
                )],
                None,
                String::new(),
            )
        }
    }
}
