package com.github.yoep.popcorn.backend.settings;

import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.junit.jupiter.api.io.TempDir;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.io.File;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class ApplicationConfigTest {
    @Mock
    private LocaleText localeText;
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @TempDir
    public File workingDir;

    @Test
    void testInit() {
        var settings = mock(ApplicationSettings.class);
        var uiSettings = mock(UISettings.class);
        when(fxLib.application_settings(instance)).thenReturn(settings);
        when(settings.getUiSettings()).thenReturn(uiSettings);
        when(uiSettings.getUiScale()).thenReturn(mock(UIScale.class));
        when(uiSettings.getDefaultLanguage()).thenReturn("en-US");

        new ApplicationConfig(fxLib, instance, localeText);

        verify(fxLib).register_settings_callback(eq(instance), isA(ApplicationConfigEvent.class));
    }

    @Test
    void testGetSettings() {
        var settings = mock(ApplicationSettings.class);
        var uiSettings = mock(UISettings.class);
        when(fxLib.application_settings(instance)).thenReturn(settings);
        when(settings.getUiSettings()).thenReturn(uiSettings);
        when(uiSettings.getUiScale()).thenReturn(mock(UIScale.class));
        when(uiSettings.getDefaultLanguage()).thenReturn("en-US");
        var config = new ApplicationConfig(fxLib, instance, localeText);

        var result = config.getSettings();

        assertEquals(settings, result);
    }

    @Test
    void testUpdateSubtitleSettings() {
        var settings = mock(ApplicationSettings.class);
        var uiSettings = mock(UISettings.class);
        when(fxLib.application_settings(instance)).thenReturn(settings);
        when(settings.getUiSettings()).thenReturn(uiSettings);
        when(uiSettings.getUiScale()).thenReturn(mock(UIScale.class));
        when(uiSettings.getDefaultLanguage()).thenReturn("en-US");
        var subtitleSettings = new SubtitleSettings();
        subtitleSettings.directory = workingDir.getAbsolutePath();
        subtitleSettings.autoCleaningEnabled = (byte) 1;
        var expected = new SubtitleSettings.ByValue();
        expected.directory = workingDir.getAbsolutePath();
        expected.autoCleaningEnabled = (byte) 1;
        var config = new ApplicationConfig(fxLib, instance, localeText);

        config.update(subtitleSettings);

        verify(fxLib).update_subtitle_settings(instance, expected);
    }

    @Test
    void testUpdateTorrentSettings() {
        var settings = mock(ApplicationSettings.class);
        var uiSettings = mock(UISettings.class);
        when(fxLib.application_settings(instance)).thenReturn(settings);
        when(settings.getUiSettings()).thenReturn(uiSettings);
        when(uiSettings.getUiScale()).thenReturn(mock(UIScale.class));
        when(uiSettings.getDefaultLanguage()).thenReturn("en-US");
        var torrentSettings = new TorrentSettings();
        torrentSettings.cleaningMode = CleaningMode.ON_SHUTDOWN;
        var expected = new TorrentSettings.ByValue();
        expected.cleaningMode = CleaningMode.ON_SHUTDOWN;
        var config = new ApplicationConfig(fxLib, instance, localeText);

        config.update(torrentSettings);

        verify(fxLib).update_torrent_settings(instance, expected);
    }

    @Test
    void testIsTvMode() {
        var settings = mock(ApplicationSettings.class);
        var uiSettings = mock(UISettings.class);
        when(fxLib.application_settings(instance)).thenReturn(settings);
        when(settings.getUiSettings()).thenReturn(uiSettings);
        when(uiSettings.getUiScale()).thenReturn(mock(UIScale.class));
        when(uiSettings.getDefaultLanguage()).thenReturn("en-US");
        when(fxLib.is_tv_mode(instance)).thenReturn((byte) 1);
        var config = new ApplicationConfig(fxLib, instance, localeText);

        var result = config.isTvMode();

        assertTrue(result);
    }

    @Test
    void testIsMaximized() {
        var settings = mock(ApplicationSettings.class);
        var uiSettings = mock(UISettings.class);
        when(fxLib.application_settings(instance)).thenReturn(settings);
        when(settings.getUiSettings()).thenReturn(uiSettings);
        when(uiSettings.getUiScale()).thenReturn(mock(UIScale.class));
        when(uiSettings.getDefaultLanguage()).thenReturn("en-US");
        when(fxLib.is_maximized(instance)).thenReturn((byte) 1);
        var config = new ApplicationConfig(fxLib, instance, localeText);

        var result = config.isMaximized();

        assertTrue(result);
    }

    @Test
    void testIsKioskMode() {
        var settings = mock(ApplicationSettings.class);
        var uiSettings = mock(UISettings.class);
        when(fxLib.application_settings(instance)).thenReturn(settings);
        when(settings.getUiSettings()).thenReturn(uiSettings);
        when(uiSettings.getUiScale()).thenReturn(mock(UIScale.class));
        when(uiSettings.getDefaultLanguage()).thenReturn("en-US");
        when(fxLib.is_kiosk_mode(isA(PopcornFx.class))).thenReturn((byte) 1);
        var config = new ApplicationConfig(fxLib, instance, localeText);

        var result = config.isKioskMode();

        assertTrue(result);
        verify(fxLib).is_kiosk_mode(instance);
    }

    @Test
    void testIsFxPlayerEnabled() {
        var settings = mock(ApplicationSettings.class);
        var uiSettings = mock(UISettings.class);
        when(fxLib.application_settings(instance)).thenReturn(settings);
        when(settings.getUiSettings()).thenReturn(uiSettings);
        when(uiSettings.getUiScale()).thenReturn(mock(UIScale.class));
        when(uiSettings.getDefaultLanguage()).thenReturn("en-US");
        when(fxLib.is_fx_video_player_enabled(isA(PopcornFx.class))).thenReturn((byte) 1);
        var config = new ApplicationConfig(fxLib, instance, localeText);

        var result = config.isFxPlayerEnabled();

        assertTrue(result);
        verify(fxLib).is_fx_video_player_enabled(instance);
    }
}