package com.github.yoep.popcorn.ui.updater;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.data.util.Version;
import org.springframework.stereotype.Service;
import org.springframework.web.reactive.function.client.WebClient;
import org.springframework.web.util.UriComponentsBuilder;

import javax.annotation.PostConstruct;
import java.time.Duration;

@Slf4j
@Service
@RequiredArgsConstructor
public class UpdaterService {
    static final String UPDATE_FILE_INFO = "versions.json";
    static final String SNAPSHOT_SUFFIX = "-SNAPSHOT";

    private final PlatformProvider platformProvider;
    private final PopcornProperties properties;
    private final WebClient webClient;

    @PostConstruct
    void init() {
        new Thread(this::checkForNewVersion, "AutoUpdater")
                .start();
    }

    void checkForNewVersion() {
        var uri = UriComponentsBuilder.fromUri(properties.getUpdateChannel())
                .path(UPDATE_FILE_INFO)
                .build()
                .toUri();
        var currentVersion = currentVersion();

        try {
            log.trace("Retrieving version update information from {}", uri);
            var response = webClient.get()
                    .uri(uri)
                    .retrieve()
                    .bodyToMono(VersionInfo.class)
                    .doOnError(e -> log.error("Failed to retrieve version update info, {}", e.getMessage()))
                    .block(Duration.ofSeconds(30));

            if (response != null) {
                var latestVersion = Version.parse(response.getVersion());

                if (latestVersion.isGreaterThan(currentVersion)) {
                    log.debug("A new application version ({}) is available", latestVersion);
                }
            } else {
                log.error("Failed to retrieve version update info, no data was received");
            }
        } catch (RuntimeException ex) {
            log.error("Failed to retrieve version update info, {}", ex.getMessage(), ex);
        }
    }

    private Version currentVersion() {
        var currentVersion = properties.getVersion().replace(SNAPSHOT_SUFFIX, "");
        return Version.parse(currentVersion);
    }
}
