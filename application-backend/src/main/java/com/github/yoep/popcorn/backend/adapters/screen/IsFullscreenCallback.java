package com.github.yoep.popcorn.backend.adapters.screen;

import com.sun.jna.Callback;

public interface IsFullscreenCallback extends Callback {
    byte callback();
}
