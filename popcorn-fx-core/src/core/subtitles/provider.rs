use crate::core::media::{Episode, MovieDetails, ShowDetails};
use crate::core::subtitles;
use crate::core::subtitles::matcher::SubtitleMatcher;
use crate::core::subtitles::model::SubtitleInfo;
use async_trait::async_trait;
#[cfg(any(test, feature = "testing"))]
use mockall::automock;
use std::fmt::Debug;
use std::path::PathBuf;

/// The subtitle provider is responsible for discovering & downloading of [Subtitle] files
/// for [Media] items.
#[cfg_attr(any(test, feature = "testing"), automock)]
#[async_trait]
pub trait SubtitleProvider: Debug + Send + Sync {
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
    ) -> subtitles::Result<PathBuf>;
}
