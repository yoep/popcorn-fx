package com.github.yoep.popcorn.ui.screen;

import com.github.spring.boot.javafx.view.ViewManager;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.settings.OptionsService;
import com.github.yoep.popcorn.backend.settings.models.ApplicationOptions;
import javafx.beans.property.ReadOnlyProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.beans.value.ChangeListener;
import javafx.stage.Stage;
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
    private OptionsService optionsService;
    @Mock
    private ApplicationOptions options;
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @InjectMocks
    private ScreenServiceImpl screenService;

    private AtomicReference<ChangeListener<Stage>> primaryStageListener = new AtomicReference<>();

    @Test
    void testToggleFullscreen() {
        var primaryStage = mock(Stage.class);
        var primaryStageProperty = mock(ReadOnlyProperty.class);
        var fullscreenProperty = new SimpleBooleanProperty();
        var future = new CompletableFuture<Boolean>();
        doAnswer(invocation -> {
            primaryStageListener.set(invocation.getArgument(0, ChangeListener.class));
            return null;
        }).when(primaryStageProperty).addListener(isA(ChangeListener.class));
        doAnswer(invocation -> {
            fullscreenProperty.set(invocation.getArgument(0, Boolean.class));
            return null;
        }).when(primaryStage).setFullScreen(isA(Boolean.class));
        when(viewManager.primaryStageProperty()).thenReturn(primaryStageProperty);
        when(optionsService.options()).thenReturn(options);
        when(primaryStage.fullScreenProperty()).thenReturn(fullscreenProperty);
        screenService.fullscreenProperty().addListener((observable, oldValue, newValue) -> future.complete(newValue));
        screenService.init();

        primaryStageListener.get().changed(null, null, primaryStage);
        screenService.toggleFullscreen();

        assertEquals(true, future.join());
        assertTrue(screenService.isFullscreen());
    }
}