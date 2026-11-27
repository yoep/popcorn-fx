package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings.PlaybackSettings;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings.PlaybackSettings.Quality;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import javafx.scene.control.Button;
import javafx.scene.control.CheckBox;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.ArgumentCaptor;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
@MockitoSettings(strictness = Strictness.LENIENT)
class TvSettingsPlaybackComponentTest {
    private static final String NONE_TEXT = "Not set";

    @Mock
    private EventPublisher eventPublisher;
    @Mock
    private ApplicationConfig applicationConfig;
    @Mock
    private LocaleText localeText;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private TvSettingsPlaybackComponent component;

    @BeforeEach
    void setUp() {
        when(localeText.get(TvSettingsPlaybackComponent.QUALITY_NONE_KEY)).thenReturn(NONE_TEXT);

        component.qualityButton = new Button();
        component.qualityOverlay = spy(new Overlay());
        component.qualities = new AxisItemSelection<>();
        component.fullscreen = new CheckBox();
        component.autoPlayNextEpisode = new CheckBox();
    }

    @Test
    void testInitialize_whenNoQualityIsSet() {
        initialize(PlaybackSettings.newBuilder()
                .setFullscreen(true)
                .setAutoPlayNextEpisodeEnabled(false)
                .build());

        assertEquals(NONE_TEXT, component.qualityButton.getText());
        assertTrue(component.fullscreen.isSelected());
        assertFalse(component.autoPlayNextEpisode.isSelected());
        verify(applicationConfig, never()).update(isA(PlaybackSettings.class));
    }

    @Test
    void testInitialize_whenQualityIsSet() {
        initialize(PlaybackSettings.newBuilder()
                .setQuality(Quality.P1080)
                .setFullscreen(false)
                .setAutoPlayNextEpisodeEnabled(true)
                .build());

        assertEquals("1080p", component.qualityButton.getText());
        assertFalse(component.fullscreen.isSelected());
        assertTrue(component.autoPlayNextEpisode.isSelected());
        verify(applicationConfig, never()).update(isA(PlaybackSettings.class));
    }

    @Test
    void testOnQualityChanged() {
        initialize(PlaybackSettings.newBuilder().build());

        component.qualities.setSelectedItem(Quality.P720);
        WaitForAsyncUtils.waitForFxEvents();

        var settings = captureUpdate();
        assertTrue(settings.hasQuality());
        assertEquals(Quality.P720, settings.getQuality());
        assertEquals("720p", component.qualityButton.getText());
        verify(component.qualityOverlay, atLeastOnce()).hide();
    }

    @Test
    void testOnQualityChanged_whenNoneIsSelected() {
        initialize(PlaybackSettings.newBuilder()
                .setQuality(Quality.P480)
                .build());

        component.qualities.setSelectedItem(Quality.P0);
        WaitForAsyncUtils.waitForFxEvents();

        var settings = captureUpdate();
        assertFalse(settings.hasQuality());
        assertEquals(NONE_TEXT, component.qualityButton.getText());
    }

    @Test
    void testOnFullscreenChanged() {
        initialize(PlaybackSettings.newBuilder().setFullscreen(false).build());

        component.fullscreen.setSelected(true);
        WaitForAsyncUtils.waitForFxEvents();

        assertTrue(captureUpdate().getFullscreen());
    }

    @Test
    void testOnAutoPlayNextEpisodeChanged() {
        initialize(PlaybackSettings.newBuilder().setAutoPlayNextEpisodeEnabled(false).build());

        component.autoPlayNextEpisode.setSelected(true);
        WaitForAsyncUtils.waitForFxEvents();

        assertTrue(captureUpdate().getAutoPlayNextEpisodeEnabled());
    }

    private void initialize(PlaybackSettings playbackSettings) {
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setPlaybackSettings(playbackSettings)
                .build()));

        component.initialize(url, resourceBundle);
        WaitForAsyncUtils.waitForFxEvents();
    }

    private PlaybackSettings captureUpdate() {
        var captor = ArgumentCaptor.forClass(PlaybackSettings.class);
        verify(applicationConfig, atLeastOnce()).update(captor.capture());
        return captor.getValue();
    }
}
