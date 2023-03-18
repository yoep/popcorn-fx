package com.github.yoep.popcorn.backend;

/**
 * Generic popcorn FX exception which can be used as base exception.
 */
public class PopcornException extends RuntimeException{
    public PopcornException(String message, Throwable cause) {
        super(message, cause);
    }
}
