package com.github.yoep.player.chromecast.services;

import com.github.yoep.player.chromecast.model.VideoMetadata;
import lombok.extern.slf4j.Slf4j;
import org.apache.http.client.HttpClient;

import java.net.URI;

// TODO: refactor to rust
@Slf4j
public record MetaDataService(HttpClient chromecastWebClient) {

    /**
     * Resolve the meta data for the given video uri.
     *
     * @param uri The uri to resolve the metadata of.
     * @return Returns the resolved metadata.
     */
    public VideoMetadata resolveMetadata(URI uri) {
        log.trace("Resolving video metadata of {}", uri);
        var contentType = "application/octet-stream";
        var duration = VideoMetadata.UNKNOWN_DURATION;
        var metadata = VideoMetadata.builder()
                .contentType(contentType)
                .duration(duration)
                .build();

        log.debug("Resolved metadata {} for uri {}", metadata, uri);
        return metadata;
    }
}
