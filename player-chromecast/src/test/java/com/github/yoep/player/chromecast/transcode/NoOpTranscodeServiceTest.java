package com.github.yoep.player.chromecast.transcode;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertEquals;

@ExtendWith(MockitoExtension.class)
class NoOpTranscodeServiceTest {
    @InjectMocks
    private NoOpTranscodeService service;

    @Test
    void testTranscode_whenUrlIsGiven_shouldReturnTheSameUrl() {
        var url = "http://my-video-url";

        var result = service.transcode(url);

        assertEquals(url, result);
    }
}