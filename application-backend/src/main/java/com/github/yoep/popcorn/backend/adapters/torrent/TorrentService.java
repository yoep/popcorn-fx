package com.github.yoep.popcorn.backend.adapters.torrent;

import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.lib.Handle;

/**
 * The {@link TorrentService} manages the {@link Torrent}'s and creation of them.
 * Use this service to resolve magnet url's or start downloading a torrent.
 */
public interface TorrentService {
    Handle addListener(Handle handle, TorrentStreamListener listener);
    
    void removeListener(Handle callbackHandle);

    /**
     * Clean the torrents directory.
     * This will remove all torrents from the system.
     */
    void cleanup();
}
