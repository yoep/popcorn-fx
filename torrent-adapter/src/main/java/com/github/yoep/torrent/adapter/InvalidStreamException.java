package com.github.yoep.torrent.adapter;

public class InvalidStreamException extends StreamException {
    public InvalidStreamException(String message) {
        super(message);
    }

    public InvalidStreamException(String message, Throwable cause) {
        super(message, cause);
    }
}
