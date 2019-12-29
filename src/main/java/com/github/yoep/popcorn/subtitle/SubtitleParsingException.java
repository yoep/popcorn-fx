package com.github.yoep.popcorn.subtitle;

public class SubtitleParsingException extends SubtitleException {
    public SubtitleParsingException(String message) {
        super(message);
    }

    public SubtitleParsingException(String message, Throwable cause) {
        super(message, cause);
    }
}
