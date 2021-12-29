package com.github.yoep.popcorn.backend.storage;

import lombok.Getter;

import java.text.MessageFormat;

@Getter
public class StorageSerializationException extends StorageException {
    private final Class<?> valueType;

    protected StorageSerializationException(Class<?> valueType, Throwable cause) {
        super(MessageFormat.format("Failed to serialize type {0}", valueType.getSimpleName()), cause);
        this.valueType = valueType;
    }
}
