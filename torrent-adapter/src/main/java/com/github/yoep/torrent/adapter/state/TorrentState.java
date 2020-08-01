package com.github.yoep.torrent.adapter.state;

/**
 * The torrent state.
 */
public enum TorrentState {
    /**
     * The torrent is currently being created.
     */
    CREATING,
    /**
     * The torrent is starting the download process.
     */
    STARTING,
    /**
     * The torrent is currently paused.
     */
    PAUSED,
    /**
     * The torrent is currently downloading.
     */
    DOWNLOADING,
    /**
     * The torrent has completed the download.
     */
    COMPLETED
}
