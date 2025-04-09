package com.github.yoep.popcorn.ui;

import com.github.yoep.popcorn.backend.PopcornFx;
import javafx.scene.Cursor;
import javafx.scene.Scene;
import javafx.stage.Stage;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class PopcornTimePreloaderTest {
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @InjectMocks
    private PopcornTimePreloader controller;

    @Test
    void testOnMouseDisabled() {
        var stage = mock(Stage.class);
        var scene = mock(Scene.class);
        when(fxLib.is_mouse_disabled(instance)).thenReturn((byte) 1);

        controller.processParameters(stage, scene);

        verify(scene).setCursor(Cursor.NONE);
    }

    @Test
    void testOnKioskMode() {
        var stage = mock(Stage.class);
        var scene = mock(Scene.class);
        when(fxLib.is_kiosk_mode(instance)).thenReturn((byte) 1);

        controller.processParameters(stage, scene);

        verify(stage).setMaximized(true);
    }
}