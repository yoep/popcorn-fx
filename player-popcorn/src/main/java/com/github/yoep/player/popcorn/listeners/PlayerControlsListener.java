package com.github.yoep.player.popcorn.listeners;

public interface PlayerControlsListener {
    void onFullscreenStateChanged(boolean isFullscreenEnabled);

    void onSubtitleStateChanged(boolean isSubtitlesEnabled);
}
