package com.github.yoep.popcorn.backend.torrent;

import com.sun.jna.Callback;

public interface CancelTorrentCallback extends Callback {
    void callback(String handle);
}
