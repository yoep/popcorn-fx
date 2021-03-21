package com.github.yoep.player.adapter;

/**
 * Defines the base exception for all player exceptions.
 * This exception indicates that an issue occurred while working with the {@link Player}.
 */
public class PlayerException extends RuntimeException {
    public PlayerException(String message) {
        super(message);
    }

    public PlayerException(String message, Throwable cause) {
        super(message, cause);
    }
}
