package com.github.yoep.popcorn.backend.media.watched;

import com.sun.jna.Callback;

public interface WatchedEventCallback extends Callback {
    void callback(WatchedEvent.ByValue event);
}
