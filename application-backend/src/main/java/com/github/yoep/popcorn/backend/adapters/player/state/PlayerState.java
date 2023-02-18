package com.github.yoep.popcorn.backend.adapters.player.state;

/**
 * The state of the player.
 */
public enum PlayerState {
    UNKNOWN,
    READY,
    LOADING,
    BUFFERING,
    PLAYING,
    PAUSED,
    STOPPED,
    ERROR
}
