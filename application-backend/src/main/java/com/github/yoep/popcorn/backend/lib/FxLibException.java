package com.github.yoep.popcorn.backend.lib;

public class FxLibException extends RuntimeException {
    public FxLibException(String message) {
        super(message);
    }

    public FxLibException(String message, Throwable cause) {
        super(message, cause);
    }
}
