package com.github.yoep.popcorn.backend.player;

import com.sun.jna.Callback;

public interface PlayerPauseCallback extends Callback {
    void callback();
}
