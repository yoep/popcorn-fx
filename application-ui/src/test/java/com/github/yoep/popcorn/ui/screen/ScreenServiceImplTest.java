package com.github.yoep.popcorn.ui.screen;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayerStoppedEvent;
import com.github.yoep.popcorn.backend.lib.FxChannel;
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
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Function;

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
    private FxChannel fxChannel;
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private ReadOnlyProperty<Stage> primaryStageProperty;

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
        var service = new ScreenServiceImpl(viewManager, applicationConfig, eventPublisher, maximizeService, fxChannel);

        verify(eventPublisher).register(eq(PlayerStoppedEvent.class), isA(Function.class));
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
        var screenService = new ScreenServiceImpl(viewManager, applicationConfig, eventPublisher, maximizeService, fxChannel);
        screenService.fullscreenProperty().addListener((observable, oldValue, newValue) -> future.complete(newValue));

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
        var screenService = new ScreenServiceImpl(viewManager, applicationConfig, eventPublisher, maximizeService, fxChannel);

        primaryStageListener.get().changed(null, null, primaryStage);

        verify(maximizeService).setMaximized(false);
        verify(primaryStage).setFullScreenExitKeyCombination(KeyCombination.NO_MATCH);
        verify(primaryStage).setFullScreen(true);
    }
}