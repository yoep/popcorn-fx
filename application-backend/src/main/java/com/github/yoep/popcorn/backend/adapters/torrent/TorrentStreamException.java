package com.github.yoep.popcorn.backend.adapters.torrent;

/**
 * Exception indicating that an error occurred while streaming.
 */
public class TorrentStreamException extends TorrentException {
    public TorrentStreamException(String message) {
        super(message);
    }

    public TorrentStreamException(String message, Throwable cause) {
        super(message, cause);
    }
}
