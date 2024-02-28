package com.github.yoep.popcorn.backend.media.tracking;

import com.sun.jna.Callback;

public interface TrackingEventCallback extends Callback {
    void callback(TrackingEventC.ByValue event);
}
