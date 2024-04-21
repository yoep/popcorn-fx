package com.github.yoep.popcorn.ui;

import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.text.MessageFormat;

/**
 * Custom exception class for Inversion of Control (IoC) related errors.
 * This exception is thrown to indicate issues with IoC configuration or resolution.
 */
@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
public class IoCException extends RuntimeException {
    /**
     * The class associated with the IoC exception.
     */
    private final Class<?> clazz;

    /**
     * Constructs a new IoCException with the specified class and error message.
     *
     * @param clazz   the class associated with the IoC exception
     * @param message the detail message
     */
    public IoCException(Class<?> clazz, String message) {
        super(createMessage(clazz, message));
        this.clazz = clazz;
    }

    /**
     * Constructs a new IoCException with the specified class, error message, and cause.
     *
     * @param clazz   the class associated with the IoC exception
     * @param message the detail message
     * @param cause   the cause (which is saved for later retrieval by the Throwable.getCause() method)
     */
    public IoCException(Class<?> clazz, String message, Throwable cause) {
        super(createMessage(clazz, message), cause);
        this.clazz = clazz;
    }

    /**
     * Creates a formatted error message using the class name and provided message.
     *
     * @param clazz   the class associated with the IoC exception
     * @param message the detail message
     * @return the formatted error message
     */
    private static String createMessage(Class<?> clazz, String message) {
        return MessageFormat.format("[{0}] {1}", clazz.getSimpleName(), message);
    }
}
