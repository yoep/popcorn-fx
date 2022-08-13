package com.github.yoep.popcorn.backend.adapters.torrent;

public class InvalidTorrentStreamException extends TorrentStreamException {
    public InvalidTorrentStreamException(String message) {
        super(message);
    }

    public InvalidTorrentStreamException(String message, Throwable cause) {
        super(message, cause);
    }
}
