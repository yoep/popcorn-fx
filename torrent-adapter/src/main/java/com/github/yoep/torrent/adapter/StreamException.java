package com.github.yoep.torrent.adapter;

/**
 * Exception indicating that an error occurred while streaming.
 */
public class StreamException extends RuntimeException {
    public StreamException(String message) {
        super(message);
    }

    public StreamException(String message, Throwable cause) {
        super(message, cause);
    }
}
