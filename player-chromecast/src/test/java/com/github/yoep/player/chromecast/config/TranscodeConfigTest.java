package com.github.yoep.player.chromecast.config;

import com.github.yoep.player.chromecast.transcode.NoOpTranscodeService;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class TranscodeConfigTest {

    @Test
    void testNoOpTranscodeService_whenInvoked_shouldReturnNoOpService() {
        var service = new TranscodeConfig().noOpTranscodeService();

        assertEquals(new NoOpTranscodeService(), service);
    }
}