package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.ui.view.ViewManager;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.stage.Stage;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class MaximizeServiceTest {
    @Mock
    private ViewManager viewManager;
    @Mock
    private ApplicationConfig applicationConfig;
    @Mock
    private ApplicationSettings settings;
    private MaximizeService service;

    private final ObjectProperty<Stage> primaryStageProperty = new SimpleObjectProperty<>();
    private final BooleanProperty maximizedProperty = new SimpleBooleanProperty();

    @BeforeEach
    void setUp() {
        lenient().when(viewManager.primaryStageProperty()).thenReturn(primaryStageProperty);
        lenient().when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(settings));
        lenient().when(settings.getUiSettings()).thenReturn(ApplicationSettings.UISettings.newBuilder().build());

        service = new MaximizeService(viewManager, applicationConfig);
    }

    @Test
    void testMinimize_whenInvoked_shouldIconizeThePrimaryStage() {
        var stage = mock(Stage.class);
        when(viewManager.getPrimaryStage()).thenReturn(Optional.of(stage));

        service.minimize();

        verify(stage).setIconified(true);
    }

    @Test
    void testListeners_whenPrimaryStageIsChanged_shouldStoreMaximizeValueWhenStageStateIsChanged() {
        var stage = mock(Stage.class);
        when(stage.maximizedProperty()).thenReturn(maximizedProperty);

        primaryStageProperty.set(stage);
        maximizedProperty.set(true);

        assertTrue(service.isMaximized());
    }

    @Test
    void testMaximizeChanged() {
        var uiSettings = new AtomicReference<ApplicationSettings.UISettings>();
        doAnswer(invocations -> {
            uiSettings.set(invocations.getArgument(0, ApplicationSettings.UISettings.class));
            return null;
        }).when(applicationConfig).update(isA(ApplicationSettings.UISettings.class));

        service.setMaximized(true);
        verify(applicationConfig).update(isA(ApplicationSettings.UISettings.class));
        assertTrue(uiSettings.get().getMaximized(), "expected maximized to be true");

        service.setMaximized(false);
        verify(applicationConfig, times(2)).update(isA(ApplicationSettings.UISettings.class));
        assertFalse(uiSettings.get().getMaximized(), "expected maximized to be false");
    }
}