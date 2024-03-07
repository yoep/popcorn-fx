package com.github.yoep.popcorn.backend.player;

import com.sun.jna.Callback;

public interface PlayerManagerCallback extends Callback {
    void callback(PlayerManagerEvent.ByValue event);
}
