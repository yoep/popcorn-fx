use crate::popcorn::fx::media::media::MediaControl;

/// The media control implementation for windows.
pub struct MediaControlWin {

}

impl MediaControlWin {
    /// Create a new instance of the media control for windows.
    pub fn new() -> MediaControlWin {
        MediaControlWin{}
    }
}

impl MediaControl for MediaControlWin {

}