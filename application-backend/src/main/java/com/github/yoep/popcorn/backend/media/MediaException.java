package com.github.yoep.popcorn.backend.media;

import lombok.EqualsAndHashCode;
import lombok.ToString;

import java.util.Optional;

@ToString
@EqualsAndHashCode(callSuper = true)
public class MediaException extends RuntimeException {
    private final Media media;

    public MediaException(String message) {
        super(message);
        this.media = null;
    }

    public MediaException(Media media, String message) {
        super(message);
        this.media = media;
    }

    public MediaException(Media media, String message, Throwable cause) {
        super(message, cause);
        this.media = media;
    }

    protected MediaException(String message, Throwable cause) {
        super(message, cause);
        this.media = null;
    }

    public Optional<Media> getMedia() {
        return Optional.ofNullable(media);
    }
}
