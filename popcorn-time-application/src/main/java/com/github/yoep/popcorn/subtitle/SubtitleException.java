package com.github.yoep.popcorn.subtitle;

public class SubtitleException extends RuntimeException {
    public SubtitleException(String message) {
        super(message);
    }

    public SubtitleException(String message, Throwable cause) {
        super(message, cause);
    }
}
