package com.github.yoep.popcorn.ui.view.services;

import com.github.spring.boot.javafx.view.ViewManager;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.UISettings;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.stage.Stage;
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
    private SettingsService settingsService;
    @Mock
    private ApplicationSettings settings;
    @Mock
    private UISettings uiSettings;
    @InjectMocks
    private MaximizeService service;

    private final ObjectProperty<Stage> primaryStageProperty = new SimpleObjectProperty<>();
    private final BooleanProperty maximizedProperty = new SimpleBooleanProperty();

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
        when(settingsService.getSettings()).thenReturn(settings);
        when(settings.getUiSettings()).thenReturn(uiSettings);
        service.init();

        primaryStageProperty.set(stage);
        maximizedProperty.set(true);

        assertTrue(service.isMaximized());
    }
}