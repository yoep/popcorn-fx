package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import javafx.scene.control.CheckBox;
import javafx.scene.control.ComboBox;
import javafx.scene.control.TextField;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
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

    @BeforeEach
    void setUp() {
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setSubtitleSettings(ApplicationSettings.SubtitleSettings.newBuilder().build())
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
    void testOnDefaultSubtitleChanged() {
        var newValue = Subtitle.Language.HEBREW;
        var request = new AtomicReference<ApplicationSettings.SubtitleSettings>();
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setSubtitleSettings(ApplicationSettings.SubtitleSettings.newBuilder()
                        .setDefaultSubtitle(Subtitle.Language.CROATIAN)
                        .build())
                .build()));
        doAnswer(invocations -> {
            request.set(invocations.getArgument(0, ApplicationSettings.SubtitleSettings.class));
            return null;
        }).when(applicationConfig).update(isA(ApplicationSettings.SubtitleSettings.class));
        component.initialize(url, resourceBundle);
        WaitForAsyncUtils.waitForFxEvents();

        component.defaultSubtitle.getSelectionModel().select(newValue);
        WaitForAsyncUtils.waitForFxEvents();

        verify(applicationConfig).update(isA(ApplicationSettings.SubtitleSettings.class));
        assertEquals(newValue, request.get().getDefaultSubtitle());
    }

    @Test
    void testOnFontFamilyChanged() {
        var newValue = ApplicationSettings.SubtitleSettings.Family.COMIC_SANS;
        var request = new AtomicReference<ApplicationSettings.SubtitleSettings>();
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setSubtitleSettings(ApplicationSettings.SubtitleSettings.newBuilder()
                        .setFontFamily(ApplicationSettings.SubtitleSettings.Family.ARIAL)
                        .build())
                .build()));
        doAnswer(invocations -> {
            request.set(invocations.getArgument(0, ApplicationSettings.SubtitleSettings.class));
            return null;
        }).when(applicationConfig).update(isA(ApplicationSettings.SubtitleSettings.class));
        component.initialize(url, resourceBundle);
        WaitForAsyncUtils.waitForFxEvents();

        component.fontFamily.getSelectionModel().select(newValue);
        WaitForAsyncUtils.waitForFxEvents();

        verify(applicationConfig).update(isA(ApplicationSettings.SubtitleSettings.class));
        assertEquals(newValue, request.get().getFontFamily());
    }

    @Test
    void testOnDecorationChanged() {
        var newValue = ApplicationSettings.SubtitleSettings.DecorationType.OPAQUE_BACKGROUND;
        var request = new AtomicReference<ApplicationSettings.SubtitleSettings>();
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setSubtitleSettings(ApplicationSettings.SubtitleSettings.newBuilder()
                        .setDecoration(ApplicationSettings.SubtitleSettings.DecorationType.NONE)
                        .build())
                .build()));
        doAnswer(invocations -> {
            request.set(invocations.getArgument(0, ApplicationSettings.SubtitleSettings.class));
            return null;
        }).when(applicationConfig).update(isA(ApplicationSettings.SubtitleSettings.class));
        component.initialize(url, resourceBundle);
        WaitForAsyncUtils.waitForFxEvents();

        component.decoration.getSelectionModel().select(newValue);
        WaitForAsyncUtils.waitForFxEvents();

        verify(applicationConfig).update(isA(ApplicationSettings.SubtitleSettings.class));
        assertEquals(newValue, request.get().getDecoration());
    }

    @Test
    void testOnFontSizeChanged() {
        var newValue = 32;
        var request = new AtomicReference<ApplicationSettings.SubtitleSettings>();
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setSubtitleSettings(ApplicationSettings.SubtitleSettings.newBuilder()
                        .setFontSize(12)
                        .build())
                .build()));
        doAnswer(invocations -> {
            request.set(invocations.getArgument(0, ApplicationSettings.SubtitleSettings.class));
            return null;
        }).when(applicationConfig).update(isA(ApplicationSettings.SubtitleSettings.class));
        component.initialize(url, resourceBundle);
        WaitForAsyncUtils.waitForFxEvents();

        component.fontSize.getSelectionModel().select((Integer) newValue);
        WaitForAsyncUtils.waitForFxEvents();

        verify(applicationConfig).update(isA(ApplicationSettings.SubtitleSettings.class));
        assertEquals(newValue, request.get().getFontSize());
    }

    @Test
    void testOnBoldChanged() {
        var newValue = true;
        var request = new AtomicReference<ApplicationSettings.SubtitleSettings>();
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setSubtitleSettings(ApplicationSettings.SubtitleSettings.newBuilder()
                        .setBold(false)
                        .build())
                .build()));
        doAnswer(invocations -> {
            request.set(invocations.getArgument(0, ApplicationSettings.SubtitleSettings.class));
            return null;
        }).when(applicationConfig).update(isA(ApplicationSettings.SubtitleSettings.class));
        component.initialize(url, resourceBundle);
        WaitForAsyncUtils.waitForFxEvents();

        component.fontBold.setSelected(newValue);
        WaitForAsyncUtils.waitForFxEvents();

        verify(applicationConfig).update(isA(ApplicationSettings.SubtitleSettings.class));
        assertEquals(newValue, request.get().getBold());
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