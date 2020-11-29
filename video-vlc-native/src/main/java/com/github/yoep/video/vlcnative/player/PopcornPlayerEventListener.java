package com.github.yoep.video.vlcnative.player;

public interface PopcornPlayerEventListener {
    /**
     * Invoked when the player state has been changed.
     *
     * @param newState The new player state value.
     */
    void onStateChanged(PopcornPlayerState newState);

    /**
     * Invoked when the time of the player has been changed.
     *
     * @param newValue The new time of the player in millis.
     */
    void onTimeChanged(long newValue);

    /**
     * Invoked when the duration of the player has been changed.
     *
     * @param newValue The new duration of the player in millis.
     */
    void onDurationChanged(long newValue);
}
