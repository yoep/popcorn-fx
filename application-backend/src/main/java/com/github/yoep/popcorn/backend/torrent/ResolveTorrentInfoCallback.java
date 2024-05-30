package com.github.yoep.popcorn.backend.torrent;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentInfoResult;
import com.sun.jna.Callback;

public interface ResolveTorrentInfoCallback extends Callback {
    TorrentInfoResult.ByValue callback(String url);
}
