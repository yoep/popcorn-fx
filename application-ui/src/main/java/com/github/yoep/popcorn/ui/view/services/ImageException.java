package com.github.yoep.popcorn.ui.view.services;

import lombok.Getter;

import java.text.MessageFormat;

@Getter
public class ImageException extends RuntimeException {
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
