package com.github.yoep.popcorn.trakt.authorization;

public class AccessTokenException extends RuntimeException {
    public AccessTokenException() {
        super("Access token not available");
    }

    public AccessTokenException(String message) {
        super(message);
    }

    public AccessTokenException(String message, Throwable cause) {
        super(message, cause);
    }
}
