package com.github.yoep.popcorn.backend.adapters.torrent;

/**
 * Indicates that the torrent stream couldn't be prepared.
 */
public class FailedToPrepareTorrentStreamException extends TorrentStreamException {
    public FailedToPrepareTorrentStreamException(String message) {
        super(message);
    }

    public FailedToPrepareTorrentStreamException(String message, Throwable cause) {
        super(message, cause);
    }
}
