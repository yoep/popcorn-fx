package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import com.github.yoep.popcorn.ui.view.controls.VirtualKeyboard;
import javafx.event.ActionEvent;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class TvSettingsServerComponentTest {
    @Mock
    private ApplicationConfig applicationConfig;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    private TvSettingsServerComponent component;

    @BeforeEach
    void setUp() {
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setServerSettings(ApplicationSettings.ServerSettings.newBuilder()
                        .build())
                .build()));

        component = new TvSettingsServerComponent(applicationConfig);

        component.apiServerBtn = new Button();
        component.apiServerTxt = new Label();
        component.apiServerOverlay = new Overlay();
        component.apiServerVirtualKeyboard = new VirtualKeyboard();
    }

    @Test
    void testOnCloseApiOverlay() {
        var event = mock(ActionEvent.class);
        component.initialize(url, resourceBundle);

        component.onCloseApiOverlay(event);

        verify(event).consume();
        assertFalse(component.apiServerOverlay.isShown(), "expected apiServerOverlay to be hidden");
    }
}