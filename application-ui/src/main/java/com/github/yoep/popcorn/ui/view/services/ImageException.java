package com.github.yoep.popcorn.ui.view.services;

import lombok.Getter;

import java.text.MessageFormat;

/**
 * Indicates that an issue occurred when loading a certain image.
 */
@Getter
public class ImageException extends RuntimeException {
    /**
     * The image url that was being loaded and failed.
     */
    private final String url;

    public ImageException(String url, String message) {
        super(formatMessage(url, message));
        this.url = url;
    }

    public ImageException(String url, String message, Throwable cause) {
        super(formatMessage(url, message), cause);
        this.url = url;
    }

    private static String formatMessage(String url, String message) {
        return MessageFormat.format("Failed to load image \"{0}\", {1}", url, message);
    }
}
