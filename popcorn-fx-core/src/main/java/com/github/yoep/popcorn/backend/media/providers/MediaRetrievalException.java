package com.github.yoep.popcorn.backend.media.providers;

import lombok.Getter;

import java.net.URI;

@Getter
public class MediaRetrievalException extends MediaException {
    private final URI uri;

    public MediaRetrievalException(URI uri, String message) {
        super(message);
        this.uri = uri;
    }

    public MediaRetrievalException(URI uri, String message, Throwable cause) {
        super(message, cause);
        this.uri = uri;
    }
}
