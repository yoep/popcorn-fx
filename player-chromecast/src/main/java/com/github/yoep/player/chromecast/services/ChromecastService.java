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
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleMatcher;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleType;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;
import org.springframework.stereotype.Service;
import su.litvak.chromecast.api.v2.Media;

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
     * Convert the given play request to a {@link su.litvak.chromecast.api.v2.ChromeCast} media request.
     *
     * @param request The request to convert.
     * @return Returns the chromecast media request for playback.
     */
    public Load toLoadRequest(String sessionId, PlayRequest request) {
        Objects.requireNonNull(request, "request cannot be null");
        log.trace("Creating ChromeCast media request for {}", request);
        var tracks = loadTracks(request);
        var url = request.getUrl();
        var videoMetadata = resolveMetadata(URI.create(url));
        var streamType = Media.StreamType.BUFFERED;

        // verify if the media url is supported
        // if not, we start a transcoding process through VLC
        if (!isSupportedVideoFormat(url)) {
            log.debug("Current video format/codec is not supported by Chromecast, starting transcoding of the video");
            videoMetadata = VideoMetadata.builder()
                    .contentType("video/mp4")
                    .duration(videoMetadata.getDuration())
                    .build();
            streamType = Media.StreamType.LIVE;
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
                        .duration(toChromecastTime(videoMetadata.getDuration()))
                        .streamType(streamType)
                        .customData(null)
                        .metadata(metadata)
                        .textTrackStyle(getTrackStyle())
                        .tracks(tracks)
                        .build())
                .activeTrackIds(!tracks.isEmpty() ? Collections.singletonList(0) : Collections.emptyList())
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

    private boolean isSupportedVideoFormat(String url) {
        Objects.requireNonNull(url, "url cannot be null");
        try {
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
        } catch (Exception ex) {
            log.error("Failed to verify if video format is supported, {}", ex.getMessage(), ex);
            return true;
        }
    }

    private List<Track> loadTracks(PlayRequest request) {
        var filename = FilenameUtils.getName(request.getUrl());
        var quality = request.getQuality().orElse(null);

        log.trace("Loading chromecast tracks for filename: {}, quality: {}", filename, quality);
        return subtitleService.preferredSubtitle()
                .filter(e -> !e.isNone())
                .flatMap(e -> {
                    try {
                        return Optional.of(subtitleService.downloadAndParse(e, SubtitleMatcher.from(filename, quality)).get());
                    } catch (Exception ex) {
                        log.error(ex.getMessage(), ex);
                        return Optional.empty();
                    }
                })
                .map(this::getMediaTrack)
                .orElse(Collections.emptyList());
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
        var uri = subtitleService.serve(subtitle, SubtitleType.VTT);

        return Collections.singletonList(Track.builder()
                .trackId(0)
                .type(su.litvak.chromecast.api.v2.Track.TrackType.TEXT)
                .trackContentId(uri)
                .trackContentType(SUBTITLE_CONTENT_TYPE)
                .subtype(TextTrackType.SUBTITLES)
                .language(languageCode)
                .name(languageName)
                .build());
    }

    private static Map<String, Object> getMediaMetaData(PlayRequest request) {
        var thumbnailImage = request.getBackground()
                .orElse(request.getThumbnail().orElse(null));
        return new HashMap<>() {{
            put(Media.METADATA_TYPE, Media.MetadataType.MOVIE);
            put(Media.METADATA_TITLE, request.getTitle());
            put(Media.METADATA_SUBTITLE, request.getQuality().orElse(null));
            put(ChromeCastMetadata.METADATA_THUMBNAIL, thumbnailImage);
            put(ChromeCastMetadata.METADATA_THUMBNAIL_URL, thumbnailImage);
            put(ChromeCastMetadata.METADATA_POSTER_URL, thumbnailImage);
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
