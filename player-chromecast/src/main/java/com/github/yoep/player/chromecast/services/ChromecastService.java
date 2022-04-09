package com.github.yoep.player.chromecast.services;

import com.github.yoep.player.chromecast.ChromeCastMetaData;
import com.github.yoep.player.chromecast.model.VideoMetadata;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleType;
import com.github.yoep.popcorn.backend.utils.HostUtils;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;
import org.springframework.boot.autoconfigure.web.ServerProperties;
import org.springframework.stereotype.Service;
import org.springframework.web.util.UriComponentsBuilder;
import su.litvak.chromecast.api.v2.Media;
import su.litvak.chromecast.api.v2.Track;

import java.io.File;
import java.io.InputStream;
import java.net.URI;
import java.util.*;

import static java.util.Arrays.asList;

/**
 * General purpose service for chromecast devices.
 * This service tries to handle the most logic which isn't directly related to a {@link su.litvak.chromecast.api.v2.ChromeCast} device.
 */
@Slf4j
@Service
public record ChromecastService(MetaDataService contentTypeService,
                                SubtitleService subtitleService,
                                TranscodeService transcodeService,
                                ServerProperties serverProperties) {
    static final Collection<String> SUPPORTED_MEDIA_TYPES = asList("mp4", "ogg", "wav", "webm");

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
                .filter(this::isSubtitleNotDisabled)
                .flatMap(Subtitle::getFile)
                .map(File::getName)
                .map(FilenameUtils::getBaseName)
                .map(e -> e + "." + SubtitleType.VTT.getExtension())
                .map(e -> UriComponentsBuilder.newInstance()
                        .scheme("http")
                        .host(HostUtils.hostAddress())
                        .port(serverProperties.getPort())
                        .path("/subtitle/{subtitle}")
                        .build(Collections.singletonMap("subtitle", e)));
    }

    public Optional<InputStream> retrieveVttSubtitle(String subtitle) {
        var name = FilenameUtils.getBaseName(subtitle);

        return subtitleService.getActiveSubtitle()
                .filter(this::isSubtitleNotDisabled)
                .filter(e -> e.getFile().isPresent())
                .filter(e -> FilenameUtils.getBaseName(e.getFile().get().getName()).equals(name))
                .map(e -> subtitleService.convert(e, SubtitleType.VTT));
    }

    /**
     * Convert the given play request to a {@link su.litvak.chromecast.api.v2.ChromeCast} media request.
     *
     * @param request The request to convert.
     * @return Returns the chromecast media request for playback.
     */
    public Media toMediaRequest(PlayRequest request) {
        Objects.requireNonNull(request, "request cannot be null");
        log.trace("Creating ChromeCast media request for {}", request);
        var tracks = subtitleService.getActiveSubtitle()
                .filter(this::isSubtitleNotDisabled)
                .map(e -> Collections.singletonList(new Track(1, Track.TrackType.TEXT)))
                .orElse(Collections.emptyList());
        var url = request.getUrl();
        var extension = FilenameUtils.getExtension(url);

        // verify if the media url is supported
        // if not, we start a transcoding process through VLC
        if (!isSupportedVideoFormat(extension)) {
            log.debug("Current video format {} is not supported by Chromecast, starting transcoding of the video", extension);
            url = transcodeService.transcode(url);
        } else {
            log.debug("Current video format {} is supported, transcoding not needed", extension);
        }

        var metadata = getMediaMetaData(request);
        var videoMetadata = resolveMetadata(URI.create(url));

        return new Media(url, videoMetadata.getContentType(), videoMetadata.getDuration().doubleValue(), Media.StreamType.BUFFERED,
                null, metadata, null, tracks);
    }

    private boolean isSubtitleNotDisabled(Subtitle e) {
        return !e.isNone();
    }

    private boolean isSupportedVideoFormat(String extension) {
        Objects.requireNonNull(extension, "extension cannot be null");
        return SUPPORTED_MEDIA_TYPES.contains(extension.toLowerCase());
    }

    private Map<String, Object> getMediaMetaData(PlayRequest request) {
        var subtitleUri = retrieveVttSubtitleUri()
                .map(URI::toString)
                .map(this::subtitleAvailability)
                .orElseGet(this::noSubtitleAvailable);

        return new HashMap<>() {{
            put(Media.METADATA_TYPE, Media.MetadataType.MOVIE);
            put(Media.METADATA_TITLE, request.getTitle().orElse(null));
            put(Media.METADATA_SUBTITLE, request.getQuality().orElse(null));
            put(ChromeCastMetaData.METADATA_SUBTITLES, new HashMap<>() {{
                put("uri", subtitleUri);
            }});
            put(ChromeCastMetaData.METADATA_THUMBNAIL, request.getThumbnail().orElse(null));
            put(ChromeCastMetaData.METADATA_THUMBNAIL_URL, request.getThumbnail().orElse(null));
            put(ChromeCastMetaData.METADATA_POSTER_URL, request.getThumbnail().orElse(null));
        }};
    }

    private String subtitleAvailability(String uri) {
        log.debug("Chromecast subtitle will be available at {}", uri);
        return uri;
    }

    private String noSubtitleAvailable() {
        log.debug("No active subtitle available for Chromecast");
        return null;
    }
}
