package com.github.yoep.video.javafx;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import static org.junit.jupiter.api.Assertions.assertTrue;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class VideoPlayerFXTest {
    @Test
    void testNewInstance() {
        var player = new VideoPlayerFX();

        var result = player.isInitialized();

        assertTrue(result, "expected the player to have been initialized");
    }
}