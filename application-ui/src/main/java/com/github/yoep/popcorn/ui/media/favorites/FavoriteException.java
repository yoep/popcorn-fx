package com.github.yoep.popcorn.ui.media.favorites;

public class FavoriteException extends RuntimeException {
    public FavoriteException(String message) {
        super(message);
    }

    public FavoriteException(String message, Throwable cause) {
        super(message, cause);
    }
}
