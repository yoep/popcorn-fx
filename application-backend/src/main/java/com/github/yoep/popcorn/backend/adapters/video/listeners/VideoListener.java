package com.github.yoep.popcorn.backend.adapters.video.listeners;

import com.github.yoep.popcorn.backend.adapters.video.state.VideoState;

/**
 * The video listener triggers events for the video playback.
 */
public interface VideoListener {
    /**
     * Invoked when the video duration is changed.
     *
     * @param newDuration The new duration of the playback in millis.
     */
    void onDurationChanged(long newDuration);

    /**
     * Invoked when the video time is changed.
     *
     * @param newTime The new time of the playback in millis.
     */
    void onTimeChanged(long newTime);

    /**
     * Invoked when the video playback state is changed.
     *
     * @param newState The new state of the video playback.
     */
    void onStateChanged(VideoState newState);
}
