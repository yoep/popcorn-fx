package com.github.yoep.popcorn.backend.player;

import com.sun.jna.Callback;

public interface PlayerSeekCallback extends Callback {
    void callback(Long time);
}
