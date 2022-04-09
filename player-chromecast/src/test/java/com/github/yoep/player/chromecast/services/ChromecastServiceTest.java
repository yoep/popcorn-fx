package com.github.yoep.player.chromecast.services;

import com.github.yoep.player.chromecast.ChromeCastMetaData;
import com.github.yoep.player.chromecast.model.VideoMetadata;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.player.model.SimplePlayRequest;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleType;
import com.github.yoep.popcorn.backend.utils.HostUtils;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.boot.autoconfigure.web.ServerProperties;
import su.litvak.chromecast.api.v2.Media;
import su.litvak.chromecast.api.v2.Track;

import java.io.File;
import java.io.InputStream;
import java.net.InetAddress;
import java.net.URI;
import java.net.UnknownHostException;
import java.text.MessageFormat;
import java.util.Collections;
import java.util.HashMap;
import java.util.Map;
import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class ChromecastServiceTest {
    @Mock
    private MetaDataService contentTypeService;
    @Mock
    private SubtitleService subtitleService;
    @Mock
    private ServerProperties serverProperties;
    @Mock
    private TranscodeService transcodeService;
    @InjectMocks
    private ChromecastService service;

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
    void testRetrieveVttSubtitleUri_whenTheActiveSubtitleIsNone_shouldReturnEmpty() {
        when(subtitleService.getActiveSubtitle()).thenReturn(Optional.of(Subtitle.none()));

        var result = service.retrieveVttSubtitleUri();

        assertTrue(result.isEmpty(), "Expected no subtitle uri to have been returned");
    }

    @Test
    void testRetrieveVttSubtitleUri_whenSubtitleIsActive_shouldReturnExpectedSubtitleUri() throws UnknownHostException {
        var port = 9998;
        var file = new File("my-subtitle.srt");
        var subtitle = new Subtitle(file, Collections.emptyList());
        var host = InetAddress.getLocalHost().getHostAddress();
        var expectedSubtitle = MessageFormat.format("http://{0}:{1}/subtitle/my-subtitle.vtt", host, String.valueOf(port));
        when(serverProperties.getPort()).thenReturn(port);
        when(subtitleService.getActiveSubtitle()).thenReturn(Optional.of(subtitle));

        var result = service.retrieveVttSubtitleUri();

        assertTrue(result.isPresent(), "Expected subtitle uri to be present");
        assertEquals(expectedSubtitle, result.get().toString());
    }

    @Test
    void testRetrieveVttSubtitle_whenSubtitleDoesNotMatchActiveSubtitle_shouldReturnEmpty() {
        var subtitleName = "lorem.vtt";
        var activeSubtitle = "ipsum.vtt";
        var subtitle = new Subtitle(new File(activeSubtitle), Collections.emptyList());
        when(subtitleService.getActiveSubtitle()).thenReturn(Optional.of(subtitle));

        var result = service.retrieveVttSubtitle(subtitleName);

        assertTrue(result.isEmpty(), "Expected an empty subtitle to be returned");
    }

    @Test
    void testRetrieveVttSubtitle_whenSubtitleMatchesActiveSubtitle_shouldReturnSubtitleContents() {
        var activeSubtitle = "ipsum.vtt";
        var subtitle = new Subtitle(new File(activeSubtitle), Collections.emptyList());
        var expectedResult = mock(InputStream.class);
        when(subtitleService.getActiveSubtitle()).thenReturn(Optional.of(subtitle));
        when(subtitleService.convert(subtitle, SubtitleType.VTT)).thenReturn(expectedResult);

        var result = service.retrieveVttSubtitle(activeSubtitle);

        assertTrue(result.isPresent(), "Expected a subtitle to be returned");
        assertEquals(expectedResult, result.get());
    }

    @Test
    void testToMediaRequest_whenFormatIsSupported_shouldUseOriginalUrl() {
        var url = "http://localhost:9976/my-video-url.mp4";
        var contentType = "video/mp4";
        var duration = 20000L;
        var port = 9999;
        var subtitle = new Subtitle(new File("my-subtitle.srt"), Collections.emptyList());
        var request = SimplePlayRequest.builder()
                .url(url)
                .title("My movie title")
                .autoResumeTimestamp(20000L)
                .thumb("https://thumbs.com/my-thumb.jpg")
                .build();
        var metadata = createMetadata(request, MessageFormat.format("http://{0}:{1}/subtitle/my-subtitle.vtt", HostUtils.hostAddress(), String.valueOf(port)));
        var tracks = Collections.singletonList(new Track(1, Track.TrackType.TEXT));
        var expectedResult = new Media(url, contentType, (double) duration, Media.StreamType.BUFFERED, null, metadata, null, tracks);
        when(contentTypeService.resolveMetadata(URI.create(url))).thenReturn(VideoMetadata.builder()
                .contentType(contentType)
                .duration(duration)
                .build());
        when(serverProperties.getPort()).thenReturn(port);
        when(subtitleService.getActiveSubtitle()).thenReturn(Optional.of(subtitle));

        var result = service.toMediaRequest(request);

        assertEquals(expectedResult, result);
        assertEquals(metadata, result.metadata);
    }

    @Test
    void testToMediaRequest_whenFormatIsNotSupported_shouldUseTranscodedUrl() {
        var url = "http://localhost:9976/my-video-url.mkv";
        var transcodedUrl = "http://localhost:9976/my-video-url.mp4";
        var contentType = "video/mp4";
        var duration = 20000L;
        var request = SimplePlayRequest.builder()
                .url(url)
                .title("My movie title")
                .autoResumeTimestamp(20000L)
                .thumb("https://thumbs.com/my-thumb.jpg")
                .build();
        var metadata = createMetadata(request, null);
        var expectedResult = new Media(transcodedUrl, contentType, (double) duration, Media.StreamType.BUFFERED, null, metadata, null, Collections.emptyList());
        when(contentTypeService.resolveMetadata(URI.create(transcodedUrl))).thenReturn(VideoMetadata.builder()
                .contentType(contentType)
                .duration(duration)
                .build());
        when(subtitleService.getActiveSubtitle()).thenReturn(Optional.empty());
        when(transcodeService.transcode(url)).thenReturn(transcodedUrl);

        var result = service.toMediaRequest(request);

        assertEquals(expectedResult, result);
        assertEquals(metadata, result.metadata);
    }

    private static Map<String, Object> createMetadata(PlayRequest request, String subtitleUri) {
        return new HashMap<>() {{
            put(Media.METADATA_TYPE, Media.MetadataType.MOVIE);
            put(Media.METADATA_TITLE, request.getTitle().orElse(null));
            put(Media.METADATA_SUBTITLE, request.getQuality().orElse(null));
            put(ChromeCastMetaData.METADATA_SUBTITLES, subtitleUri);
            put(ChromeCastMetaData.METADATA_THUMBNAIL, request.getThumbnail().orElse(null));
            put(ChromeCastMetaData.METADATA_THUMBNAIL_URL, request.getThumbnail().orElse(null));
            put(ChromeCastMetaData.METADATA_POSTER_URL, request.getThumbnail().orElse(null));
        }};
    }
}