package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import com.github.yoep.popcorn.ui.view.controls.VirtualKeyboard;
import javafx.event.ActionEvent;
import javafx.scene.control.Button;
import javafx.scene.control.CheckBox;
import javafx.scene.control.Label;
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
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class TvSettingsServerComponentTest {
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
    private TvSettingsServerComponent component;

    @BeforeEach
    void setUp() {
        component = new TvSettingsServerComponent(eventPublisher, localeText, applicationConfig);

        component.movieServersBtn = new Button();
        component.movieServersOverlay = new Overlay();
        component.movieServersTxt = new Label();
        component.movieServersInput = new VirtualKeyboard();

        component.seriesServersBtn = new Button();
        component.serieServersOverlay = new Overlay();
        component.serieServersTxt = new Label();
        component.serieServersInput = new VirtualKeyboard();

        component.updateServersAutomatically = new CheckBox();
    }

    @Test
    void testInitialize_shouldLoadSettings() throws TimeoutException {
        var movieServers = "https://FooBar.com";
        var serieServers1 = "https://LoremIpsum.com";
        var serieServers2 = "https://LoremIpsum2.com";
        var expectedSerieServersText = String.join(",", serieServers1, serieServers2);
        var updateServersAutomatically = true;
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setServerSettings(ApplicationSettings.ServerSettings.newBuilder()
                        .addMovieApiServers(movieServers)
                        .addSerieApiServers(serieServers1)
                        .addSerieApiServers(serieServers2)
                        .setUpdateApiServersAutomatically(updateServersAutomatically)
                        .build())
                .build()));
        component.initialize(url, resourceBundle);

        WaitForAsyncUtils.waitForFxEvents();
        WaitForAsyncUtils.waitFor(500, TimeUnit.MILLISECONDS, () -> !component.serieServersTxt.getText().isEmpty());

        assertEquals(movieServers, component.movieServersTxt.getText());
        assertEquals(expectedSerieServersText, component.serieServersTxt.getText());
        assertEquals(updateServersAutomatically, component.updateServersAutomatically.isSelected());
    }

    @Test
    void testOnCloseInputOverlay() {
        var event = mock(ActionEvent.class);
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setServerSettings(ApplicationSettings.ServerSettings.newBuilder()
                        .build())
                .build()));
        component.initialize(url, resourceBundle);

        component.onCloseInputOverlay(event);

        verify(event).consume();
        assertFalse(component.movieServersOverlay.isShown(), "expected movieServersOverlay to be hidden");
        assertFalse(component.serieServersOverlay.isShown(), "expected serieServersOverlay to be hidden");
    }
}