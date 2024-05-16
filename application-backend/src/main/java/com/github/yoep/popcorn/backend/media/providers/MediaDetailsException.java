package com.github.yoep.popcorn.backend.media.providers;

public class MediaDetailsException extends MediaException {
    public MediaDetailsException(Media media, String message, Throwable cause) {
        super(media, message, cause);
    }
}
