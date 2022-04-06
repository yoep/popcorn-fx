package com.github.yoep.player.chromecast.services;

import com.github.yoep.player.chromecast.model.VideoMetadata;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleType;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.boot.autoconfigure.web.ServerProperties;

import java.io.File;
import java.io.InputStream;
import java.net.InetAddress;
import java.net.URI;
import java.net.UnknownHostException;
import java.text.MessageFormat;
import java.util.Collections;
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
}