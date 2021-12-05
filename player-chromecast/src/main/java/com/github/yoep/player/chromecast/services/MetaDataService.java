package com.github.yoep.player.chromecast.services;

import com.github.yoep.player.chromecast.model.VideoMetadata;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.http.HttpEntity;
import org.springframework.http.HttpHeaders;
import org.springframework.http.MediaType;
import org.springframework.stereotype.Service;
import org.springframework.web.reactive.function.client.WebClient;

import java.net.URI;
import java.time.Duration;
import java.util.Optional;

@Slf4j
@Service
@RequiredArgsConstructor
public class MetaDataService {
    private static final int DEFAULT_RETRIEVE_TIMEOUT_SECONDS = 3;

    private final WebClient chromecastWebClient;

    /**
     * Resolve the meta data for the given video uri.
     *
     * @param uri The uri to resolve the metadata of.
     * @return Returns the resolved metadata.
     */
    public VideoMetadata resolveMetadata(URI uri) {
        var headers = retrieveVideoHeaders(uri);
        var contentType = headers
                .map(HttpHeaders::getContentType)
                .orElse(MediaType.APPLICATION_OCTET_STREAM)
                .toString();
        var duration = headers
                .map(HttpHeaders::getContentLength)
                .orElse(VideoMetadata.UNKNOWN_DURATION);
        var metadata = VideoMetadata.builder()
                .contentType(contentType)
                .duration(duration)
                .build();

        log.debug("Resolved metadata {} for uri {}", metadata, uri);
        return metadata;
    }

    private Optional<HttpHeaders> retrieveVideoHeaders(URI uri) {
        log.trace("Retrieving HTTP headers for {}", uri);
        try {
            return Optional.ofNullable(chromecastWebClient.head()
                            .uri(uri)
                            .retrieve()
                            .toBodilessEntity()
                            .block(Duration.ofSeconds(DEFAULT_RETRIEVE_TIMEOUT_SECONDS)))
                    .map(HttpEntity::getHeaders);
        } catch (RuntimeException ex) {
            log.warn("Failed to retrieve video headers, {}", ex.getMessage(), ex);
            return Optional.empty();
        }
    }
}
