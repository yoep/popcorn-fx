package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.SubtitleSettings;
import javafx.scene.control.CheckBox;
import javafx.scene.control.ComboBox;
import javafx.scene.control.TextField;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.junit.jupiter.api.io.TempDir;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import java.io.File;
import java.net.URL;
import java.util.ResourceBundle;

import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.verify;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class SettingsSubtitlesComponentTest {
    @Mock
    private ApplicationConfig applicationConfig;
    @Mock
    private LocaleText localeText;
    @Mock
    private ApplicationSettings applicationSettings;
    @Mock
    private SubtitleSettings settings;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private SettingsSubtitlesComponent component;
    @TempDir
    public File workingDir;

    @BeforeEach
    void setUp() {
        lenient().when(applicationConfig.getSettings()).thenReturn(applicationSettings);
        lenient().when(applicationSettings.getSubtitleSettings()).thenReturn(settings);
        lenient().when(settings.getDirectory()).thenReturn(workingDir.getAbsolutePath());
        component.clearCache = new CheckBox();
        component.defaultSubtitle = new ComboBox<>();
        component.fontFamily = new ComboBox<>();
        component.decoration = new ComboBox<>();
        component.fontSize = new ComboBox<>();
        component.fontBold = new CheckBox();
        component.cacheDirectory = new TextField();
    }

    @Test
    void testChangeClearCache_shouldUpdateSettings() {
        component.initialize(url, resourceBundle);

        component.clearCache.setSelected(true);

        verify(settings).setAutoCleaningEnabled(true);
        verify(applicationConfig).update(settings);
    }
}