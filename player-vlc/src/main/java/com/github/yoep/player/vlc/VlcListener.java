package com.github.yoep.player.vlc;

import com.github.yoep.player.vlc.model.VlcState;

public interface VlcListener {
    /**
     * Invoked when the time of the VLC player has changed.
     *
     * @param time The new time of the VLC playback.
     */
    void onTimeChanged(Long time);

    /**
     * Invoked when the duration of the VLC playback is changed.
     *
     * @param duration The new duration of the VLC playback.
     */
    void onDurationChanged(Long duration);

    /**
     * Invoked when the VLC playback state is changed.
     *
     * @param state The new VLC playback state.
     */
    void onStateChanged(VlcState state);
}
