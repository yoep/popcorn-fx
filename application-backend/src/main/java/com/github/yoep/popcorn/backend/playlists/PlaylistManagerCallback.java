package com.github.yoep.popcorn.backend.playlists;

import com.sun.jna.Callback;

public interface PlaylistManagerCallback extends Callback {
    void callback(PlaylistManagerEvent.ByValue event);
}
