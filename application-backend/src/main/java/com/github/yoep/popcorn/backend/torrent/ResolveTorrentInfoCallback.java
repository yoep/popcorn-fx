package com.github.yoep.popcorn.backend.torrent;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentInfoWrapper;
import com.sun.jna.Callback;

public interface ResolveTorrentInfoCallback extends Callback {
    TorrentInfoWrapper.ByValue callback(String url);
}
