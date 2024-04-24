use std::fmt::Debug;

use async_trait::async_trait;
use derive_more::Display;

pub use errors::*;
pub use none::*;
pub use vlc::*;

mod errors;
mod lib_vlc;
mod none;
mod vlc;

/// Represents the type of transcoding.
#[derive(Debug, Clone, PartialEq)]
pub enum TranscodeType {
    /// Transcoded media stream is buffered.
    Buffered,
    /// Transcoding media stream is live.
    Live,
}

/// The state of the transcoding process.
#[derive(Debug, Display, Clone, PartialEq)]
pub enum TranscodeState {
    /// The state of the transcoding process is unknown.
    Unknown,
    /// The transcoding process is in the preparing phase.
    Preparing,
    /// The transcoding process is starting.
    Starting,
    /// The transcoding process is ongoing.
    Transcoding,
    /// The transcoding process has stopped.
    Stopped,
    /// An error occurred during the transcoding process.
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TranscodeOutput {
    pub url: String,
    pub stream_type: TranscodeType,
}

/// A trait representing a media transcoder.
#[async_trait]
pub trait Transcoder: Debug + Sync + Send {
    /// Gets the current state of the transcoder.
    fn state(&self) -> TranscodeState;

    /// Transcodes the input media.
    ///
    /// # Arguments
    ///
    /// * `url`: The URL of the input media.
    ///
    /// # Returns
    ///
    /// A `Result` containing the URL of the transcoded media if successful, or an error if transcoding fails.
    async fn transcode(&self, url: &str) -> Result<String>;

    /// Stops the current transcoding process.
    async fn stop(&self);
}
