package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.UISettings;
import com.github.yoep.popcorn.ui.view.ViewManager;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.stage.Stage;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class MaximizeServiceTest {
    @Mock
    private ViewManager viewManager;
    @Mock
    private ApplicationConfig applicationConfig;
    @Mock
    private ApplicationSettings settings;
    @Mock
    private UISettings uiSettings;
    @InjectMocks
    private MaximizeService service;

    private final ObjectProperty<Stage> primaryStageProperty = new SimpleObjectProperty<>();
    private final BooleanProperty maximizedProperty = new SimpleBooleanProperty();

    @BeforeEach
    void setUp() {
        lenient().when(viewManager.primaryStageProperty()).thenReturn(primaryStageProperty);
        lenient().when(applicationConfig.getSettings()).thenReturn(settings);
        lenient().when(settings.getUiSettings()).thenReturn(uiSettings);
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
        when(viewManager.primaryStageProperty()).thenReturn(primaryStageProperty);
        when(stage.maximizedProperty()).thenReturn(maximizedProperty);
        when(applicationConfig.getSettings()).thenReturn(settings);
        when(settings.getUiSettings()).thenReturn(uiSettings);
        service.init();

        primaryStageProperty.set(stage);
        maximizedProperty.set(true);

        assertTrue(service.isMaximized());
    }

    @Test
    void testMaximizeChanged() {
        service.init();

        service.setMaximized(true);
        verify(uiSettings).setMaximized(true);
        verify(applicationConfig).update(uiSettings);

        service.setMaximized(false);
        verify(uiSettings).setMaximized(false);
        verify(applicationConfig, times(2)).update(uiSettings);
    }
}