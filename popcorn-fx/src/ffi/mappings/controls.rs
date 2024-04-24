use popcorn_fx_core::core::playback::PlaybackControlEvent;

/// The C compatible callback for playback control events.
pub type PlaybackControlsCallbackC = extern "C" fn(PlaybackControlEvent);
