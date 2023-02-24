package com.github.yoep.popcorn.backend.settings;

import com.sun.jna.Callback;

public interface ApplicationConfigEventCallback extends Callback {
    void callback(ApplicationConfigEvent.ByValue event);
}
