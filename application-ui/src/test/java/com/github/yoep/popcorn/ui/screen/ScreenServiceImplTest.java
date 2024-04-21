package com.github.yoep.popcorn.ui.screen;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.screen.FullscreenCallback;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.ui.view.ViewManager;
import com.github.yoep.popcorn.ui.view.services.MaximizeService;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.ReadOnlyProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.beans.value.ChangeListener;
import javafx.scene.input.KeyCombination;
import javafx.stage.Stage;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class ScreenServiceImplTest {
    @Mock
    private ViewManager viewManager;
    @Mock
    private ApplicationConfig applicationConfig;
    @Mock
    private MaximizeService maximizeService;
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private ReadOnlyProperty<Stage> primaryStageProperty;
    @InjectMocks
    private ScreenServiceImpl screenService;

    private final AtomicReference<ChangeListener<Stage>> primaryStageListener = new AtomicReference<>();
    private final BooleanProperty fullscreenProperty = new SimpleBooleanProperty();

    @BeforeEach
    void setUp() {
        doAnswer(invocation -> {
            primaryStageListener.set(invocation.getArgument(0));
            return null;
        }).when(primaryStageProperty).addListener(isA(ChangeListener.class));
        when(viewManager.primaryStageProperty()).thenReturn(primaryStageProperty);
    }

    @Test
    void testInit() {
        screenService.init();

        verify(fxLib).register_fullscreen_callback(eq(instance), isA(FullscreenCallback.class));
    }

    @Test
    void testToggleFullscreen() {
        var primaryStage = mock(Stage.class);
        var future = new CompletableFuture<Boolean>();
        doAnswer(invocation -> {
            fullscreenProperty.set(invocation.getArgument(0, Boolean.class));
            return null;
        }).when(primaryStage).setFullScreen(isA(Boolean.class));
        when(viewManager.primaryStageProperty()).thenReturn(primaryStageProperty);
        when(primaryStage.fullScreenProperty()).thenReturn(fullscreenProperty);
        when(applicationConfig.isKioskMode()).thenReturn(false);
        screenService.fullscreenProperty().addListener((observable, oldValue, newValue) -> future.complete(newValue));
        screenService.init();

        primaryStageListener.get().changed(null, null, primaryStage);
        screenService.toggleFullscreen();

        assertEquals(true, future.join());
        assertTrue(screenService.isFullscreen());
    }

    @Test
    void testRegisterListenerOnKioskMode() {
        var primaryStage = mock(Stage.class);
        when(viewManager.primaryStageProperty()).thenReturn(primaryStageProperty);
        when(primaryStage.fullScreenProperty()).thenReturn(fullscreenProperty);
        when(applicationConfig.isKioskMode()).thenReturn(true);
        screenService.init();

        primaryStageListener.get().changed(null, null, primaryStage);

        verify(maximizeService).setMaximized(false);
        verify(primaryStage).setFullScreenExitKeyCombination(KeyCombination.NO_MATCH);
        verify(primaryStage).setFullScreen(true);
    }
}