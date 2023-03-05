package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.StartScreen;
import com.github.yoep.popcorn.backend.settings.models.UISettings;
import com.github.yoep.popcorn.ui.events.CategoryChangedEvent;
import com.github.yoep.popcorn.ui.events.ShowSettingsEvent;
import javafx.animation.Animation;
import javafx.scene.control.Label;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.ColumnConstraints;
import javafx.scene.layout.GridPane;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.MalformedURLException;
import java.net.URL;
import java.util.ResourceBundle;

import static org.junit.jupiter.api.Assertions.*;
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
    private EventPublisher eventPublisher;
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private SidebarController controller;

    @BeforeEach
    void setUp() throws MalformedURLException {
        lenient().when(applicationConfig.getSettings()).thenReturn(applicationSettings);
        lenient().when(applicationSettings.getUiSettings()).thenReturn(settings);
        controller.sidebar = new GridPane();
        controller.searchIcon = new Icon();
        controller.movieIcon = new Icon();
        controller.movieText = new Label();
        controller.serieIcon = new Icon();
        controller.serieText = new Label();
        controller.favoriteIcon = new Icon();
        controller.favoriteText = new Label();
        controller.settingsIcon = new Icon();
        controller.settingsText = new Label();

        controller.sidebar.getColumnConstraints().add(new ColumnConstraints());
        controller.sidebar.getColumnConstraints().add(new ColumnConstraints());
        controller.movieText.setLabelFor(controller.movieIcon);
        controller.serieText.setLabelFor(controller.serieIcon);
        controller.favoriteText.setLabelFor(controller.favoriteIcon);
        controller.settingsText.setLabelFor(controller.settingsIcon);

        url = new URL("http://localhost");
    }

    @Test
    void testInitialize_shouldActivePreferredDefaultCategory() {
        when(settings.getStartScreen()).thenReturn(StartScreen.SERIES);
        var expectedEvent = new CategoryChangedEvent(controller, Category.SERIES);

        controller.initialize(url, resourceBundle);

        verify(eventPublisher, timeout(250)).publish(expectedEvent);
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
        verify(eventPublisher).publish(new CategoryChangedEvent(controller, Category.FAVORITES));
    }

    @Test
    void testOnCategoryClicked_LabelForIcon() {
        var movieEvent = mock(MouseEvent.class);
        when(movieEvent.getSource()).thenReturn(controller.movieText);
        var serieEvent = mock(MouseEvent.class);
        when(serieEvent.getSource()).thenReturn(controller.serieText);
        var favoriteEvent = mock(MouseEvent.class);
        when(favoriteEvent.getSource()).thenReturn(controller.favoriteText);

        controller.onCategoryClicked(movieEvent);
        verify(movieEvent).consume();
        verify(eventPublisher).publish(new CategoryChangedEvent(controller, Category.MOVIES));

        controller.onCategoryClicked(serieEvent);
        verify(serieEvent).consume();
        verify(eventPublisher).publish(new CategoryChangedEvent(controller, Category.SERIES));

        controller.onCategoryClicked(favoriteEvent);
        verify(favoriteEvent).consume();
        verify(eventPublisher).publish(new CategoryChangedEvent(controller, Category.FAVORITES));
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
        verify(eventPublisher).publish(new CategoryChangedEvent(controller, Category.FAVORITES));
    }

    @Test
    void testOnHovering() {
        var event = mock(MouseEvent.class);
        when(settings.getStartScreen()).thenReturn(StartScreen.MOVIES);
        controller.initialize(url, resourceBundle);

        controller.onHovering(event);
        WaitForAsyncUtils.waitForFxEvents();

        assertEquals(Animation.Status.RUNNING, controller.slideAnimation.getStatus());
        assertEquals(1, controller.slideAnimation.getToValue());
    }

    @Test
    void testOnHoverStopped() {
        var event = mock(MouseEvent.class);
        when(settings.getStartScreen()).thenReturn(StartScreen.MOVIES);
        controller.initialize(url, resourceBundle);

        controller.onHoverStopped(event);
        WaitForAsyncUtils.waitForFxEvents();

        assertEquals(Animation.Status.RUNNING, controller.slideAnimation.getStatus());
        assertEquals(0, controller.slideAnimation.getToValue());
    }

    @Test
    void testOnSettingsClicked() {
        var event = mock(MouseEvent.class);
        when(settings.getStartScreen()).thenReturn(StartScreen.MOVIES);
        controller.initialize(url, resourceBundle);

        controller.onSettingsClicked(event);

        verify(event).consume();
        verify(eventPublisher).publish(new ShowSettingsEvent(controller));
    }

    @Test
    void testOnSettingsPressed() {
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        when(settings.getStartScreen()).thenReturn(StartScreen.MOVIES);
        controller.initialize(url, resourceBundle);

        controller.onSettingsPressed(event);

        verify(event).consume();
        verify(eventPublisher).publish(new ShowSettingsEvent(controller));
    }
}