package com.github.yoep.popcorn.backend.adapters.player.listeners;

import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;

/**
 * The player listener triggers events when the player/playback is changed.
 * These events include the state, duration, time, etc.
 */
public interface PlayerListener {
    /**
     * Invoked when the playback duration is changed.
     *
     * @param newDuration The new duration of the playback in millis.
     */
    void onDurationChanged(long newDuration);

    /**
     * Invoked when the playback time is changed.
     *
     * @param newTime The new time of the playback in millis.
     */
    void onTimeChanged(long newTime);

    /**
     * Invoked when the player state is changed.
     *
     * @param newState The new state of the player.
     */
    void onStateChanged(PlayerState newState);

    /**
     * Invoked when the player volume is changed.
     *
     * @param volume The volume level between 0 and 100.
     */
    void onVolumeChanged(int volume);
}
