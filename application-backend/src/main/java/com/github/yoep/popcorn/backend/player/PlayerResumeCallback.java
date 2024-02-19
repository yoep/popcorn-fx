package com.github.yoep.popcorn.backend.player;

import com.sun.jna.Callback;

public interface PlayerResumeCallback extends Callback {
    void callback();
}
