package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.SubtitleSettings;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import javafx.scene.control.Button;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import java.net.URL;
import java.util.ResourceBundle;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class TvSettingsSubtitlesComponentTest {
    @Mock
    private ApplicationConfig applicationConfig;
    @Mock
    private ApplicationSettings settings;
    @Mock
    private SubtitleSettings subtitleSettings;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    TvSettingsSubtitlesComponent component;

    @BeforeEach
    void setUp() {
        when(applicationConfig.getSettings()).thenReturn(settings);
        when(settings.getSubtitleSettings()).thenReturn(subtitleSettings);

        component.defaultSubtitle = new Button();
        component.subtitles = new AxisItemSelection<>();
        component.defaultSubtitleOverlay = spy(new Overlay());
        component.fontFamily = new Button();
        component.fontFamilies = new AxisItemSelection<>();
        component.fontFamilyOverlay = spy(new Overlay());
        component.fontSize = new Button();
        component.fontSizes = new AxisItemSelection<>();
        component.fontSizeOverlay = spy(new Overlay());
    }

    @Test
    void testOnSubtitleChanged() {
        when(subtitleSettings.getDefaultSubtitle()).thenReturn(SubtitleLanguage.BOSNIAN);
        component.initialize(url, resourceBundle);

        component.subtitles.setSelectedItem(SubtitleLanguage.BULGARIAN);

        verify(component.defaultSubtitleOverlay, times(2)).hide();
        verify(applicationConfig, atLeast(2)).update(subtitleSettings);
        verify(subtitleSettings).setDefaultSubtitle(SubtitleLanguage.BULGARIAN);
        assertEquals(SubtitleLanguage.BULGARIAN.getNativeName(), component.defaultSubtitle.getText());
    }

    @Test
    void testOnFontSizeChanged() {
        when(subtitleSettings.getFontSize()).thenReturn(28);
        component.initialize(url, resourceBundle);

        component.fontSizes.setSelectedItem(32);

        verify(component.fontSizeOverlay, times(2)).hide();
        verify(applicationConfig, times(2)).update(subtitleSettings);
        verify(subtitleSettings).setFontSize(32);
        assertEquals("32", component.fontSize.getText());
    }
}