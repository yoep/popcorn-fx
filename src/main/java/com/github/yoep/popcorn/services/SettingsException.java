package com.github.yoep.popcorn.services;

public class SettingsException extends RuntimeException {
    public SettingsException(String message) {
        super(message);
    }

    public SettingsException(String message, Throwable cause) {
        super(message, cause);
    }
}
