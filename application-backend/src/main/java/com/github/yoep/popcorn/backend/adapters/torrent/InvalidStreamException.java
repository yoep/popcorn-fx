package com.github.yoep.popcorn.backend.adapters.torrent;

public class InvalidStreamException extends StreamException {
    public InvalidStreamException(String message) {
        super(message);
    }

    public InvalidStreamException(String message, Throwable cause) {
        super(message, cause);
    }
}
