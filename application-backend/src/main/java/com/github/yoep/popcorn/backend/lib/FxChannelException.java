package com.github.yoep.popcorn.backend.lib;

public class FxChannelException extends RuntimeException {
    public FxChannelException(String message) {
        super(message);
    }

    public FxChannelException(String message, Throwable cause) {
        super(message, cause);
    }
}
