package com.github.yoep.popcorn.backend.storage;

import javax.validation.constraints.NotNull;
import java.io.File;
import java.nio.file.Path;
import java.util.Optional;

/**
 * The storage service is responsible for storing application data to the file system.
 * It will determine in the background where the data should be stored or read on the system.
 */
public interface StorageService {
    /**
     * Determine the subdirectory within the storage.
     *
     * @param directory The subdirectory to place within the storage.
     * @return Returns the directory from within the storage.
     * @throws StorageException Is thrown when the directory is invalid or the directory couldn't be determined.
     */
    File determineDirectoryWithinStorage(String directory);

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
     * Retrieve a path from the storage based on the given filename.
     *
     * @param name The filename to retrieve.
     * @return Returns the path to the storage file if found, else {@link Optional#empty()}.
     */
    Optional<Path> retrieve(@NotNull String name);

    /**
     * Store the given contents to the name within the storage.
     * The contents will be serialized by the {@link StorageService}.
     *
     * @param name     The name within the storage to write to.
     * @param contents The contents that need to be stored.
     * @throws StorageException Is thrown when storing of the contents failed within the storage.
     */
    void store(@NotNull String name, Object contents);

    /**
     * Clean/remove the given name from within the storage.
     * If the name doesn't exist in the storage, the action will be ignored and not result in an exception.
     *
     * @param name The name to clean within the storage.
     */
    void remove(@NotNull String name);
}
