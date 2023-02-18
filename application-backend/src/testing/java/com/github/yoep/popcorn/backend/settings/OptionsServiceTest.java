package testing.java.com.github.yoep.popcorn.backend.settings;

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
    void testInit_whenTvModeIsEnabled_shouldSetTheTvModeOption() {
        when(arguments.containsOption(OptionsService.BIG_PICTURE_MODE_OPTION)).thenReturn(false);
        when(arguments.containsOption(OptionsService.KIOSK_MODE_OPTION)).thenReturn(false);
        when(arguments.containsOption(OptionsService.TV_MODE_OPTION)).thenReturn(true);

        optionsService.init();

        assertTrue(optionsService.options().isTvMode());
    }

    @Test
    void testInit_whenMaximizedIsEnabled_shouldSetTheMaximizedOption() {
        when(arguments.containsOption(OptionsService.BIG_PICTURE_MODE_OPTION)).thenReturn(false);
        when(arguments.containsOption(OptionsService.KIOSK_MODE_OPTION)).thenReturn(false);
        when(arguments.containsOption(OptionsService.TV_MODE_OPTION)).thenReturn(false);
        when(arguments.containsOption(OptionsService.MAXIMIZED_OPTION)).thenReturn(true);

        optionsService.init();

        assertTrue(optionsService.options().isMaximized());
    }

    @Test
    void testInit_whenMouseIsDisabled_shouldSetTheMouseDisabledOption() {
        when(arguments.containsOption(OptionsService.BIG_PICTURE_MODE_OPTION)).thenReturn(false);
        when(arguments.containsOption(OptionsService.KIOSK_MODE_OPTION)).thenReturn(false);
        when(arguments.containsOption(OptionsService.TV_MODE_OPTION)).thenReturn(false);
        when(arguments.containsOption(OptionsService.MAXIMIZED_OPTION)).thenReturn(false);
        when(arguments.containsOption(OptionsService.DISABLE_MOUSE_OPTION)).thenReturn(true);

        optionsService.init();

        assertTrue(optionsService.options().isMouseDisabled());
    }

    @Test
    void testInit_whenKeepAliveIsDisabled_shouldSetTheKeepAliveDisabledOption() {
        when(arguments.containsOption(OptionsService.BIG_PICTURE_MODE_OPTION)).thenReturn(false);
        when(arguments.containsOption(OptionsService.KIOSK_MODE_OPTION)).thenReturn(false);
        when(arguments.containsOption(OptionsService.TV_MODE_OPTION)).thenReturn(false);
        when(arguments.containsOption(OptionsService.MAXIMIZED_OPTION)).thenReturn(false);
        when(arguments.containsOption(OptionsService.DISABLE_MOUSE_OPTION)).thenReturn(false);
        when(arguments.containsOption(OptionsService.DISABLE_KEEP_ALIVE_OPTION)).thenReturn(true);

        optionsService.init();

        assertTrue(optionsService.options().isKeepAliveDisabled());
    }
}
