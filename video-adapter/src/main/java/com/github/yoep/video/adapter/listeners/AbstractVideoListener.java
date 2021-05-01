package com.github.yoep.video.adapter.listeners;

import com.github.yoep.video.adapter.state.VideoState;

/**
 * Abstract no-op implementation of the {@link VideoListener}.
 */
public abstract class AbstractVideoListener implements VideoListener {
    @Override
    public void onDurationChanged(long newDuration) {
        // no-op
    }

    @Override
    public void onTimeChanged(long newTime) {
        // no-op
    }

    @Override
    public void onStateChanged(VideoState newState) {
        // no-op
    }
}
