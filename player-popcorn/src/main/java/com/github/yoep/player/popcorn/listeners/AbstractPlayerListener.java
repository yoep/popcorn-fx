package com.github.yoep.player.popcorn.listeners;

import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;

public abstract class AbstractPlayerListener implements PlayerListener {
    @Override
    public void onDurationChanged(long newDuration) {
        // no-op
    }

    @Override
    public void onTimeChanged(long newTime) {
        // no-op
    }

    @Override
    public void onStateChanged(PlayerState newState) {
        // no-op
    }
}
