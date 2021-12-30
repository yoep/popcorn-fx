package com.github.yoep.popcorn.backend.utils;

import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FileUtils;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

import javax.validation.constraints.NotNull;
import java.io.File;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.text.MessageFormat;
import java.util.Objects;

/**
 * The {@link FileService} is a simple wrapper around {@link File} and {@link org.apache.commons.io.FileUtils}.
 * This wrapper is mainly used for unit testing services which read/write to files.
 * @deprecated Use {@link com.github.yoep.popcorn.backend.storage.StorageService} instead.
 */
@Slf4j
@Service
@Deprecated
public class FileService {
    /**
     * Get the file for the given path.
     *
     * @param path The file path to retrieve the file from.
     * @return Returns the file.
     */
    public File getFile(@NotNull String path) {
        Assert.notNull(path, "path cannot be null");

        log.trace("Retrieving file for path {}", path);
        return new File(path);
    }

    /**
     * Get the absolute path for the given file path.
     *
     * @param path The path to resolve to an absolute path.
     * @return Returns the absolute path for the given file path.
     */
    public String getAbsolutePath(String path) {
        var file = getFile(path);
        return file.getAbsolutePath();
    }

    /**
     * Save the given contents to the given file path.
     *
     * @param path     The file path to write the contents to.
     * @param contents The contents that need to be written to the file path.
     * @return Returns true if the contents where saved with success, else false.
     */
    public boolean save(String path, String contents) {
        Objects.requireNonNull(path, "path cannot be null");
        var file = getFile(path);

        try {
            log.trace("Saving contents to file \"{}\"", file.getAbsolutePath());
            FileUtils.writeStringToFile(file, contents, StandardCharsets.UTF_8);
            log.debug("File \"{}\" has been saved", file.getAbsolutePath());
            return true;
        } catch (IOException ex) {
            var message = MessageFormat.format("Failed to save file contents to {0}, {1}", file.getAbsolutePath(), ex.getMessage());
            log.error(message, ex);
        }

        return false;
    }
}
