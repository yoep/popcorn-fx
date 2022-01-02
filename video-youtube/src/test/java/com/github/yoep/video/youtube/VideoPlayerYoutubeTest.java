package com.github.yoep.video.youtube;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.junit.jupiter.MockitoExtension;

import java.text.MessageFormat;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

@ExtendWith(MockitoExtension.class)
class VideoPlayerYoutubeTest {
    @InjectMocks
    private VideoPlayerYoutube player;

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
}