package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.subtitles.SubtitleHelper;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import javafx.scene.control.Button;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class TvSettingsSubtitlesComponentTest {
    @Mock
    private ApplicationConfig applicationConfig;
    @Mock
    private LocaleText localeText;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    TvSettingsSubtitlesComponent component;

    @BeforeEach
    void setUp() {
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setSubtitleSettings(ApplicationSettings.SubtitleSettings.newBuilder()
                        .setFontSize(12)
                        .build())
                .build()));

        component = new TvSettingsSubtitlesComponent(applicationConfig, localeText);

        component.defaultSubtitle = new Button();
        component.subtitles = new AxisItemSelection<>();
        component.defaultSubtitleOverlay = spy(new Overlay());
        component.fontFamily = new Button();
        component.fontFamilies = new AxisItemSelection<>();
        component.fontFamilyOverlay = spy(new Overlay());
        component.decoration = new Button();
        component.decorations = new AxisItemSelection<>();
        component.decorationOverlay = spy(new Overlay());
        component.fontSize = new Button();
        component.fontSizes = new AxisItemSelection<>();
        component.fontSizeOverlay = spy(new Overlay());
    }

    @Test
    void testOnSubtitleChanged() {
        var settings = new AtomicReference<ApplicationSettings.SubtitleSettings>();
        doAnswer(invocations -> {
            settings.set(invocations.getArgument(0, ApplicationSettings.SubtitleSettings.class));
            return null;
        }).when(applicationConfig).update(isA(ApplicationSettings.SubtitleSettings.class));
        component.initialize(url, resourceBundle);
        WaitForAsyncUtils.waitForFxEvents();

        component.subtitles.setSelectedItem(Subtitle.Language.BULGARIAN);
        WaitForAsyncUtils.waitForFxEvents();

        verify(component.defaultSubtitleOverlay).hide();
        verify(applicationConfig).update(isA(ApplicationSettings.SubtitleSettings.class));
        assertEquals(Subtitle.Language.BULGARIAN, settings.get().getDefaultSubtitle());
        assertEquals(SubtitleHelper.getNativeName(Subtitle.Language.BULGARIAN), component.defaultSubtitle.getText());
    }

    @Test
    void testOnFontSizeChanged() {
        var settings = new AtomicReference<ApplicationSettings.SubtitleSettings>();
        doAnswer(invocations -> {
            settings.set(invocations.getArgument(0, ApplicationSettings.SubtitleSettings.class));
            return null;
        }).when(applicationConfig).update(isA(ApplicationSettings.SubtitleSettings.class));
        component.initialize(url, resourceBundle);
        WaitForAsyncUtils.waitForFxEvents();

        component.fontSizes.setSelectedItem(32);
        WaitForAsyncUtils.waitForFxEvents();

        verify(component.fontSizeOverlay).hide();
        verify(applicationConfig).update(isA(ApplicationSettings.SubtitleSettings.class));
        assertEquals(32, settings.get().getFontSize());
        assertEquals("32", component.fontSize.getText());
    }

    @Test
    void testOnDecorationChanged() {
        var expectedText = "lorem";
        var settings = new AtomicReference<ApplicationSettings.SubtitleSettings>();
        doAnswer(invocations -> {
            settings.set(invocations.getArgument(0, ApplicationSettings.SubtitleSettings.class));
            return null;
        }).when(applicationConfig).update(isA(ApplicationSettings.SubtitleSettings.class));
        when(localeText.get("settings_subtitles_style_outline")).thenReturn(expectedText);
        when(localeText.get("settings_subtitles_style_none")).thenReturn("none");
        component.initialize(url, resourceBundle);
        WaitForAsyncUtils.waitForFxEvents();

        component.decorations.setSelectedItem(ApplicationSettings.SubtitleSettings.DecorationType.OUTLINE);
        WaitForAsyncUtils.waitForFxEvents();

        verify(component.decorationOverlay).hide();
        verify(applicationConfig).update(isA(ApplicationSettings.SubtitleSettings.class));
        assertEquals(ApplicationSettings.SubtitleSettings.DecorationType.OUTLINE, settings.get().getDecoration());
        assertEquals(expectedText, component.decoration.getText());
    }
}