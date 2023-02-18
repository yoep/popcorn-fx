package com.github.yoep.popcorn.backend.adapters.torrent.state;

/**
 * The torrent session state.
 */
public enum SessionState {
    /**
     * The torrent session is being created and not ready for use.
     */
    CREATING,
    /**
     * The torrent session is being initialized and not ready for use.
     */
    INITIALIZING,
    /**
     * The torrent session is running and ready for use.
     */
    RUNNING,
    /**
     * The torrent session encountered an error and was unable to start correctly.
     */
    ERROR
}
