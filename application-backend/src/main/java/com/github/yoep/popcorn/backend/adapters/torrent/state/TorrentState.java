package com.github.yoep.popcorn.backend.adapters.torrent.state;

/**
 * The torrent state.
 */
public enum TorrentState {
    /**
     * The torrent is currently being created.
     */
    CREATING,
    /**
     * The torrent is ready to start the download process.
     */
    READY,
    /**
     * The torrent is starting the download process.
     */
    STARTING,
    /**
     * The torrent is currently downloading.
     */
    DOWNLOADING,
    /**
     * The torrent is currently paused.
     */
    PAUSED,
    /**
     * The torrent has completed the download.
     */
    COMPLETED,
    /**
     * The torrent encountered a fatal error and cannot continue.
     * This state is mostly encountered during the creation/start of the torrent.
     */
    ERROR
}
