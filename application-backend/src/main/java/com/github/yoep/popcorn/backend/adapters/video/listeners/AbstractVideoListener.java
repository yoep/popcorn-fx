package com.github.yoep.popcorn.backend.adapters.video.listeners;

import com.github.yoep.popcorn.backend.adapters.video.state.VideoState;

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
