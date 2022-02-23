package com.github.yoep.popcorn.ui.updater;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import com.github.yoep.popcorn.backend.storage.StorageService;
import javafx.beans.property.ReadOnlyObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.buffer.DataBuffer;
import org.springframework.core.task.TaskExecutor;
import org.springframework.data.util.Version;
import org.springframework.http.MediaType;
import org.springframework.stereotype.Service;
import org.springframework.web.reactive.function.client.WebClient;
import org.springframework.web.util.UriComponentsBuilder;

import javax.annotation.PostConstruct;
import javax.annotation.PreDestroy;
import java.net.URI;
import java.net.URISyntaxException;
import java.time.Duration;
import java.util.Optional;

@Slf4j
@Service
@RequiredArgsConstructor
public class UpdateService {
    public static final String UPDATE_INFO_PROPERTY = "updateInfo";
    public static final String UPDATE_STATE_PROPERTY = "state";

    static final String UPDATE_FILE_INFO = "versions.json";
    static final String SNAPSHOT_SUFFIX = "-SNAPSHOT";
    static final String DOWNLOAD_NAME = "update";

    private final PlatformProvider platformProvider;
    private final PopcornProperties properties;
    private final WebClient webClient;
    private final StorageService storageService;
    private final TaskExecutor taskExecutor;

    private final SimpleObjectProperty<VersionInfo> updateInfo = new SimpleObjectProperty<>(this, UPDATE_INFO_PROPERTY);
    private final SimpleObjectProperty<UpdateState> state = new SimpleObjectProperty<>(this, UPDATE_STATE_PROPERTY, UpdateState.CHECKING_FOR_NEW_VERSION);

    private boolean shouldUpdateOnExit;

    //region Properties

    /**
     * Get the known update information if available.
     *
     * @return Returns the version info if available, else {@link Optional#empty()}.
     */
    public Optional<VersionInfo> getUpdateInfo() {
        return Optional.ofNullable(updateInfo.get());
    }

    /**
     * The update info property which holds the latest version information.
     *
     * @return Returns the version info property.
     */
    public ReadOnlyObjectProperty<VersionInfo> updateInfoProperty() {
        return updateInfo;
    }

    public UpdateState getState() {
        return state.get();
    }

    public ReadOnlyObjectProperty<UpdateState> stateProperty() {
        return state;
    }

    //endregion

    //region Methods

    public void startUpdateAndExit() {
        taskExecutor.execute(this::doInternalUpdate);
    }

    public void downloadUpdate() {
        taskExecutor.execute(() -> getUpdateInfo().ifPresent(e -> {
            var platformInfo = platformProvider.platformInfo();

            e.downloadForPlatform(platformInfo.getType(), platformInfo.getArch())
                    .ifPresent(this::doInternalDownload);
        }));
    }

    //endregion

    @PostConstruct
    void init() {
        taskExecutor.execute(this::checkForNewVersion);
    }

    @PreDestroy
    void onDestroy() {
        if (shouldUpdateOnExit) {
            log.debug("Launching application update");
            storageService.retrieve(getTargetFilename())
                    .ifPresent(platformProvider::launch);
        }
    }

    private void checkForNewVersion() {
        try {
            var uri = new URI(properties.getUpdateChannel() + "/" + UPDATE_FILE_INFO);

            log.trace("Retrieving version update information from {}", uri);
            var response = webClient.get()
                    .uri(uri)
                    .retrieve()
                    .toEntity(VersionInfo.class)
                    .doOnError(e -> {
                        state.set(UpdateState.ERROR);
                        log.error("Failed to retrieve version update info, {}", e.getMessage());
                    })
                    .block(Duration.ofSeconds(30));

            if (response != null && response.hasBody()) {
                Optional.ofNullable(response.getBody())
                        .ifPresent(this::verifyIfNewerVersionIsAvailable);
            } else {
                log.error("Failed to retrieve version update info, no data was received");
            }
        } catch (RuntimeException | URISyntaxException ex) {
            log.error("Failed to retrieve version update info, {}", ex.getMessage(), ex);
        }
    }

    private void verifyIfNewerVersionIsAvailable(VersionInfo versionInfo) {
        var currentVersion = currentVersion();
        var latestVersion = Version.parse(versionInfo.getVersion());

        if (latestVersion.isGreaterThan(currentVersion)) {
            log.debug("A new application version ({}) is available", latestVersion);
            var platformInfo = platformProvider.platformInfo();
            var optionalDownloadUri = versionInfo.downloadForPlatform(platformInfo.getType(), platformInfo.getArch());

            if (optionalDownloadUri.isPresent()) {
                log.trace("Update download uri {} has been found for the platform {}", optionalDownloadUri.get(), platformInfo);
                updateInfo.set(versionInfo);
                state.set(UpdateState.UPDATE_AVAILABLE);
                return;
            } else {
                log.warn("No download uri for platform {} is available for the latest version", platformInfo);
            }
        }

        cleanup();
        state.set(UpdateState.NO_UPDATE_AVAILABLE);
    }

    private void doInternalDownload(String downloadUri) {
        var uri = UriComponentsBuilder.fromUriString(downloadUri)
                .build()
                .toUri();
        var targetFilename = getTargetFilename();

        state.set(UpdateState.DOWNLOADING);
        var dataBufferFlux = webClient.get()
                .uri(uri)
                .accept(MediaType.ALL)
                .retrieve()
                .bodyToFlux(DataBuffer.class);

        storageService.store(targetFilename, dataBufferFlux);
        state.set(UpdateState.DOWNLOAD_FINISHED);
    }

    private void doInternalUpdate() {
        state.set(UpdateState.INSTALLING);
        shouldUpdateOnExit = true;
        platformProvider.exit();
    }

    private Version currentVersion() {
        var currentVersion = properties.getVersion().replace(SNAPSHOT_SUFFIX, "");
        return Version.parse(currentVersion);
    }

    private String getTargetFilename() {
        var extension = "deb";

        switch (platformProvider.platformInfo().getType()) {
            case WINDOWS -> extension = "exe";
            case MAC -> extension = "dmg";
        }

        return DOWNLOAD_NAME + "." + extension;
    }

    private void cleanup() {
        storageService.remove(getTargetFilename());
    }
}
