package com.github.yoep.player.adapter.state;

/**
 * The state of the player.
 */
public enum PlayerState {
    UNKNOWN,
    LOADING,
    BUFFERING,
    PLAYING,
    PAUSED,
    STOPPED,
    ERROR
}
