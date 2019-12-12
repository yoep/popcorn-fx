package com.github.yoep.popcorn.torrent;

public class TorrentException extends RuntimeException {
    public TorrentException(String message) {
        super(message);
    }

    public TorrentException(String message, Throwable cause) {
        super(message, cause);
    }
}
