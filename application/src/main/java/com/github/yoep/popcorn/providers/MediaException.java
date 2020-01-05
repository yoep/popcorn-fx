package com.github.yoep.popcorn.providers;

import com.github.yoep.popcorn.providers.models.Media;
import lombok.Getter;

@Getter
public class MediaException extends RuntimeException {
    private final Media media;

    public MediaException(Media media, String message) {
        super(message);
        this.media = media;
    }

    public MediaException(Media media, String message, Throwable cause) {
        super(message, cause);
        this.media = media;
    }
}
