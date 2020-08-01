package com.github.yoep.torrent.adapter.model;

import com.github.yoep.torrent.adapter.state.TorrentHealthState;

public interface TorrentHealth {
    /**
     * Get the health state of the torrent.
     *
     * @return Returns the health state.
     */
    TorrentHealthState getState();

    /**
     * Get the health ration of the torrent.
     *
     * @return Return the health ration.
     */
    double getRatio();

    /**
     * Get number of seeds of the torrent.
     *
     * @return Returns the number of seeds.
     */
    int getSeeds();

    /**
     * Get the peers of the torrent.
     *
     * @return Returns the peers of the torrent.
     */
    int getPeers();
}
