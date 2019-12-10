package com.github.yoep.popcorn.media.video.state;

public interface PlayerStateListener {
    /**
     * Invoked when the video player state changes.
     *
     * @param oldState The old player state.
     * @param newState The new player state.
     */
    void onChange(PlayerState oldState, PlayerState newState);
}
