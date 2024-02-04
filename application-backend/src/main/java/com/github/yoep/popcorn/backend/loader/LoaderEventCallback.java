package com.github.yoep.popcorn.backend.loader;

import com.sun.jna.Callback;

public interface LoaderEventCallback extends Callback {
    void callback(LoaderEventC.ByValue event);
}
