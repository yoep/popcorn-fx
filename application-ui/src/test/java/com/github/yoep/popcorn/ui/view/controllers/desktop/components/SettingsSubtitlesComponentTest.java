package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import javafx.scene.control.CheckBox;
import javafx.scene.control.ComboBox;
import javafx.scene.control.TextField;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.junit.jupiter.api.io.TempDir;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import java.io.File;
import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class SettingsSubtitlesComponentTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private ApplicationConfig applicationConfig;
    @Mock
    private LocaleText localeText;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    private SettingsSubtitlesComponent component;
    @TempDir
    public File workingDir;

    @BeforeEach
    void setUp() {
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setSubtitleSettings(ApplicationSettings.SubtitleSettings.newBuilder()
                        .setDirectory(workingDir.getAbsolutePath())
                        .build()
                )
                .build()));

        component = new SettingsSubtitlesComponent(eventPublisher, localeText, applicationConfig);

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
        var settings = new AtomicReference<ApplicationSettings.SubtitleSettings>();
        doAnswer(invocations -> {
            settings.set(invocations.getArgument(0, ApplicationSettings.SubtitleSettings.class));
            return null;
        }).when(applicationConfig).update(isA(ApplicationSettings.SubtitleSettings.class));
        component.initialize(url, resourceBundle);

        component.clearCache.setSelected(true);

        verify(applicationConfig).update(isA(ApplicationSettings.SubtitleSettings.class));
        assertTrue(settings.get().getAutoCleaningEnabled(), "expected auto cleaning to have been enabled");
    }
}