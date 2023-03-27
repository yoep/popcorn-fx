package com.github.yoep.popcorn.backend.settings;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.boot.ApplicationArguments;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class OptionsServiceTest {
    @Mock
    private ApplicationArguments arguments;
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @InjectMocks
    private OptionsService optionsService;

    @Test
    void testIsTvMode() {
        when(fxLib.is_tv_mode(instance)).thenReturn((byte) 1);

        var result = optionsService.isTvMode();

        assertTrue(result);
    }

    @Test
    void testIsMaximized() {
        when(fxLib.is_maximized(instance)).thenReturn((byte) 1);

        var result = optionsService.isMaximized();

        assertTrue(result);
    }

    @Test
    void testIsKioskMode() {
        when(fxLib.is_kiosk_mode(instance)).thenReturn((byte) 1);

        var result = optionsService.isKioskMode();

        assertTrue(result);
    }
}
