package com.github.yoep.popcorn.backend.media.tracking;

import com.github.yoep.popcorn.backend.services.ListenerService;

public interface TrackingService extends ListenerService<TrackingListener> {
    boolean isAuthorized();

    void authorize();

    void disconnect();
}
