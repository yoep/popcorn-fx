package com.github.yoep.popcorn.backend.media;

import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.util.Optional;

@ToString
@EqualsAndHashCode(callSuper = true)
public class MediaException extends RuntimeException {
    private final Media media;
    @Getter
    private final ErrorType type;

    public MediaException(String message) {
        super(message);
        this.media = null;
        this.type = ErrorType.OTHER;
    }

    public MediaException(ErrorType type, String message) {
        super(message);
        this.media = null;
        this.type = type;
    }

    public MediaException(Media media, ErrorType type, String message) {
        super(message);
        this.media = media;
        this.type = type;
    }

    public Optional<Media> getMedia() {
        return Optional.ofNullable(media);
    }

    public enum ErrorType {
        PARSING,
        RETRIEVAL,
        OTHER,
    }
}
