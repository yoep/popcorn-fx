package com.github.yoep.popcorn.backend.adapters.torrent;

import com.github.yoep.popcorn.backend.lib.Handle;

public interface TorrentService {
    void addListener(Handle handle, TorrentListener listener);

    /**
     * Clean the torrents directory.
     * This will remove all torrents from the system.
     */
    void cleanup();
}
