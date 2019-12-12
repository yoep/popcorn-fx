package com.github.yoep.popcorn.torrent.listeners;

import com.github.yoep.popcorn.torrent.Torrent;

public interface TorrentCreationListener {
    /**
     * Invoked when a new torrent has been created.
     *
     * @param torrent The torrent that has been created.
     */
    void onTorrentCreated(Torrent torrent);
}
