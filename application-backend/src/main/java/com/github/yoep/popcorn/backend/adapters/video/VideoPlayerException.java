package com.github.yoep.popcorn.backend.adapters.video;

public class VideoPlayerException extends RuntimeException {
    public VideoPlayerException(String message) {
        super(message);
    }

    public VideoPlayerException(String message, Throwable cause) {
        super(message, cause);
    }
}
