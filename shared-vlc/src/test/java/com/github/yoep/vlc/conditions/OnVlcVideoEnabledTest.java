package com.github.yoep.vlc.conditions;

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
class OnVlcVideoEnabledTest {
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;

    @Test
    void testMatches_whenDisableVlcIsNotPresent_shouldReturnTrue() {
        when(fxLib.is_vlc_video_player_disabled(instance)).thenReturn((byte) 0);

        var result = OnVlcVideoEnabled.matches(fxLib, instance);

        assertTrue(result, "Expected the condition to match");
    }

    @Test
    void testMatches_whenDisableVlcIsPresent_shouldReturnFalse() {
        when(fxLib.is_vlc_video_player_disabled(instance)).thenReturn((byte) 1);

        var result = OnVlcVideoEnabled.matches(fxLib, instance);

        assertFalse(result, "Expected the condition be disabled");
    }
}