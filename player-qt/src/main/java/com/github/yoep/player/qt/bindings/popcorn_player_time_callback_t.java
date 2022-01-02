package com.github.yoep.player.qt.bindings;

import com.sun.jna.Callback;

public interface popcorn_player_time_callback_t extends Callback {
    /**
     * The current playback time callback from the player.
     * This value is originally a long, but for some strange reason the long is incorrectly passed along as -829894590195142xxxx on ARM.
     * To work around this issue, the long is converted to a {@code char *} which then needs to be converted back to a {@code long}.
     *
     * @param newValue The new duration value.
     */
    void callback(String newValue);
}
