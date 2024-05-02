package com.github.yoep.video.javafx;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import static org.junit.jupiter.api.Assertions.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class VideoPlayerFXTest {
    @Test
    void testNewInstance() {
        var player = new VideoPlayerFX();
        WaitForAsyncUtils.waitForFxEvents();

        var result = player.isInitialized();

        assertTrue(result, "expected the player to have been initialized");
    }

    @Test
    void testName() {
        var player = new VideoPlayerFX();

        var result = player.getName();

        assertEquals(VideoPlayerFX.NAME, result);
    }

    @Test
    void testDescription() {
        var player = new VideoPlayerFX();

        var result = player.getDescription();

        assertEquals(VideoPlayerFX.DESCRIPTION, result);
    }

    @Test
    void testGetVideoSurface() {
        var player = new VideoPlayerFX();
        WaitForAsyncUtils.waitForFxEvents();

        var result = player.getVideoSurface();

        assertNotNull(result, "expected the player to have a video surface");
    }
}