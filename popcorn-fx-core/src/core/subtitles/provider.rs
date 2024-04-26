use std::fmt::Debug;
use std::path::Path;

use async_trait::async_trait;
#[cfg(any(test, feature = "testing"))]
use mockall::automock;

use crate::core::media::{Episode, MovieDetails, ShowDetails};
use crate::core::subtitles;
use crate::core::subtitles::matcher::SubtitleMatcher;
use crate::core::subtitles::model::{Subtitle, SubtitleInfo, SubtitleType};

/// The subtitle provider is responsible for discovering & downloading of [Subtitle] files
/// for [Media] items.
#[cfg_attr(any(test, feature = "testing"), automock)]
#[async_trait]
pub trait SubtitleProvider: Debug + Send + Sync {
    /// The available default subtitle options.
    fn default_subtitle_options(&self) -> Vec<SubtitleInfo> {
        vec![SubtitleInfo::none(), SubtitleInfo::custom()]
    }

    /// Retrieve the available subtitles for the given movie.
    async fn movie_subtitles(&self, media: &MovieDetails) -> subtitles::Result<Vec<SubtitleInfo>>;

    /// Retrieve the available subtitles for the given episode.
    async fn episode_subtitles(
        &self,
        media: &ShowDetails,
        episode: &Episode,
    ) -> subtitles::Result<Vec<SubtitleInfo>>;

    /// Retrieve the available subtitles for the given filename.
    async fn file_subtitles(&self, filename: &str) -> subtitles::Result<Vec<SubtitleInfo>>;

    /// Download the subtitle for the given [SubtitleInfo].
    ///
    /// It returns the location the downloaded subtitle file on success, else the [subtitles::SubtitleError].
    async fn download(
        &self,
        subtitle_info: &SubtitleInfo,
        matcher: &SubtitleMatcher,
    ) -> subtitles::Result<String>;

    /// Download the subtitle for the given [SubtitleInfo].
    /// This method automatically parses the downloaded file.
    ///
    /// It returns the parsed [Subtitle] on success, else the [subtitles::SubtitleError].
    async fn download_and_parse(
        &self,
        subtitle_info: &SubtitleInfo,
        matcher: &SubtitleMatcher,
    ) -> subtitles::Result<Subtitle>;

    /// Parse the given file path to a subtitle struct.
    ///
    /// It returns a [SubtitleError] when the path doesn't exist of the file failed to be parsed.
    fn parse(&self, file_path: &Path) -> subtitles::Result<Subtitle>;

    /// Convert the given [Subtitle] back to a raw format of [SubtitleType].
    /// It returns the raw format string for the given type on success, else the error.
    fn convert(&self, subtitle: Subtitle, output_type: SubtitleType) -> subtitles::Result<String>;
}
