use std::path::Path;

use async_trait::async_trait;

use crate::core::media::model::{Episode, Movie, Show};
use crate::core::subtitles::errors::SubtitleError;
use crate::core::subtitles::matcher::SubtitleMatcher;
use crate::core::subtitles::model::{Subtitle, SubtitleInfo, SubtitleType};
use crate::observer::Observer;

/// The specialized subtitle result.
pub type Result<T> = std::result::Result<T, SubtitleError>;

/// The [Observer] for the [SubtitleService].
pub trait SubtitleServiceObserver: Observer {
    /// Invoked when the subtitle is changed.
    fn on_subtitle_changed(&mut self, old_value: Option<&Subtitle>, new_value: Option<&Subtitle>);
}

/// The subtitle service is responsible for discovering & downloading of subtitle files for Media
/// items.
#[async_trait]
pub trait SubtitleService {
    /// The active subtitle for the media playback.
    fn active_subtitle(&self) -> Option<&Subtitle>;

    /// The subtitle which should be activated for the media playback.
    /// The [SubtitleService] will always take ownership of the [Subtitle] and
    /// will return read-only references through [Self::active_subtitle()].
    fn update_active_subtitle(&mut self, subtitle: Option<Subtitle>);

    /// The available default subtitle options.
    fn default_subtitle_options(&self) -> Vec<SubtitleInfo> {
        vec![SubtitleInfo::none(), SubtitleInfo::custom()]
    }

    /// Retrieve the available subtitles for the given movie.
    async fn movie_subtitles(&self, media: Movie) -> Result<Vec<SubtitleInfo>>;

    /// Retrieve the available subtitles for the given episode.
    async fn episode_subtitles(&self, media: Show, episode: Episode) -> Result<Vec<SubtitleInfo>>;

    /// Retrieve the available subtitles for the given filename.
    async fn file_subtitles(&self, filename: &String) -> Result<Vec<SubtitleInfo>>;

    /// Download the [Subtitle] for the given [SubtitleInfo].
    /// This method automatically parses the downloaded file.
    async fn download(&self, subtitle_info: &SubtitleInfo, matcher: &SubtitleMatcher) -> Result<Subtitle>;

    /// Parse the given file path to a subtitle struct.
    /// It returns a [SubtitleError] when the path doesn't exist of the file failed to be parsed.
    fn parse(&self, file_path: &Path) -> Result<Subtitle>;

    /// Select one of the available subtitles.
    /// It returns the default [SubtitleInfo::none] when the preferred subtitle is not present.
    fn select_or_default(&self, subtitles: &Vec<SubtitleInfo>) -> SubtitleInfo;
    
    /// Convert the given [Subtitle] back to a raw format of [SubtitleType].
    /// It returns the raw format string for the given type on success, else the error.
    fn convert(&self, subtitle: Subtitle, output_type: SubtitleType) -> Result<String>;
}