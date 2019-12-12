package com.github.yoep.popcorn.media.providers;

import com.github.yoep.popcorn.media.providers.models.Media;
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
