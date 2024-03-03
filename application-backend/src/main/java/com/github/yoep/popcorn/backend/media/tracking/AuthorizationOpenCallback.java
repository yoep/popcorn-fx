package com.github.yoep.popcorn.backend.media.tracking;

import com.sun.jna.Callback;

public interface AuthorizationOpenCallback extends Callback {
    byte callback(String uri);
}
