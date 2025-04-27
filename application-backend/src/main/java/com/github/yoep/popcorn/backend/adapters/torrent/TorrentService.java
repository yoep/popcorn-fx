package com.github.yoep.popcorn.backend.adapters.torrent;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Handle;

public interface TorrentService {
    /**
     * Add the given listener to the torrent service.
     *
     * @param handle The handle to listen on.
     * @param listener The torrent listener to invoke.
     */
    void addListener(Handle handle, TorrentListener listener);

    /**
     * Remove the given listener from the service.
     *
     * @param handle The handle to remove.
     * @param listener The torrent listener to remove.
     */
    void removeListener(Handle handle, TorrentListener listener);

    /**
     * Clean the torrents directory.
     * This will remove all torrents from the system.
     */
    void cleanup();
}
