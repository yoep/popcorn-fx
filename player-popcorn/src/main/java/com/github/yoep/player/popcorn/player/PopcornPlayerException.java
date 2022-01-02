package com.github.yoep.player.popcorn.player;

import lombok.Getter;

@Getter
public class PopcornPlayerException extends RuntimeException {
    private final String url;

    public PopcornPlayerException( String url, String message, Throwable cause) {
        super(message, cause);
        this.url = url;
    }
}
