package com.github.yoep.popcorn.backend.adapters.player.listeners;


import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;

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
    public void onStateChanged(Player.State newState) {
        // no-op
    }

    @Override
    public void onVolumeChanged(int volume) {
        // no-op
    }
}
