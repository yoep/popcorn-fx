package com.github.yoep.player.chromecast.controllers;

import com.github.yoep.player.chromecast.services.ChromecastService;
import org.apache.commons.io.IOUtils;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.core.io.InputStreamResource;
import org.springframework.http.HttpStatus;

import java.nio.charset.StandardCharsets;
import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class SubtitleControllerTest {
    @Mock
    private ChromecastService chromecastService;
    @InjectMocks
    private SubtitleController controller;

    @Test
    void testRetrieveSubtitle_whenSubtitleIsNotFound_shouldReturnNotFound() {
        var subtitle = "my-file.vtt";
        when(chromecastService.retrieveVttSubtitle(subtitle)).thenReturn(Optional.empty());

        var result = controller.retrieveSubtitle(subtitle);

        assertEquals(HttpStatus.NOT_FOUND, result.getStatusCode());
    }

    @Test
    void testRetrieveSubtitle_WhenSubtitleIsKnown_shouldReturnContents() {
        var subtitle = "my-file.vtt";
        var expectedContent = "lorem subtitle";
        var inputStream = IOUtils.toInputStream(expectedContent, StandardCharsets.UTF_8);
        when(chromecastService.retrieveVttSubtitle(subtitle)).thenReturn(Optional.of(inputStream));

        var result = controller.retrieveSubtitle(subtitle);

        assertEquals(HttpStatus.OK, result.getStatusCode());
        assertEquals(new InputStreamResource(inputStream), result.getBody());
    }
}