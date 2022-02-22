package com.github.yoep.popcorn.backend.storage;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.JsonMappingException;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.github.yoep.popcorn.backend.BackendConstants;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FileUtils;
import org.apache.commons.lang3.StringUtils;
import org.springframework.core.io.buffer.DataBuffer;
import org.springframework.core.io.buffer.DataBufferUtils;
import org.springframework.lang.Nullable;
import org.springframework.stereotype.Service;
import reactor.core.publisher.Flux;

import java.io.File;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.StandardOpenOption;
import java.text.MessageFormat;
import java.util.Objects;
import java.util.Optional;

@Slf4j
@Service
@RequiredArgsConstructor
public class StorageServiceImpl implements StorageService {
    private final ObjectMapper objectMapper;

    @Override
    public File determineDirectoryWithinStorage(String directory) {
        Objects.requireNonNull(directory, "directory cannot be null");
        var storageDirectory = new File(determineStorageBaseDirectory());
        var calculatedDirectory = new File(storageDirectory.getAbsolutePath() + File.separator + directory);

        try {
            var storagePath = storageDirectory.getCanonicalPath();
            var calculatedPath = calculatedDirectory.getCanonicalPath();

            if (calculatedPath.startsWith(storagePath)) {
                return calculatedDirectory;
            } else {
                throw new StorageException(calculatedDirectory, "Directory is invalid as it leaves the storage space");
            }
        } catch (IOException ex) {
            throw new StorageException(calculatedDirectory, ex.getMessage(), ex);
        }
    }

    @Override
    public <T> Optional<T> read(String name, Class<T> valueType) {
        Objects.requireNonNull(name, "name cannot be null");
        var file = determineStorageFile(name);

        if (file.exists()) {
            try {
                return Optional.ofNullable(objectMapper.readValue(file, valueType));
            } catch (JsonMappingException ex) {
                throw new StorageDeserializationException(file, valueType, ex);
            } catch (IOException ex) {
                var message = MessageFormat.format("Failed to read from {0}, {1}", file.getAbsolutePath(), ex.getMessage());
                throw new StorageException(file, message, ex);
            }
        }

        return Optional.empty();
    }

    @Override
    public void store(String name, Object contents) {
        Objects.requireNonNull(name, "name cannot be null");
        var file = determineStorageFile(name);
        var serializedContents = serializeContentsIfNeeded(name, contents);

        try {
            // verify if the parent directory exist
            // if not, create it before storing the actual file
            createStorageDirectoryIfNeeded(file);

            log.trace("Saving contents to file \"{}\"", file.getAbsolutePath());
            FileUtils.writeStringToFile(file, serializedContents, StandardCharsets.UTF_8);
            log.debug("{} \"{}\" has been saved", Optional.ofNullable(contents)
                    .map(Object::getClass)
                    .map(Class::getSimpleName)
                    .orElse("File"), file.getAbsolutePath());
        } catch (IOException ex) {
            var message = MessageFormat.format("Failed to save file contents to {0}, {1}", file.getAbsolutePath(), ex.getMessage());
            throw new StorageException(file, message, ex);
        }
    }

    @Override
    public void store(String name, Flux<DataBuffer> buffer) {
        Objects.requireNonNull(name, "name cannot be null");
        var file = determineStorageFile(name);

        DataBufferUtils
                .write(buffer, file.toPath(), StandardOpenOption.CREATE)
                .block();
    }

    @Nullable
    private String serializeContentsIfNeeded(String name, Object contents) {
        if (Objects.isNull(contents)) {
            return null;
        }

        var contentsType = contents.getClass();

        try {
            log.trace("Serializing contents for {} of type {}", name, contentsType.getSimpleName());
            return objectMapper.writeValueAsString(contents);
        } catch (JsonProcessingException ex) {
            throw new StorageSerializationException(contentsType, ex);
        }
    }

    private void createStorageDirectoryIfNeeded(File file) {
        var storageDirectory = file.getParentFile();

        if (!storageDirectory.exists()) {
            log.trace("Creating storage directory {}", storageDirectory.getAbsolutePath());
            if (storageDirectory.mkdirs()) {
                log.debug("Storage directory has been created");
            } else {
                log.warn("Failed to create storage directory at \"{}\", this can cause fatal exceptions in the future", storageDirectory.getAbsolutePath());
            }
        }
    }

    private File determineStorageFile(String filename) {
        var storageDirectory = determineStorageBaseDirectory();

        return new File(storageDirectory + File.separator + filename);
    }

    /**
     * Determine the storage base directory path to use.
     * This directory path is based on the home directory of the user by default,
     * but can be overridden by setting {@link BackendConstants#POPCORN_HOME_PROPERTY} in the system properties.
     *
     * @return Returns the storage base directory path.
     */
    private String determineStorageBaseDirectory() {
        var baseDir = System.getProperty("user.home") + File.separator + BackendConstants.POPCORN_HOME_DIRECTORY;

        if (StringUtils.isNotEmpty(System.getProperty(BackendConstants.POPCORN_HOME_PROPERTY))) {
            log.trace("Property {} has been defined and will override the base storage directory", BackendConstants.POPCORN_HOME_PROPERTY);
            baseDir = System.getProperty(BackendConstants.POPCORN_HOME_PROPERTY);
        }

        return baseDir;
    }
}
