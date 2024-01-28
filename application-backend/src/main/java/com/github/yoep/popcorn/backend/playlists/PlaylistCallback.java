package com.github.yoep.popcorn.backend.playlists;

import javax.security.auth.callback.Callback;

public interface PlaylistCallback extends Callback {
    void callback(PlaylistEvent event);
}
