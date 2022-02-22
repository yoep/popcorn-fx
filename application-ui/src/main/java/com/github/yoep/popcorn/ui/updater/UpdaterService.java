package com.github.yoep.popcorn.ui.updater;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import com.github.yoep.popcorn.backend.storage.StorageService;
import javafx.beans.property.ReadOnlyBooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;
import org.springframework.core.io.buffer.DataBuffer;
import org.springframework.data.util.Version;
import org.springframework.stereotype.Service;
import org.springframework.web.reactive.function.client.WebClient;
import org.springframework.web.util.UriComponentsBuilder;

import javax.annotation.PostConstruct;
import java.io.File;
import java.net.URI;
import java.net.URISyntaxException;
import java.time.Duration;
import java.util.Optional;

@Slf4j
@Service
@RequiredArgsConstructor
public class UpdaterService {
    public static final String UPDATE_AVAILABLE_PROPERTY = "updateAvailable";
    static final String UPDATE_FILE_INFO = "versions.json";
    static final String SNAPSHOT_SUFFIX = "-SNAPSHOT";
    static final String DOWNLOAD_DIRECTORY = "updates";

    private final PlatformProvider platformProvider;
    private final PopcornProperties properties;
    private final WebClient webClient;
    private final ObjectMapper objectMapper;
    private final StorageService storageService;

    private final SimpleBooleanProperty updateAvailable = new SimpleBooleanProperty(this, UPDATE_AVAILABLE_PROPERTY);

    //region Properties

    public boolean isUpdateAvailable() {
        return updateAvailable.get();
    }

    public ReadOnlyBooleanProperty updateAvailableProperty() {
        return updateAvailable;
    }

    //endregion

    //region Methods

    public void runUpdate() {

    }

    //endregion

    @PostConstruct
    void init() {
        new Thread(this::checkForNewVersion, "AutoUpdater")
                .start();
    }

    void checkForNewVersion() {
        try {
            var uri = new URI(properties.getUpdateChannel() + "/" + UPDATE_FILE_INFO);

            log.trace("Retrieving version update information from {}", uri);
            var response = webClient.get()
                    .uri(uri)
                    .retrieve()
                    .toEntity(String.class)
                    .doOnError(e -> log.error("Failed to retrieve version update info, {}", e.getMessage()))
                    .block(Duration.ofSeconds(30));

            if (response != null && response.hasBody()) {
                Optional.ofNullable(response.getBody())
                        .map(this::parseResponse)
                        .ifPresent(this::verifyIfNewerVersion);
            } else {
                log.error("Failed to retrieve version update info, no data was received");
            }
        } catch (RuntimeException | URISyntaxException ex) {
            log.error("Failed to retrieve version update info, {}", ex.getMessage(), ex);
        }
    }

    private void verifyIfNewerVersion(VersionInfo versionInfo) {
        var currentVersion = currentVersion();
        var latestVersion = Version.parse(versionInfo.getVersion());

        if (latestVersion.isGreaterThan(currentVersion)) {
            log.debug("A new application version ({}) is available", latestVersion);
            downloadNewVersion(versionInfo);
        }
    }

    private void downloadNewVersion(VersionInfo versionInfo) {
        var platformInfo = platformProvider.platformInfo();

        // retrieve the download link based on the platform
        versionInfo.downloadForPlatform(platformInfo.getType(), platformInfo.getArch())
                .ifPresentOrElse(this::doInternalDownload, () -> log.warn("The download link for the update couldn't be found for the current platform"));
    }

    private void doInternalDownload(String downloadUri) {
        var uri = UriComponentsBuilder.fromUriString(downloadUri)
                .build()
                .toUri();
        var filename = DOWNLOAD_DIRECTORY + File.separator + FilenameUtils.getName(downloadUri);

        var dataBufferFlux = webClient.get()
                .uri(uri)
                .retrieve()
                .bodyToFlux(DataBuffer.class);

        storageService.store(filename, dataBufferFlux);
        updateAvailable.set(true);
    }

    private VersionInfo parseResponse(String response) {
        try {
            log.trace("Parsing update version info response");
            return objectMapper.readValue(response, VersionInfo.class);
        } catch (JsonProcessingException ex) {
            log.error("Failed to parse update version info, {}", ex.getMessage());
            return null;
        }
    }

    private Version currentVersion() {
        var currentVersion = properties.getVersion().replace(SNAPSHOT_SUFFIX, "");
        return Version.parse(currentVersion);
    }
}
