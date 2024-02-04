package com.github.yoep.popcorn.backend.events;

import com.sun.jna.Callback;

public interface EventBridgeCallback extends Callback {
    void callback(EventC.ByValue event);
}
