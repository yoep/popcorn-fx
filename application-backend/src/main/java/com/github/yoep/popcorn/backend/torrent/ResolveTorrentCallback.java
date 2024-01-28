package com.github.yoep.popcorn.backend.torrent;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentFileInfoWrapper;
import com.sun.jna.Callback;

public interface ResolveTorrentCallback extends Callback {
    TorrentWrapper.ByValue callback(TorrentFileInfoWrapper.ByValue fileInfo, String torrentDirectory, byte autoStartDownload);
}
