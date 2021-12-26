package com.github.yoep.popcorn.backend.subtitles;

public class SubtitleException extends RuntimeException {
    public SubtitleException(String message) {
        super(message);
    }

    public SubtitleException(String message, Throwable cause) {
        super(message, cause);
    }
}
