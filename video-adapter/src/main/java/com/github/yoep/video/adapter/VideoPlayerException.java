package com.github.yoep.video.adapter;

public class VideoPlayerException extends RuntimeException {
    public VideoPlayerException(String message) {
        super(message);
    }

    public VideoPlayerException(String message, Throwable cause) {
        super(message, cause);
    }
}
