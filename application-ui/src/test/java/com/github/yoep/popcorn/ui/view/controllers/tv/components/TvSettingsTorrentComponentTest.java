package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
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
class TvSettingsTorrentComponentTest {
    @Mock
    private ApplicationConfig applicationConfig;
    @Mock
    private LocaleText localeText;
    @Mock
    private ApplicationSettings settings;
    @Mock
    private com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings.TorrentSettings torrentSettings;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private TvSettingsTorrentComponent component;

    @BeforeEach
    void setUp() {
        when(applicationConfig.getSettings()).thenReturn(settings);
        when(settings.getTorrentSettings()).thenReturn(torrentSettings);

        component.cacheCleanup = new Button();
        component.cacheCleanupOverlay = spy(new Overlay());
        component.cleanupModes = new AxisItemSelection<>();
    }

    @Test
    void testOnCleaningModeChanged() {
        var text = "Lorem";
        when(torrentSettings.getCleaningMode()).thenReturn(com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings.TorrentSettings.CleaningMode.OFF);
        when(localeText.get(TvSettingsTorrentComponent.CLEANING_MODE_PREFIX + "off")).thenReturn("Invalid");
        when(localeText.get(TvSettingsTorrentComponent.CLEANING_MODE_PREFIX + "on_shutdown")).thenReturn(text);
        component.initialize(url, resourceBundle);

        component.cleanupModes.setSelectedItem(com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings.TorrentSettings.CleaningMode.ON_SHUTDOWN);

        verify(component.cacheCleanupOverlay, times(2)).hide();
        verify(applicationConfig, atLeast(2)).update(torrentSettings);
        assertEquals(text, component.cacheCleanup.getText());
    }
}