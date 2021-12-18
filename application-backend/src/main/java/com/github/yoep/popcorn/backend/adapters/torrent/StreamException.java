package com.github.yoep.popcorn.backend.adapters.torrent;

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
