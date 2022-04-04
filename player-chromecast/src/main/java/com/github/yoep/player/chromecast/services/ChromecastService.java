package com.github.yoep.player.chromecast.services;

import com.github.yoep.player.chromecast.model.VideoMetadata;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleType;
import lombok.RequiredArgsConstructor;
import org.apache.commons.io.FilenameUtils;
import org.springframework.boot.autoconfigure.web.ServerProperties;
import org.springframework.stereotype.Service;
import org.springframework.web.util.UriComponentsBuilder;

import java.io.File;
import java.io.InputStream;
import java.net.URI;
import java.util.Collections;
import java.util.Objects;
import java.util.Optional;

/**
 * General purpose service for chromecast devices.
 * This service tries to handle the most logic which isn't directly related to a {@link su.litvak.chromecast.api.v2.ChromeCast} device.
 */
@Service
@RequiredArgsConstructor
public class ChromecastService {
    private final MetaDataService contentTypeService;
    private final SubtitleService subtitleService;
    private final ServerProperties serverProperties;

    /**
     * Resolve the metadata of the given video uri.
     * This will fetch the headers of the given video uri and extract the metadata from it.
     *
     * @param uri The uri to extract the metadata from.
     * @return Returns the resolved metadata of the video uri.
     */
    public VideoMetadata resolveMetadata(URI uri) {
        Objects.requireNonNull(uri, "uri cannot be null");
        return contentTypeService.resolveMetadata(uri);
    }

    /**
     * Retrieve the uri on which the subtitle can be found.
     * This uri is based on the currently active subtitle.
     *
     * @return Returns the uri of the subtitle if one is available, else {@link Optional#empty()}.
     */
    public Optional<URI> retrieveVttSubtitleUri() {
        return subtitleService.getActiveSubtitle()
                .filter(e -> !e.isNone())
                .flatMap(Subtitle::getFile)
                .map(File::getName)
                .map(FilenameUtils::getName)
                .map(e -> UriComponentsBuilder.newInstance()
                        .scheme("http")
                        .host("127.0.0.1")
                        .port(serverProperties.getPort())
                        .path("/subtitle/{subtitle}")
                        .build(Collections.singletonMap("subtitle", e)));
    }

    public Optional<InputStream> retrieveVttSubtitle(String subtitle) {
        return subtitleService.getActiveSubtitle()
                .filter(e -> !e.isNone())
                .map(e -> subtitleService.convert(e, SubtitleType.VTT));
    }
}
