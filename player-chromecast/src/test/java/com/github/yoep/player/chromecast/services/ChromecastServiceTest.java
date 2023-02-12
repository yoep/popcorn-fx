package com.github.yoep.player.chromecast.services;

import com.github.kokorin.jaffree.StreamType;
import com.github.kokorin.jaffree.ffprobe.FFprobe;
import com.github.kokorin.jaffree.ffprobe.FFprobeResult;
import com.github.kokorin.jaffree.ffprobe.Stream;
import com.github.kokorin.jaffree.ffprobe.data.ProbeData;
import com.github.yoep.player.chromecast.ChromeCastMetadata;
import com.github.yoep.player.chromecast.api.v2.Load;
import com.github.yoep.player.chromecast.api.v2.TextTrackType;
import com.github.yoep.player.chromecast.api.v2.Track;
import com.github.yoep.player.chromecast.model.VideoMetadata;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.player.model.SimplePlayRequest;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleMatcher;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleType;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import su.litvak.chromecast.api.v2.Media;

import java.net.URI;
import java.util.*;
import java.util.concurrent.CompletableFuture;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class ChromecastServiceTest {
    @Mock
    private MetaDataService contentTypeService;
    @Mock
    private SubtitleService subtitleService;
    @Mock
    private TranscodeService transcodeService;
    @Mock
    private FFprobe ffprobe;
    @Mock
    private FFprobeResult ffprobeResult;
    @Mock
    private ProbeData probeData;
    @InjectMocks
    private ChromecastService service;

    @BeforeEach
    void setUp() {
        lenient().when(ffprobe.setShowStreams(true)).thenReturn(ffprobe);
        lenient().when(ffprobe.setInput(isA(String.class))).thenReturn(ffprobe);
        lenient().when(ffprobe.execute()).thenReturn(ffprobeResult);
    }

    @Test
    void testResolveMetadata_whenUriIsGiven_shouldResolveMetadata() {
        var uri = URI.create("http://localhost:8080/lipsum");
        var expectedResult = VideoMetadata.builder()
                .contentType("video/mp4")
                .duration(2000L)
                .build();
        when(contentTypeService.resolveMetadata(uri)).thenReturn(expectedResult);

        var result = service.resolveMetadata(uri);

        assertEquals(expectedResult, result);
    }

    @Test
    void testToLoadRequest_whenFormatIsSupported_shouldUseOriginalUrl() {
        var url = "http://localhost:9976/my-video-url.mp4";
        var subtitleUri = "http://localhost:8754/lorem.vtt";
        var sessionId = "mySessionId";
        var contentType = "video/mp4";
        var duration = 20000L;
        var subtitleInfo = mock(SubtitleInfo.class);
        var subtitle = mock(Subtitle.class);
        var request = SimplePlayRequest.builder()
                .url(url)
                .title("My movie title")
                .autoResumeTimestamp(20000L)
                .thumb("https://thumbs.com/my-thumb.jpg")
                .build();
        var metadata = createMetadata(request);
        var tracks = Collections.singletonList(Track.builder()
                .trackId(0)
                .type(su.litvak.chromecast.api.v2.Track.TrackType.TEXT)
                .subtype(TextTrackType.SUBTITLES)
                .language("en")
                .name("English")
                .trackContentType("text/vtt")
                .trackContentId(subtitleUri)
                .build());
        var expectedResult = Load.builder()
                .sessionId(sessionId)
                .autoplay(true)
                .currentTime(20.0)
                .media(com.github.yoep.player.chromecast.api.v2.Media.builder()
                        .url(url)
                        .contentType(contentType)
                        .duration((double) duration / 1000)
                        .streamType(Media.StreamType.BUFFERED)
                        .metadata(metadata)
                        .textTrackStyle(createTrackStyle())
                        .tracks(tracks)
                        .build())
                .activeTrackIds(Collections.singletonList(0))
                .build();
        when(subtitleService.preferredSubtitle()).thenReturn(Optional.of(subtitleInfo));
        when(subtitleService.downloadAndParse(eq(subtitleInfo), isA(SubtitleMatcher.class))).thenReturn(CompletableFuture.completedFuture(subtitle));
        when(subtitleService.serve(subtitle, SubtitleType.VTT)).thenReturn(subtitleUri);
        when(contentTypeService.resolveMetadata(URI.create(url))).thenReturn(VideoMetadata.builder()
                .contentType(contentType)
                .duration(duration)
                .build());
        when(ffprobeResult.getStreams()).thenReturn(Collections.singletonList(new Stream(probeData)));
        when(probeData.getStreamType("codec_type")).thenReturn(StreamType.VIDEO);
        when(probeData.getString("codec_name")).thenReturn(getSupportedCodec());

        var result = service.toLoadRequest(sessionId, request);

        assertEquals(expectedResult, result);
    }

    @Test
    void testToMediaRequest_whenFormatIsNotSupported_shouldUseTranscodedUrl() {
        var url = "http://localhost:9976/my-video-url.mkv";
        var sessionId = "mySessionId";
        var transcodedUrl = "http://localhost:9976/my-video-url.mp4";
        var contentType = "video/mp4";
        var duration = 20000L;
        var request = SimplePlayRequest.builder()
                .url(url)
                .title("My movie title")
                .autoResumeTimestamp(60500L)
                .thumb("https://thumbs.com/my-thumb.jpg")
                .build();
        var metadata = createMetadata(request);
        var expectedResult = Load.builder()
                .sessionId(sessionId)
                .autoplay(true)
                .currentTime(60.5)
                .media(com.github.yoep.player.chromecast.api.v2.Media.builder()
                        .url(transcodedUrl)
                        .contentType(contentType)
                        .duration((double) duration / 1000)
                        .streamType(Media.StreamType.LIVE)
                        .metadata(metadata)
                        .textTrackStyle(createTrackStyle())
                        .tracks(Collections.emptyList())
                        .build())
                .activeTrackIds(Collections.emptyList())
                .build();
        when(contentTypeService.resolveMetadata(URI.create(url))).thenReturn(VideoMetadata.builder()
                .contentType(contentType)
                .duration(duration)
                .build());
        when(transcodeService.transcode(url)).thenReturn(transcodedUrl);
        when(ffprobeResult.getStreams()).thenReturn(Collections.singletonList(new Stream(probeData)));
        when(probeData.getStreamType("codec_type")).thenReturn(StreamType.VIDEO);
        when(probeData.getString("codec_name")).thenReturn("mkv");

        var result = service.toLoadRequest(sessionId, request);

        assertEquals(expectedResult, result);
    }

    @Test
    void testToApplicationTime_whenTimeIsGiven_shouldReturnExpectedResult() {
        var time = 20.5;
        var expectedTime = 20500L;

        var result = service.toApplicationTime(time);

        assertEquals(expectedTime, result);
    }

    private static Map<String, Object> createMetadata(PlayRequest request) {
        return new HashMap<>() {{
            put(Media.METADATA_TYPE, Media.MetadataType.MOVIE);
            put(Media.METADATA_TITLE, request.getTitle().orElse(null));
            put(Media.METADATA_SUBTITLE, request.getQuality().orElse(null));
            put(ChromeCastMetadata.METADATA_THUMBNAIL, request.getThumbnail().orElse(null));
            put(ChromeCastMetadata.METADATA_THUMBNAIL_URL, request.getThumbnail().orElse(null));
            put(ChromeCastMetadata.METADATA_POSTER_URL, request.getThumbnail().orElse(null));
        }};
    }

    private static Map<String, Object> createTrackStyle() {
        return new HashMap<>() {{
            put("backgroundColor", "#00000000");
            put("edgeType", "OUTLINE");
            put("edgeColor", "#000000FF");
            put("foregroundColor", "#FFFFFFFF");
        }};
    }

    private static String getSupportedCodec() {
        return new ArrayList<>(ChromecastService.SUPPORTED_CODECS).get(0);
    }
}