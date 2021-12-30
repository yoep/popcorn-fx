package com.github.yoep.player.popcorn.listeners;

import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;

public interface PlayerControlsListener {
    void onFullscreenStateChanged(boolean isFullscreenEnabled);

    void onSubtitleStateChanged(boolean isSubtitlesEnabled);

    void onPlayerStateChanged(PlayerState state);

    void onPlayerTimeChanged(long time);

    void onPlayerDurationChanged(long duration);
}
