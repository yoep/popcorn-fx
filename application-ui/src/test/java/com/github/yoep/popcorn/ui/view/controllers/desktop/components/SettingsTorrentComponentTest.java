package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.TorrentSettings;
import com.github.yoep.popcorn.ui.view.controls.DelayedTextField;
import com.github.yoep.popcorn.ui.view.services.TorrentSettingService;
import javafx.scene.control.CheckBox;
import javafx.scene.control.TextField;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.junit.jupiter.api.io.TempDir;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.context.ApplicationEventPublisher;
import org.testfx.framework.junit5.ApplicationExtension;

import java.io.File;
import java.net.URL;
import java.util.ResourceBundle;

import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class SettingsTorrentComponentTest {
    @Mock
    private ApplicationEventPublisher eventPublisher;
    @Mock
    private LocaleText localeText;
    @Mock
    private ApplicationConfig applicationConfig;
    @Mock
    private ApplicationSettings applicationSettings;
    @Mock
    private TorrentSettings settings;
    @Mock
    private TorrentSettingService torrentSettingService;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private SettingsTorrentComponent component;
    @TempDir
    public File workingDir;

    @BeforeEach
    void setUp() {
        lenient().when(applicationConfig.getSettings()).thenReturn(applicationSettings);
        lenient().when(applicationSettings.getTorrentSettings()).thenReturn(settings);
        component.cacheDirectory = new TextField();
        component.connectionLimit = new DelayedTextField();
        component.downloadLimit = new DelayedTextField();
        component.uploadLimit = new DelayedTextField();
        component.clearCache = new CheckBox();
    }

    @Test
    void testChangeClearCache() {
        when(settings.getDirectory()).thenReturn(workingDir.getAbsolutePath());
        component.initialize(url, resourceBundle);

        component.clearCache.setSelected(true);

        verify(settings).setAutoCleaningEnabled(true);
        verify(applicationConfig).update(settings);
    }
}