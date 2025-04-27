package com.github.yoep.popcorn.backend.media.watched;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.WatchedEvent;

public interface WatchedEventListener {
    void onWatchedStateChanged(WatchedEvent.WatchedStateChanged event);
}
