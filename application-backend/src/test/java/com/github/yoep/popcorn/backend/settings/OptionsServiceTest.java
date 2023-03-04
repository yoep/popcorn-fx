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
    void testInit_whenBigPictureIsEnabled_shouldSetTheBigPictureOption() {
        when(arguments.containsOption(OptionsService.BIG_PICTURE_MODE_OPTION)).thenReturn(true);

        optionsService.init();

        assertTrue(optionsService.options().isBigPictureMode());
    }

    @Test
    void testInit_whenKioskModeIsEnabled_shouldSetTheKioskOption() {
        when(arguments.containsOption(OptionsService.BIG_PICTURE_MODE_OPTION)).thenReturn(false);
        when(arguments.containsOption(OptionsService.KIOSK_MODE_OPTION)).thenReturn(true);

        optionsService.init();

        assertTrue(optionsService.options().isKioskMode());
    }

    @Test
    void testInit_whenMouseIsDisabled_shouldSetTheMouseDisabledOption() {
        when(arguments.containsOption(OptionsService.BIG_PICTURE_MODE_OPTION)).thenReturn(false);
        when(arguments.containsOption(OptionsService.KIOSK_MODE_OPTION)).thenReturn(false);
        when(arguments.containsOption(OptionsService.DISABLE_MOUSE_OPTION)).thenReturn(true);

        optionsService.init();

        assertTrue(optionsService.options().isMouseDisabled());
    }

    @Test
    void testIsTvMode() {
        when(fxLib.is_tv_mode(instance)).thenReturn((byte)1);

        var result = optionsService.isTvMode();

        assertTrue(result);
    }

    @Test
    void testIsMaximized() {
        when(fxLib.is_maximized(instance)).thenReturn((byte)1);

        var result = optionsService.isMaximized();

        assertTrue(result);
    }
}
