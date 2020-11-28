package com.github.yoep.video.vlcnative.player;

public class PopcornPlayerException extends RuntimeException {
    public PopcornPlayerException(String message) {
        super(message);
    }

    public PopcornPlayerException(String message, Throwable cause) {
        super(message, cause);
    }
}
