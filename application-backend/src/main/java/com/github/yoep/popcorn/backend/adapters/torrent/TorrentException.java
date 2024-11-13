package com.github.yoep.popcorn.backend.adapters.torrent;

import lombok.Getter;

/**
 * Exception indicating that an error occurred while handling/executing an action in the torrent adapter.
 */
@Getter
public class TorrentException extends RuntimeException {
    private final TorrentError error;
    
    public TorrentException(String message) {
        super(message);
        this.error = null;
    }

    public TorrentException(String message, Throwable cause) {
        super(message, cause);
        this.error = null;
    }

    public TorrentException(TorrentError error) {
        super(error.toString());
        this.error = error;
    }
}
