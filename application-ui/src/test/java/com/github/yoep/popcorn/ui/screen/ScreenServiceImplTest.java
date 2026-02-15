package com.github.yoep.popcorn.ui.screen;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.MaximizeEvent;
import com.github.yoep.popcorn.backend.events.PlayerStoppedEvent;
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
import org.testfx.util.WaitForAsyncUtils;

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
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private ReadOnlyProperty<Stage> primaryStageProperty;
    private ScreenServiceImpl service;

    private final AtomicReference<ChangeListener<Stage>> primaryStageListener = new AtomicReference<>();
    private final BooleanProperty fullscreenProperty = new SimpleBooleanProperty();

    @BeforeEach
    void setUp() {
        doAnswer(invocation -> {
            primaryStageListener.set(invocation.getArgument(0));
            return null;
        }).when(primaryStageProperty).addListener(isA(ChangeListener.class));
        when(viewManager.primaryStageProperty()).thenReturn(primaryStageProperty);

        service = new ScreenServiceImpl(viewManager, applicationConfig, eventPublisher, maximizeService);
    }

    @Test
    void testInit() {
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
        when(primaryStage.fullScreenProperty()).thenReturn(fullscreenProperty);
        when(applicationConfig.isKioskMode()).thenReturn(false);
        service.fullscreenProperty().addListener((observable, oldValue, newValue) -> future.complete(newValue));

        primaryStageListener.get().changed(null, null, primaryStage);
        service.toggleFullscreen();
        WaitForAsyncUtils.waitForFxEvents();

        assertEquals(true, future.join());
        assertTrue(service.isFullscreen());
    }

    @Test
    void testRegisterListenerOnKioskMode() {
        var primaryStage = mock(Stage.class);
        when(primaryStage.fullScreenProperty()).thenReturn(fullscreenProperty);
        when(applicationConfig.isKioskMode()).thenReturn(true);

        primaryStageListener.get().changed(null, null, primaryStage);
        WaitForAsyncUtils.waitForFxEvents();

        verify(maximizeService).setMaximized(false);
        verify(primaryStage).setFullScreenExitKeyCombination(KeyCombination.NO_MATCH);
        verify(primaryStage).setFullScreen(true);
    }

    @Test
    void testOnMaximizeEvent() {
        var primaryStage = mock(Stage.class);
        doAnswer(invocation -> {
            fullscreenProperty.set(invocation.getArgument(0, Boolean.class));
            return null;
        }).when(primaryStage).setFullScreen(isA(Boolean.class));
        when(primaryStage.fullScreenProperty()).thenReturn(fullscreenProperty);

        primaryStageListener.get().changed(null, null, primaryStage);
        WaitForAsyncUtils.waitForFxEvents();
        eventPublisher.publish(new MaximizeEvent(this, true));
        WaitForAsyncUtils.waitForFxEvents();

        verify(primaryStage).setFullScreen(true);
    }
}