package com.github.yoep.popcorn.backend.controls;

import com.sun.jna.Callback;

public interface PlaybackControlCallback extends Callback {
    /**
     * Invoked when a playback control event is invoked on the backend.
     * This is most of the a media control event from the system.
     */
    void callback(PlaybackControlEvent event);
}
