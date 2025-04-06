package com.github.yoep.popcorn.backend.torrent;

import com.sun.jna.Callback;

public interface TorrentEventCallback extends Callback {
    void callback(TorrentEventC.ByValue event);
}
