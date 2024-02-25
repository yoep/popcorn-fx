package com.github.yoep.popcorn.backend.tracking;

import com.sun.jna.Callback;

public interface AuthorizationOpenCallback extends Callback {
    byte callback(String uri);
}
