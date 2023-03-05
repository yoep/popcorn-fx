package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.StartScreen;
import com.github.yoep.popcorn.backend.settings.models.UISettings;
import com.github.yoep.popcorn.ui.events.CategoryChangedEvent;
import javafx.scene.control.Label;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.context.ApplicationEventPublisher;
import org.testfx.framework.junit5.ApplicationExtension;

import java.net.URL;
import java.util.ResourceBundle;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class SidebarControllerTest {
    @Mock
    private ApplicationConfig applicationConfig;
    @Mock
    private ApplicationSettings applicationSettings;
    @Mock
    private UISettings settings;
    @Mock
    private ApplicationEventPublisher eventPublisher;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private SidebarController controller;

    @BeforeEach
    void setUp() {
        lenient().when(applicationConfig.getSettings()).thenReturn(applicationSettings);
        lenient().when(applicationSettings.getUiSettings()).thenReturn(settings);
        controller.searchIcon = new Icon();
        controller.movieIcon = new Icon();
        controller.movieText = new Label();
        controller.serieIcon = new Icon();
        controller.serieText = new Label();
        controller.favoriteIcon = new Icon();
        controller.favoriteText = new Label();

        controller.movieText.setLabelFor(controller.movieIcon);
        controller.serieText.setLabelFor(controller.serieIcon);
        controller.favoriteText.setLabelFor(controller.favoriteIcon);
    }

    @Test
    void testInitialize_shouldActivePreferredDefaultCategory() {
        when(settings.getStartScreen()).thenReturn(StartScreen.SERIES);
        var expectedEvent = new CategoryChangedEvent(controller, Category.SERIES);

        controller.initialize(url, resourceBundle);

        verify(eventPublisher, timeout(250)).publishEvent(expectedEvent);
        assertFalse(controller.movieIcon.getStyleClass().contains(SidebarController.ACTIVE_STYLE));
        assertFalse(controller.movieText.getStyleClass().contains(SidebarController.ACTIVE_STYLE));
        assertFalse(controller.favoriteIcon.getStyleClass().contains(SidebarController.ACTIVE_STYLE));
        assertFalse(controller.favoriteText.getStyleClass().contains(SidebarController.ACTIVE_STYLE));
        assertTrue(controller.serieIcon.getStyleClass().contains(SidebarController.ACTIVE_STYLE));
        assertTrue(controller.serieText.getStyleClass().contains(SidebarController.ACTIVE_STYLE));
    }

    @Test
    void testOnCategoryClicked() {
        var event = mock(MouseEvent.class);
        when(settings.getStartScreen()).thenReturn(StartScreen.MOVIES);
        when(event.getSource()).thenReturn(controller.favoriteIcon);
        controller.initialize(url, resourceBundle);

        controller.onCategoryClicked(event);

        verify(event).consume();
        verify(eventPublisher).publishEvent(new CategoryChangedEvent(controller, Category.FAVORITES));
    }

    @Test
    void testOnCategoryPressed() {
        var event = mock(KeyEvent.class);
        when(settings.getStartScreen()).thenReturn(StartScreen.MOVIES);
        when(event.getSource()).thenReturn(controller.favoriteIcon);
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        controller.initialize(url, resourceBundle);

        controller.onCategoryPressed(event);

        verify(event).consume();
        verify(eventPublisher).publishEvent(new CategoryChangedEvent(controller, Category.FAVORITES));
    }
}