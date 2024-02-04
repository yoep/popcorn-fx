package com.github.yoep.popcorn.backend.player;

import com.sun.jna.Callback;

public interface PlayerStopCallback extends Callback {
    void callback();
}
