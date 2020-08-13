package com.github.yoep.popcorn.ui.view.listeners;

import com.github.yoep.video.adapter.state.PlayerState;

public abstract class AbstractVideoPlayerListener implements VideoPlayerListener {
    @Override
    public void onPlayerStateChanged(PlayerState newState) {
        //no-op
    }

    @Override
    public void onTimeChanged(Number newValue) {
        //no-op
    }

    @Override
    public void onDurationChanged(Number newValue) {
        //no-op
    }
}
