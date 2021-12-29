package com.github.yoep.popcorn.backend.storage;

import javax.validation.constraints.NotNull;
import java.util.Optional;

/**
 * The storage service is responsible for storing application data to the file system.
 * It will determine in the background where the data should be stored or read on the system.
 */
public interface StorageService {
    /**
     * Read the given name from the storage.
     * The contents will deserialized by the {@link StorageService}.
     *
     * @param name      The name within the storage to read from.
     * @param valueType The expected return type (this should be the same type as the one passed to {@link #store(String, Object)}).
     * @return Returns the contents for the given name from the storage when found, else {@link Optional#empty()}.
     * @throws StorageException Is thrown when reading the contents from the storage failed.
     */
    <T> Optional<T> read(@NotNull String name, Class<T> valueType);

    /**
     * Store the given contents to the name within the storage.
     * The contents will be serialized by the {@link StorageService}.
     *
     * @param name     The name within the storage to write to.
     * @param contents The contents that need to be stored.
     * @throws StorageException Is thrown when storing of the contents failed within the storage.
     */
    void store(@NotNull String name, Object contents);
}
