package com.github.yoep.popcorn.backend.media.tracking;

public interface TrackingAuthorization {
    /**
     * Open the given tracking authorization url for the tracking provider.
     * This lets the user authorize the tracking provider.
     *
     * @param authorizationUri The authorization uri to open.
     */
    void open(String authorizationUri);
}
