package com.github.yoep.player.chromecast.services;

import com.github.yoep.player.chromecast.model.VideoMetadata;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.boot.autoconfigure.web.ServerProperties;

import java.net.URI;

import static org.junit.jupiter.api.Assertions.assertEquals;
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
}