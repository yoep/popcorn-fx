package com.github.yoep.popcorn.trakt;

public class TraktException extends RuntimeException {
    public TraktException(String message) {
        super(message);
    }

    public TraktException(String message, Throwable cause) {
        super(message, cause);
    }
}
