package com.github.yoep.popcorn.backend.storage;

import lombok.Getter;

import java.io.File;
import java.text.MessageFormat;

@Getter
public class StorageDeserializationException extends StorageException {
    private final Class<?> valueType;

    public StorageDeserializationException(File file, Class<?> valueType, Throwable cause) {
        super(MessageFormat.format("File {0} is corrupt for type {1}", file.getAbsolutePath(), valueType.getSimpleName()), cause);
        this.valueType = valueType;
    }
}
