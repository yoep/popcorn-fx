package com.github.yoep.video.vlcnative.bindings;

import com.sun.jna.Callback;

public interface popcorn_player_time_callback_t extends Callback {
    void callback(long newValue);
}
