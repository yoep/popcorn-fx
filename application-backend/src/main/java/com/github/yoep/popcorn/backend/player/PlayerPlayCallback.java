package com.github.yoep.popcorn.backend.player;

import com.sun.jna.Callback;

public interface PlayerPlayCallback extends Callback {
    void callback(PlayRequestWrapper request);
}
