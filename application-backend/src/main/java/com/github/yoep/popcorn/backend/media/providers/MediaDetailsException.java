package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.media.Media;
import com.github.yoep.popcorn.backend.media.MediaException;

public class MediaDetailsException extends MediaException {
    public MediaDetailsException(Media media, String message, Throwable cause) {
        super(media, message, cause);
    }
}
