package com.github.yoep.popcorn.backend.settings;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.SubtitleSettings;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.junit.jupiter.api.io.TempDir;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.io.File;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class ApplicationConfigTest {
    @Mock
    private ViewLoader viewLoader;
    @Mock
    private OptionsService optionsService;
    @Mock
    private LocaleText localeText;
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @InjectMocks
    private ApplicationConfig config;
    @TempDir
    public File workingDir;

    @Test
    void testGetSettings() {
        var settings = mock(ApplicationSettings.class);
        when(fxLib.application_settings(instance)).thenReturn(settings);

        var result = config.getSettings();

        assertEquals(settings, result);
    }

    @Test
    void testUpdateSubtitleSettings() {
        var settings = new SubtitleSettings();
        settings.directory = workingDir.getAbsolutePath();
        settings.autoCleaningEnabled = (byte) 1;
        var expected = new SubtitleSettings.ByValue();
        expected.directory = workingDir.getAbsolutePath();
        expected.autoCleaningEnabled = (byte) 1;

        config.update(settings);

        verify(fxLib).update_subtitle_settings(instance, expected);
    }
}