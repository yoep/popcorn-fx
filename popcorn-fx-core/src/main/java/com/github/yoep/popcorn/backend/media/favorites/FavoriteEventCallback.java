package com.github.yoep.popcorn.backend.media.favorites;

import com.sun.jna.Callback;

public interface FavoriteEventCallback extends Callback {
    void callback(FavoriteEvent.ByValue event);
}
