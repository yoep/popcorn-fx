package com.github.yoep.popcorn.backend.media.providers;

import java.net.URI;

/**
 * Indicates that the occurred exception was caused while parsing the media information.
 * This most of the time indicates an invalid response from the API.
 */
public class MediaParsingException extends MediaRetrievalException {
    public MediaParsingException(URI uri, String message, Throwable cause) {
        super(uri, message, cause);
    }
}
