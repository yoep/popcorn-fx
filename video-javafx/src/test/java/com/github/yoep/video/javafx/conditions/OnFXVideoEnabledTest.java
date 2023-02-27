package com.github.yoep.video.javafx.conditions;

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
class OnFXVideoEnabledTest {
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;

    @Test
    void testMatches_whenIsDisabled_shouldReturnFalse() {
        when(fxLib.is_fx_video_player_disabled(instance)).thenReturn((byte) 1);

        var result = OnFXVideoEnabled.matches(fxLib, instance);

        assertFalse(result);
    }

    @Test
    void testMatches_whenIsEnabled_shouldReturnTrue() {
        when(fxLib.is_fx_video_player_disabled(instance)).thenReturn((byte) 0);

        var result = OnFXVideoEnabled.matches(fxLib, instance);

        assertTrue(result);
    }
}