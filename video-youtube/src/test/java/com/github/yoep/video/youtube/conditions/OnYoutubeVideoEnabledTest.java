package com.github.yoep.video.youtube.conditions;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class OnYoutubeVideoEnabledTest {
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;

    @Test
    void testMatches_whenDisableOptionIsNotPresent_shouldReturnTrue() {
        when(fxLib.is_youtube_video_player_disabled(instance)).thenReturn((byte) 0);

        var result = OnYoutubeVideoEnabled.matches(fxLib, instance);

        assertTrue(result);
    }

    @Test
    void testMatches_whenBeanFactoryIsNotPresent_shouldReturnTrue() {
        var result = OnYoutubeVideoEnabled.matches(fxLib, instance);

        assertTrue(result);
    }

    @Test
    void testMatches_whenDisableOptionIsPresent_shouldReturnFalse() {
        when(fxLib.is_youtube_video_player_disabled(instance)).thenReturn((byte) 1);

        var result = OnYoutubeVideoEnabled.matches(fxLib, instance);

        assertFalse(result);
    }
}