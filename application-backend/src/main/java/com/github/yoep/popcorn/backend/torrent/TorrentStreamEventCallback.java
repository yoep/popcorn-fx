package com.github.yoep.popcorn.backend.torrent;

import com.sun.jna.Callback;

public interface TorrentStreamEventCallback extends Callback {
    void callback(TorrentStreamEventC.ByValue event);
}
