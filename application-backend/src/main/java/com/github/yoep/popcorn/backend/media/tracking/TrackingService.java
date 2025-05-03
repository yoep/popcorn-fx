package com.github.yoep.popcorn.backend.media.tracking;

import com.github.yoep.popcorn.backend.services.ListenerService;

import java.util.concurrent.CompletableFuture;

public interface TrackingService extends ListenerService<TrackingListener> {
    CompletableFuture<Boolean> isAuthorized();

    void authorize();

    void disconnect();
}
