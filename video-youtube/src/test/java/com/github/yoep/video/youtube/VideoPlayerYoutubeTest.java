package com.github.yoep.video.youtube;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.text.MessageFormat;

import static org.junit.jupiter.api.Assertions.*;

@ExtendWith({ApplicationExtension.class})
class VideoPlayerYoutubeTest {
    private VideoPlayerYoutube player;

    @BeforeEach
    void setUp() {
        player = new VideoPlayerYoutube();
    }

    @Test
    void testSupports_whenUrlDoesnNotContainIndicator_shouldReturnFalse() {
        var url = "http://localhost/video.mp4";

        var result = player.supports(url);

        assertFalse(result);
    }

    @Test
    void testSupports_whenUrlContainsIndicator_shouldReturnTrue() {
        var url = MessageFormat.format("http://{0}be.com/watch?v=1235556", VideoPlayerYoutube.YOUTUBE_URL_INDICATOR);

        var result = player.supports(url);

        assertTrue(result);
    }

    @Test
    void testGetName_shouldReturnTheNameOfThePlayer() {
        var result = player.getName();

        assertEquals(VideoPlayerYoutube.NAME, result);
    }

    @Test
    void testGetDescription_shouldReturnTheDescriptionOfThePlayer() {
        var result = player.getDescription();

        assertEquals(VideoPlayerYoutube.DESCRIPTION, result);
    }

    @Test
    void testInitialized_shouldReturnTrue() {
        var player = new VideoPlayerYoutube();
        WaitForAsyncUtils.waitForFxEvents(10);

        var result = player.isInitialized();

        assertTrue(result);
    }
}