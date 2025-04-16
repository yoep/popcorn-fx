package com.github.yoep.player.popcorn.listeners;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;

public abstract class AbstractPlaybackListener implements PlaybackListener{
    @Override
    public void onPlay(Player.PlayRequest request) {
        // no-op
    }

    @Override
    public void onResume() {
        // no-op
    }

    @Override
    public void onPause() {
        // no-op
    }

    @Override
    public void onSeek(long time) {
        // no-op
    }

    @Override
    public void onVolume(int volume) {
        // no-op
    }

    @Override
    public void onStop() {
        // no-op
    }
}
