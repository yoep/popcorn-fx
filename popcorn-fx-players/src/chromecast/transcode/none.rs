use async_trait::async_trait;

use crate::chromecast::transcode;
use crate::chromecast::transcode::{TranscodeError, TranscodeOutput, Transcoder, TranscodeState};

/// A no-operation transcoder implementation.
#[derive(Debug)]
pub struct NoOpTranscoder;

#[async_trait]
impl Transcoder for NoOpTranscoder {
    /// Gets the current state of the transcoder.
    fn state(&self) -> TranscodeState {
        TranscodeState::Stopped
    }

    /// Transcodes the input media.
    ///
    /// This method always returns an error indicating that transcoding is unsupported.
    ///
    /// # Arguments
    ///
    /// * `_input`: A reference to the input media.
    ///
    /// # Returns
    ///
    /// An error indicating that transcoding is unsupported.
    async fn transcode(&self, _input: &str) -> transcode::Result<TranscodeOutput> {
        Err(TranscodeError::Unsupported)
    }

    /// Stops the transcoding process.
    ///
    /// This method does nothing as there is no transcoding process to stop.
    async fn stop(&self) {
        // no-op
    }
}

#[cfg(test)]
mod tests {
    use popcorn_fx_core::core::block_in_place;

    use super::*;

    #[test]
    fn test_state() {
        let transcoder = NoOpTranscoder {};

        let result = transcoder.state();

        assert_eq!(TranscodeState::Stopped, result);
    }

    #[test]
    fn test_transcode() {
        let transcoder = NoOpTranscoder {};

        let result = block_in_place(transcoder.transcode("http://localhost/my-video.mp4"));

        assert_eq!(Err(TranscodeError::Unsupported), result);
    }

    #[test]
    fn test_stop() {
        let transcoder = NoOpTranscoder {};

        block_in_place(transcoder.stop());

        assert_eq!(TranscodeState::Stopped, transcoder.state());
    }
}
