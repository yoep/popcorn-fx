package com.github.yoep.player.chromecast.services;

import com.github.kokorin.jaffree.StreamType;
import com.github.kokorin.jaffree.ffprobe.FFprobe;
import com.github.kokorin.jaffree.ffprobe.Stream;
import com.github.yoep.player.chromecast.ChromeCastMetadata;
import com.github.yoep.player.chromecast.api.v2.Load;
import com.github.yoep.player.chromecast.api.v2.TextTrackType;
import com.github.yoep.player.chromecast.api.v2.Track;
import com.github.yoep.player.chromecast.model.VideoMetadata;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleType;
import com.github.yoep.popcorn.backend.utils.HostUtils;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;
import org.springframework.boot.autoconfigure.web.ServerProperties;
import org.springframework.stereotype.Service;
import org.springframework.web.util.UriComponentsBuilder;
import su.litvak.chromecast.api.v2.Media;

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
@RequiredArgsConstructor
public class ChromecastService {
    static final Collection<String> SUPPORTED_CODECS = asList("h264", "vp8");
    static final String SUBTITLE_CONTENT_TYPE = "text/vtt";

    private final MetaDataService contentTypeService;
    private final SubtitleService subtitleService;
    private final TranscodeService transcodeService;
    private final ServerProperties serverProperties;
    private final FFprobe ffprobe;

    //region Methods

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

    /**
     * Retrieve the subtitle contents for the given subtitle file.
     *
     * @param subtitle The subtitle to retrieve.
     * @return Returns the content stream if found, else {@link Optional#empty()}.
     */
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
    public Load toLoadRequest(String sessionId, PlayRequest request) {
        Objects.requireNonNull(request, "request cannot be null");
        Objects.requireNonNull(request, "request cannot be null");
        log.trace("Creating ChromeCast media request for {}", request);
        var tracks = subtitleService.getActiveSubtitle()
                .filter(this::isSubtitleNotDisabled)
                .map(this::getMediaTrack)
                .orElse(Collections.emptyList());
        var url = request.getUrl();
        var videoMetadata = resolveMetadata(URI.create(url));

        // verify if the media url is supported
        // if not, we start a transcoding process through VLC
        if (!isSupportedVideoFormat(url)) {
            log.debug("Current video format/codec is not supported by Chromecast, starting transcoding of the video");
            videoMetadata = VideoMetadata.builder()
                    .contentType("video/mp4")
                    .duration(videoMetadata.getDuration())
                    .build();
            url = transcodeService.transcode(url);
        } else {
            log.debug("Current video format/codec is supported, transcoding not needed");
        }

        var metadata = getMediaMetaData(request);

        return Load.builder()
                .sessionId(sessionId)
                .autoplay(true)
                .currentTime(request.getAutoResumeTimestamp()
                        .map(this::toChromecastTime)
                        .orElse(0.0))
                .media(com.github.yoep.player.chromecast.api.v2.Media.builder()
                        .url(url)
                        .contentType(videoMetadata.getContentType())
                        .duration(videoMetadata.getDuration().doubleValue())
                        .streamType(Media.StreamType.BUFFERED)
                        .customData(null)
                        .metadata(metadata)
                        .textTrackStyle(getTrackStyle())
                        .tracks(tracks)
                        .build())
                .activeTrackIds(tracks.size() > 0 ? Collections.singletonList(0) : Collections.emptyList())
                .build();
    }

    /**
     * Calculate the Chromecast time from the given application time (in millis).
     * This converts the time used within the application to the format expected by the Chromecast receiver.
     *
     * @param time The time in millis.
     * @return Returns the Chromecast time as a {@link Double}.
     */
    public double toChromecastTime(long time) {
        return (double) time / 1000;
    }

    /**
     * Calculate the application time from the given Chromecast time.
     * This convert the Chromecast receiver time to the application format time.
     *
     * @param time The time from the Chromecast receiver.
     * @return Returns the application time as a {@link Long}.
     */
    public long toApplicationTime(double time) {
        return (long) (time * 1000);
    }

    /**
     * Stop all chromecast processes.
     */
    public void stop() {
        transcodeService.stop();
    }

    //endregion

    //region Functions

    private boolean isSubtitleNotDisabled(Subtitle e) {
        return !e.isNone();
    }

    private boolean isSupportedVideoFormat(String url) {
        Objects.requireNonNull(url, "url cannot be null");
        return ffprobe
                .setShowStreams(true)
                .setInput(url)
                .execute()
                .getStreams().stream()
                .filter(e -> e.getCodecType() == StreamType.VIDEO)
                .findFirst()
                .map(e -> {
                    log.trace("Probe of {}: {}", url, e);
                    return e;
                })
                .map(Stream::getCodecName)
                .filter(SUPPORTED_CODECS::contains)
                .map(e -> {
                    log.debug("Codec {} is supported by Chromecast", e);
                    return e;
                })
                .isPresent();
    }

    private List<Track> getMediaTrack(Subtitle subtitle) {
        var languageCode = subtitle.getSubtitleInfo()
                .map(SubtitleInfo::getLanguage)
                .map(SubtitleLanguage::getCode)
                .orElse(SubtitleLanguage.ENGLISH.getCode());
        var languageName = subtitle.getSubtitleInfo()
                .map(SubtitleInfo::getLanguage)
                .map(SubtitleLanguage::getNativeName)
                .orElse(SubtitleLanguage.ENGLISH.getNativeName());

        return Collections.singletonList(Track.builder()
                .trackId(0)
                .type(su.litvak.chromecast.api.v2.Track.TrackType.TEXT)
                .trackContentId(retrieveVttSubtitleUri()
                        .map(URI::toString)
                        .orElse(null))
                .trackContentType(SUBTITLE_CONTENT_TYPE)
                .subtype(TextTrackType.SUBTITLES)
                .language(languageCode)
                .name(languageName)
                .build());
    }

    private static Map<String, Object> getMediaMetaData(PlayRequest request) {
        return new HashMap<>() {{
            put(Media.METADATA_TYPE, Media.MetadataType.MOVIE);
            put(Media.METADATA_TITLE, request.getTitle().orElse(null));
            put(Media.METADATA_SUBTITLE, request.getQuality().orElse(null));
            put(ChromeCastMetadata.METADATA_THUMBNAIL, request.getThumbnail().orElse(null));
            put(ChromeCastMetadata.METADATA_THUMBNAIL_URL, request.getThumbnail().orElse(null));
            put(ChromeCastMetadata.METADATA_POSTER_URL, request.getThumbnail().orElse(null));
        }};
    }

    private static Map<String, Object> getTrackStyle() {
        return new HashMap<>() {{
            put("backgroundColor", "#00000000");
            put("edgeType", "OUTLINE");
            put("edgeColor", "#000000FF");
            put("foregroundColor", "#FFFFFFFF");
        }};
    }

    //endregion
}
