package com.github.yoep.popcorn.backend.media.tracking;

public interface TrackingListener {
    void onAuthorizationChanged(boolean isAuthorized);
}
