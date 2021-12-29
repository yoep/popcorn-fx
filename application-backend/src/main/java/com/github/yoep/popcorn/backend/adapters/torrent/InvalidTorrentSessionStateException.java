package com.github.yoep.popcorn.backend.adapters.torrent;

import com.github.yoep.popcorn.backend.adapters.torrent.state.SessionState;

/**
 * Exception indicating that the torrent session is in an invalid state.
 */
public class InvalidTorrentSessionStateException extends TorrentException {
    private final SessionState state;

    public InvalidTorrentSessionStateException(SessionState state) {
        super("Torrent session is in an invalid state, state is " + state);
        this.state = state;
    }

    /**
     * Get the invalid session state.
     *
     * @return Returns the invalid session state.
     */
    public SessionState getState() {
        return state;
    }
}
