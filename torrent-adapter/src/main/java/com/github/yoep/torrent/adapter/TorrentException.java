package com.github.yoep.torrent.adapter;

/**
 * Exception indicating that an error occurred while handling/executing an action in the torrent adapter.
 */
public class TorrentException extends RuntimeException {
    public TorrentException(String message) {
        super(message);
    }

    public TorrentException(String message, Throwable cause) {
        super(message, cause);
    }
}
