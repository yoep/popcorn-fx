package com.github.yoep.popcorn.backend.updater;


import com.sun.jna.Callback;

public interface UpdateCallback extends Callback {
    void callback(UpdateCallbackEvent.ByValue event);
}
