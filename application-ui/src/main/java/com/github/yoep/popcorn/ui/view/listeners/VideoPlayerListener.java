package com.github.yoep.popcorn.ui.view.listeners;

import com.github.yoep.video.adapter.state.PlayerState;

public interface VideoPlayerListener {
    /**
     * Invoked when the player state changes.
     *
     * @param newState The new player state.
     */
    void onPlayerStateChanged(PlayerState newState);

    /**
     * Invoked when the player time is changed.
     *
     * @param newValue The new time value of the player.
     */
    void onTimeChanged(Number newValue);

    /**
     * Invoked when the player duration is changed.
     *
     * @param newValue The new duration value of the player.
     */
    void onDurationChanged(Number newValue);
}
