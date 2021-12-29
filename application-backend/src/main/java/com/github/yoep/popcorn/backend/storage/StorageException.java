package com.github.yoep.popcorn.backend.storage;

import lombok.EqualsAndHashCode;
import lombok.ToString;

import java.io.File;
import java.util.Optional;

@ToString
@EqualsAndHashCode(callSuper = false)
public class StorageException extends RuntimeException {
    private final File file;

    protected StorageException(String message, Throwable cause) {
        super(message, cause);
        this.file = null;
    }

    public StorageException(File file, String message, Throwable cause) {
        super(message, cause);
        this.file = file;
    }

    public Optional<File> getFile() {
        return Optional.ofNullable(file);
    }
}
